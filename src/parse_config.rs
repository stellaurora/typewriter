//! Parsing configuration file for typewriter

use anyhow::{Context, bail};
use log::warn;
use path_absolutize::Absolutize;
use serde::Deserialize;
use std::{
    collections::{HashMap, VecDeque},
    fs,
    path::PathBuf,
};

use crate::config::*;

/// Links to other typewriter configuration files
///
/// Can be used in any typewriter configuration file
/// to "include" it into the overall configuration
/// in order to have better modularity/cleaner file structure
/// for the system configuration
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ConfigLink {
    file: PathBuf,
}

/// Is this link to another file (from origin_file) valid?
fn validate_link(file_path: &PathBuf, origin_file: &PathBuf) -> anyhow::Result<()> {
    // Check if path exists, else error.
    if !file_path.exists() {
        bail!(
            "File {:?} referenced by link in configuration file {:?} does not exist",
            file_path,
            origin_file
        );
    }

    Ok(())
}

/// Parses an individual configuration file
fn parse_single_config(file_path: &PathBuf) -> anyhow::Result<Typewriter> {
    // Read in content and try parse using toml
    let file_content = fs::read_to_string(&file_path)
        .with_context(|| format!("While trying to read configuration file {:?}", file_path))?;

    let mut config: Typewriter = toml::from_str(&file_content)
        .with_context(|| format!("While trying to parse configuration file {:?}", file_path))?;

    // Add dir to the config path for file.
    config
        .files
        .iter_mut()
        .try_for_each(|tracked_file| tracked_file.add_typewriter_dir(file_path))?;

    // Add dir to variable for debug info.
    config
        .variables
        .iter_mut()
        .try_for_each(|variable| variable.add_typewriter_dir(file_path))?;

    Ok(config)
}

/// Processes a list of config links adding them to
/// the unprocessed configs vecdeque if they are not
/// already in the config_map supplied.
///
/// The current path is supplied for logging purposes.
fn process_links(
    unprocessed_configs: &mut VecDeque<PathBuf>,
    current_path: &PathBuf,
    links: &Vec<ConfigLink>,
    config_map: &mut HashMap<PathBuf, Typewriter>,
) -> anyhow::Result<()> {
    for link in links {
        // Create this linked path from the perspective of this path
        let parent = current_path
            .parent()
            .context("Configuration file has no parent directory")?;
        let linked_path = PathBuf::from(parent.join(&link.file).absolutize()?);

        // Add this unprocessed path to the list for later checking..
        validate_link(&linked_path, &current_path)?;
        if !config_map.contains_key(&linked_path) && !unprocessed_configs.contains(&linked_path) {
            unprocessed_configs.push_back(linked_path);
        }
    }

    Ok(())
}

/// Parses the configuration file supplied in as per
/// the expected config in typewriter
///
/// The result is all of the included typewriter files together in a vec.
/// which are all of the "linked" ones, and the first half of the tuple is the root.
pub fn parse_config(file_path: PathBuf) -> anyhow::Result<(Typewriter, TypewriterConfigs)> {
    if !file_path.exists() {
        bail!(
            "Supplied root configuration file {:?} does not exist",
            file_path
        );
    }

    // Hashmap of configs, since we need to track whether or not
    // a config has already been included to break recursive-deps
    let mut config_map: HashMap<PathBuf, Typewriter> = HashMap::new();

    // Track unprocessed linked configs, our root is unprocessed
    let mut unprocessed_configs: VecDeque<PathBuf> = VecDeque::new();
    unprocessed_configs.push_back(file_path.clone());

    // Go over all unprocessed configs
    while let Some(current_path) = unprocessed_configs.pop_front() {
        // Already processed, skip
        if config_map.contains_key(&current_path) {
            continue;
        }

        // Process this config, add its other configs to the unproc list
        let config = parse_single_config(&current_path)?;

        // Warn about unsued config
        if !(current_path == file_path) && config.config.is_some() {
            warn!(
                "Unused global config in {:?}, since it is not the root file",
                current_path
            )
        }

        // Process all of the linked files and add them to unprocessed_configs.
        process_links(
            &mut unprocessed_configs,
            &current_path,
            &config.links,
            &mut config_map,
        )?;
        config_map.insert(current_path, config);
    }

    // Get root back from config_map, shouldn't ever not exist (doesn't make sense)
    Ok((
        config_map.remove(&file_path).unwrap(),
        config_map.into_values().collect(),
    ))
}
