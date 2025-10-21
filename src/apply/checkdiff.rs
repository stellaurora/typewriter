//! Strategy responsible for checking if the file is diff
//! and aborting if the user does not confirm for all files
//! (atomicity in all)

use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufReader, Read},
    path::PathBuf,
};

use anyhow::{Context, bail};
use inquire::Confirm;
use serde::{Deserialize, Serialize};
use xxhash_rust::xxh3::Xxh3;

use crate::{
    apply::strategy::ApplyStrategy,
    cleanpath::CleanPath,
    config::ROOT_CONFIG,
    file::{TrackedFile, TrackedFileList},
};

/// Which strategy to use for the checkdiff stage?
/// This stage will prompt the user whether or not
/// to continue with the apply if the files are found to
/// be different.
#[derive(Deserialize, Debug)]
pub enum FileCheckDiffStrategy {
    // Checks by using XXHash for diff
    #[serde(rename = "xxhash")]
    XXHashDiff,

    // Dont check if the files are different
    #[serde(rename = "disabled")]
    Disabled,
}

/// Checksum entry in stored metadata file
#[derive(Deserialize, Serialize, Debug, Default)]
struct ChecksumEntries {
    entries: HashMap<PathBuf, String>,
}

impl Default for FileCheckDiffStrategy {
    fn default() -> Self {
        Self::XXHashDiff
    }
}

impl FileCheckDiffStrategy {
    /// Returns the file path to the checksum storage
    /// file in the metadata directory
    fn get_checksum_file_path() -> anyhow::Result<PathBuf> {
        // Get config to get file path.
        let apply_conf = &ROOT_CONFIG.get_config().apply;

        Ok(apply_conf
            .apply_metadata_dir
            .join(&apply_conf.checkdiff_file_name)
            .clean_path()?)
    }

    fn read_checksum_entries() -> anyhow::Result<ChecksumEntries> {
        // Get file path..
        let path = FileCheckDiffStrategy::get_checksum_file_path()?;

        if !path.exists() {
            return Ok(ChecksumEntries::default());
        }

        // Read in from file path
        let file_content = fs::read_to_string(&path)
            .with_context(|| format!("While trying to read checksum storage file {:?}", path))?;

        ron::from_str(&file_content).with_context(|| {
            format!(
                "While trying to parse checksum storage file {:?}, Has it been tampered with?",
                path
            )
        })
    }

    fn write_checksum_entries(checksum_entries: &ChecksumEntries) -> anyhow::Result<()> {
        let path = FileCheckDiffStrategy::get_checksum_file_path()?;

        // Make parent directories if it doesn't exist already.
        if let Some(create_result) = path.parent().map(|path| fs::create_dir_all(path)) {
            create_result?;
        }

        // Serialize back and write to file.
        let storage_string = ron::to_string(checksum_entries)
            .with_context(|| format!("While trying to serialize checksum storage file"))?;

        fs::write(&path, storage_string)
            .with_context(|| format!("While trying to write checksum storage file {:?}", path))?;

        Ok(())
    }
}

/// Hash function that should take a file path and return its hash as a string
type HashFile = fn(file_path: &PathBuf) -> anyhow::Result<String>;

/// XXHASH version of hashing a file in from file path

