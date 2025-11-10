use std::path::PathBuf;

use ansi_term::Color::{Black, White};
use serde::Deserialize;

use crate::{
    apply::{
        checkdiff::FileCheckDiffStrategy, fileperm::FilePermissionStrategy,
        strategy::ApplyStrategy, tempcopy::TemporaryCopyStrategy,
    },
    file::TrackedFileList,
};

// Strategy trait for dyn handling
pub mod strategy;

// Preprocessing handling
pub mod variables;

// Temporary copy handling
pub mod tempcopy;

// Checking diff first before writing
pub mod checkdiff;

// Hooks/command execution at certain stages.
pub mod hooks;

// File permission checking
pub mod fileperm;

/// Configuration options to apply command
/// files
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Apply {
    // Whether or not to automatically
    // skip files which cannot be properly
    // applied using typewriter
    #[serde(default)]
    pub auto_skip_unable_apply: bool,

    // Whether or not to skip the confirmation
    // prompt for the apply command
    #[serde(default = "default_is_true")]
    pub confirm_apply: bool,

    // Directory to place metadata/temporary files in
    // for the apply command
    #[serde(default = "default_tempfile_dir")]
    pub apply_metadata_dir: PathBuf,

    // Strategy for temporary copying functionality
    // for backup if failure occurs while applying
    #[serde(default)]
    pub temp_copy_strategy: TemporaryCopyStrategy,

    // Delimiter for temp copy file names
    // to replace path seperator with
    #[serde(default = "default_temp_copy_path_delim")]
    pub temp_copy_path_delim: String,

    // Should we clean up temporary copy files at
    // the end of the apply if it succeeded?
    #[serde(default = "default_is_true")]
    pub cleanup_files: bool,

    // Name of the checkdiff storage file for
    // checkdiff in the metadata path
    #[serde(default = "default_checkdiff_file_name")]
    pub checkdiff_file_name: String,

    // Strategy of the checkdiff for
    // checking if the file was modified
    // out of the system just-in-case to not
    // overwrite potential wanted files.
    #[serde(default)]
    pub checkdiff_strategy: FileCheckDiffStrategy,

    // Global toggle for whether checkdiff
    // should be permitted to skip files if
    // the content is the same in source & destination
    //  or not
    #[serde(default = "default_is_true")]
    pub checkdiff_skip_same: bool,

    // Skip prompting for confirmation if the entry is new to the checkdiff file
    // and the checkdiff file was already initialised
    //
    // so if the file was not originally added to typewriter
    // yet or hasn't been applied yet, then do not prompt for
    // overwrite.
    //
    // (first initialisation will always prompt for a non-disabled Strategy)
    #[serde(default)]
    pub skip_checkdiff_new: bool,

    // Strategy for checking file permissions and
    // optionally creating missing destination files
    #[serde(default)]
    pub file_permission_strategy: FilePermissionStrategy,
}

/// I think we have to sadly re-duplicate serde default here
/// for if the struct itself is missing.
impl Default for Apply {
    fn default() -> Self {
        Self {
            auto_skip_unable_apply: Default::default(),
            confirm_apply: default_is_true(),
            apply_metadata_dir: default_tempfile_dir(),
            temp_copy_strategy: Default::default(),
            temp_copy_path_delim: default_temp_copy_path_delim(),
            cleanup_files: default_is_true(),
            checkdiff_file_name: default_checkdiff_file_name(),
            checkdiff_strategy: Default::default(),
            skip_checkdiff_new: Default::default(),
            checkdiff_skip_same: default_is_true(),
            file_permission_strategy: Default::default(),
        }
    }
}

fn default_is_true() -> bool {
    true
}

/// Default checksum storage file name
fn default_checkdiff_file_name() -> String {
    String::from(".checkdiff")
}

/// Default delimiter for directory path in tempcopy file names
fn default_temp_copy_path_delim() -> String {
    String::from("-")
}

/// Default directory for tempfiles
fn default_tempfile_dir() -> PathBuf {
    PathBuf::from(".typewriter")
}

/// Run apply copy with atomicity and transactional behavior
pub fn apply(
    mut files: TrackedFileList,
    strategies: Vec<&dyn ApplyStrategy>,
) -> anyhow::Result<()> {
    let result = run_apply_strategies(&mut files, &strategies);

    if let Err(e) = result {
        log::error!("Apply operation failed, initiating rollback");
        // Run rollback in reverse order to undo operations properly
        for strategy in strategies.iter().rev() {
            let _ = strategy.run_on_failure(&mut files);
        }
        return Err(e);
    }

    Ok(())
}

fn run_apply_strategies(
    files: &mut TrackedFileList,
    strategies: &[&dyn ApplyStrategy],
) -> anyhow::Result<()> {
    for strategy in strategies {
        strategy.run_before_apply(files)?;
    }

    for file in &mut files.0 {
        for strategy in strategies {
            strategy.run_before_apply_file(file)?;
        }
    }

    for file in &mut files.0 {
        for strategy in strategies {
            strategy.run_after_apply_file(file)?;
        }

        println!(
            "[{}] {:?} to {:?} {}",
            White.bold().paint("APPLIED"),
            file.file,
            file.destination,
            Black.dimmed().paint(format!("[ref: {:?}]", file.src))
        );
    }

    for strategy in strategies {
        strategy.run_after_apply(files)?;
    }

    Ok(())
}
