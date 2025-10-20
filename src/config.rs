//! Configuration structs and helpers for typewriter

use std::{
    ops::{Deref, DerefMut},
    sync::OnceLock,
};

use serde::Deserialize;

/// Wrapper around oncelock config to help
/// retrieving config options globally.
pub struct GlobalConfig(OnceLock<Config>);

// Configuration from the root file oncelock that will be
// filled in once the config has been gotten
pub static ROOT_CONFIG: GlobalConfig = GlobalConfig(OnceLock::new());

use crate::{apply::Apply, file::TrackedFileList, git::Git, parse_config::ConfigLink};

/// Wrapper with helper methods for interacting
/// with a list of typewriter configs
pub struct TypewriterConfigs(Vec<Typewriter>);

/// Configuration for the a file in the typewriter system
///
/// config is not utilised outside of the root
/// file referenced directly by commands.
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Typewriter {
    // Global typewriter configuration options.
    pub config: Option<Config>,

    // Links to other files to include in the configuration
    #[serde(rename = "link", default)]
    pub links: Vec<ConfigLink>,

    // Files to update in the system
    #[serde(rename = "file", default)]
    pub files: TrackedFileList,
}

/// Global typewriter configuration options.
///
/// Can only be used by the root typewriter
/// configuration file referenced in commands
/// in order to keep tracking configuration simple
#[derive(Deserialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    // Git related configuration options
    // such as automatic commits on apply.
    #[serde(default)]
    pub git: Git,

    // Configuration options relating to
    // initial file permission check
    #[serde(default)]
    pub apply: Apply,
}

impl Deref for TypewriterConfigs {
    type Target = Vec<Typewriter>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TypewriterConfigs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromIterator<Typewriter> for TypewriterConfigs {
    fn from_iter<T: IntoIterator<Item = Typewriter>>(iter: T) -> Self {
        // Collect into wrapped form
        let iter_vec: Vec<Typewriter> = iter.into_iter().collect();
        TypewriterConfigs(iter_vec)
    }
}

impl TypewriterConfigs {
    /// Decomposes down all of the typewriter configs
    /// into only their tracked files as a list.
    pub fn all_files(self: Self) -> TrackedFileList {
        // Insert all tracked file lists entries into one new list
        self.0
            .into_iter()
            .flat_map(|config| config.files.0)
            .collect()
    }
}

impl GlobalConfig {
    /// Set's the global config
    /// in the system to be this config
    pub fn set_config(self: &Self, global_config: Config) {
        ROOT_CONFIG.0.get_or_init(|| global_config);
    }

    /// Get's the root config
    /// or returns an error if it could not succesfully be gotten
    pub fn get_config(self: &Self) -> &Config {
        ROOT_CONFIG.0.wait()
    }
}