fn xxhash_hash_file(path: &PathBuf) -> anyhow::Result<String> {
    let file = File::open(path).with_context(|| format!("While trying to hash file {:?}", path))?;
    let mut reader = BufReader::new(file);

    // Buffer 64kb reads in from file at a time for hashing
    let mut hasher = Xxh3::new();
    let mut buffer = [0u8; 65536];

    loop {
        // Update hash.
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{}", hasher.digest()))
}

/// Checks if the file is different
/// and promps the client whether to continue
/// or not based on file-specific cases, on Err then
/// client wishes to abort operation.
fn hash_check_diff(
    checksum_entries: &ChecksumEntries,
    file: &TrackedFile,
    hash_fn: HashFile,
) -> anyhow::Result<()> {
    // New file, not yet in checkdiff, prompt user if not set to skip.
    if !checksum_entries.entries.contains_key(&file.destination) {
        // Skip checkdiff new file.
        if ROOT_CONFIG.get_config().apply.skip_checkdiff_new {
            return Ok(());
        }

        // Prompt for this case.
        let to_overwrite = Confirm::new(
            format!(
                "No existing hash checksum was found for {:?} referenced in configuration file {:?}, Do you want to proceed? This will overwrite the file.",
                file.destination, file.src
            )
            .as_str(),
        )
        .with_default(false)
        .prompt()?;

        if !to_overwrite {
            bail!("Aborting apply operation")
        }

        return Ok(());
    }

    // Expected hash
    let expected_hash = checksum_entries.entries.get(&file.destination).unwrap();

    // Hash file
    let hash_result = hash_fn(&file.destination)?;

    // Same hash, no diff
    if hash_result == *expected_hash {
        return Ok(());
    }

    // Should we overwrite even if they're different?
    let to_overwrite = Confirm::new(
        format!(
            "Checksum differs for file {:?} referenced by configuration file {:?} (it was changed between last apply), Continue and overwrite?",
            file.destination, file.src
        )
        .as_str(),
    )
    .with_default(false)
    .prompt()?;

    if !to_overwrite {
        bail!("Aborting apply operation")
    }

    Ok(())
}

/// Run's the hash strategy check
/// (before copy) phase, checking
/// if the hashes match or have changed.
fn run_hash_strategy_before_copy(files: &TrackedFileList, hash_fn: HashFile) -> anyhow::Result<()> {
    // Use checksum storage file.
    let checksum_entries = FileCheckDiffStrategy::read_checksum_entries()?;

    // No entries? Confirm with
    if checksum_entries.entries.len() < 1 {
        let to_overwrite = Confirm::new(
            format!(
                "No existing hash checksum storage was found, Do you want to proceed? This will overwrite all to-apply files regardless of changes.",
            )
            .as_str(),
        )
        .with_default(false)
        .prompt()?;

        if !to_overwrite {
            bail!("Aborting apply operation")
        }

        return Ok(());
    }

    // Check diff of every file.
    for file in &files.0 {
        hash_check_diff(&checksum_entries, file, hash_fn)?;
    }

    Ok(())
}

/// Saves all the files into the checkdiff
/// file using hash_fn to produce a hash
/// for future use for diff checking
fn run_hash_strategy_after_copy(files: &TrackedFileList, hash_fn: HashFile) -> anyhow::Result<()> {
    let mut checksum_entries: HashMap<PathBuf, String> = HashMap::new();

    for file in &files.0 {
        // Insert with the new hash..
        checksum_entries.insert(
            PathBuf::from(&file.destination),
            hash_fn(&file.destination)?,
        );
    }

    // Write to the file
    FileCheckDiffStrategy::write_checksum_entries(&ChecksumEntries {
        entries: checksum_entries,
    })?;

    Ok(())
}

impl ApplyStrategy for FileCheckDiffStrategy {
    fn run_before_apply(self: &Self, files: &mut TrackedFileList) -> anyhow::Result<()> {
        // Specific method for checking file diff.
        match self {
            FileCheckDiffStrategy::Disabled => Ok(()),
            FileCheckDiffStrategy::XXHashDiff => {
                run_hash_strategy_before_copy(files, xxhash_hash_file)
            }
        }
    }

    fn run_after_apply(self: &Self, files: &mut TrackedFileList) -> anyhow::Result<()> {
        // Method for writing checksum back after copying
        match self {
            FileCheckDiffStrategy::Disabled => Ok(()),
            FileCheckDiffStrategy::XXHashDiff => {
                run_hash_strategy_after_copy(files, xxhash_hash_file)
            }
        }
    }
}
