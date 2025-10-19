//! Parsing configuration file for typewriter

use ansi_term::Colour::{Red, Yellow};
use anyhow::{Context, bail};
use serde::Deserialize;
use std::{
    collections::{HashMap, VecDeque},
    fs,
    path::PathBuf,
};

use crate::{file::TrackedFile, git::Git};

/// Configuration for the a file in the typewriter system
///
/// config is not utilised outside of the root
/// file referenced directly by commands.
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Typewriter {
    // Global typewriter configuration options.
    config: Option<Config>,

    // Links to other files to include in the configuration
    #[serde(rename = "link", default)]
    links: Vec<ConfigLink>,

    // Files to update in the system
    #[serde(rename = "file", default)]
    files: Vec<TrackedFile>,
}

/// Global typewriter configuration options.
///
/// Can only be used by the root typewriter
/// configuration file referenced in commands
/// in order to keep tracking configuration simple
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    // Git related configuration options
    // such as automatic commits on apply.
    git: Git,
}

/// Links to other typewriter configuration files
///
/// Can be used in any typewriter configuration file
/// to "include" it into the overall configuration
/// in order to have better modularity/cleaner file structure
/// for the system configuration
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ConfigLink {
    // The file that is being linked to
    file: String,
}

fn validate_path(file_path: &PathBuf, origin_file: &PathBuf) -> anyhow::Result<()> {
    // Check if path exists, else error.
    if !file_path.exists() {
        bail!(
            "{} {} {}",
            Red.paint(format!(
                "Referenced file {:?} by {:?} doesn't exist, maybe try running",
                file_path, origin_file
            )),
            Red.underline().paint("init"),
            Red.paint("first")
        );
    }

    Ok(())
}

/// Parses an individual configuration file
fn parse_single_config(file_path: &PathBuf) -> anyhow::Result<Typewriter> {
    // Read in content and try parse using toml
    let file_content = fs::read_to_string(&file_path)
        .with_context(|| format!("while trying to read config {:?}", file_path))?;
    let config: Typewriter = toml::from_str(&file_content)
        .with_context(|| format!("while trying to parse config {:?}", file_path))?;
    Ok(config)
}

/// Parses the configuration file supplied in as per
/// the expected config in typewriter
///
/// The result is all of the included typewriter files together.
/// Does validation/handles validation on paths linked, but not on the root file_path provided.
pub fn parse_config(file_path: PathBuf) -> anyhow::Result<Vec<Typewriter>> {
    // Hashmap of configs, since we need to track whether or not
    // a config has already been included to break recursive-deps
    let mut config_map: HashMap<PathBuf, Typewriter> = HashMap::new();

    // Track unprocessed linked configs
    let mut unproc_configs: VecDeque<PathBuf> = VecDeque::new();
    unproc_configs.push_back(fs::canonicalize(&file_path)?);

    // Original config
    let root_file_path = fs::canonicalize(file_path)?;

    // Go over all unprocessed configs
    while let Some(this_path) = unproc_configs.pop_front() {
        // Already processed, skip
        if config_map.contains_key(&this_path) {
            continue;
        }

        // Process this config, add its other configs to the unproc list
        let config = parse_single_config(&this_path)?;

        // Warn about unsued config
        if !(this_path == root_file_path) && config.config.is_some() {
            println!(
                "Warning: {}",
                Yellow.paint(format!(
                    "Unused global config in {:?}, since it is not the root file",
                    this_path
                ))
            )
        }

        for link in &config.links {
            // Create this linked path from the perspective of this path
            let mut linked_path = this_path.clone();

            // Parent -> directory + back on the file path
            linked_path.pop();
            linked_path.push(&link.file);
            linked_path = fs::canonicalize(linked_path)?;

            // Add this unprocessed path to the list for later checking..
            validate_path(&linked_path, &this_path)?;
            if !config_map.contains_key(&linked_path) && !unproc_configs.contains(&linked_path) {
                unproc_configs.push_back(linked_path);
            }
        }
        config_map.insert(this_path, config);
    }

    Ok(config_map.into_values().collect())
}
