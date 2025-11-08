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

use crate::{
    apply::{
        Apply,
        hooks::{HookList, HooksConfig},
    },
    command::CommandConfig,
    file::TrackedFileList,
    parse_config::ConfigLink,
    vars::{VariableConfig, VariableList},
};

/// Wrapper with helper methods for interacting
/// with a list of typewriter configs
pub struct TypewriterConfigs(pub Vec<Typewriter>);

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
    #[serde(
        alias = "link",
        alias = "include",
        alias = "use",
        alias = "import",
        default
    )]
    pub links: Vec<ConfigLink>,

    // Variables for preprocessing
    #[serde(alias = "var", alias = "variable", alias = "define", default)]
    pub variables: VariableList,

    // Files to update in the system
    #[serde(alias = "file", alias = "track", default)]
    pub files: TrackedFileList,

    // Commands that are executed globally
    #[serde(alias = "hook", alias = "command", default)]
    pub hooks: HookList,
}

/// Global typewriter configuration options.
///
/// Can only be used by the root typewriter
/// configuration file referenced in commands
/// in order to keep tracking configuration simple
#[derive(Deserialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    // Configuration options relating to
    // the apply command.
    #[serde(default)]
    pub apply: Apply,

    // Configuration options relating to
    // the preprocessor/variables
    #[serde(default)]
    pub variables: VariableConfig,

    // Configuration options relating to
    // commands ran in shell in configs/vars etc.
    #[serde(default)]
    pub commands: CommandConfig,

    // Configuration options relating to hooks
    // for running commands
    #[serde(default)]
    pub hooks: HooksConfig,
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
    /// into their useful data as lists.
    pub fn flatten_data(self: Self) -> (TrackedFileList, VariableList, HookList) {
        // Decompose each config and collect files and variables separately
        let (files, variables, hooks): (Vec<_>, Vec<_>, Vec<_>) = unzip3(
            self.0
                .into_iter()
                .map(|config| (config.files, config.variables, config.hooks)),
        );

        (
            // Flatten into inner values
            files.into_iter().flat_map(|f| f.0).collect(),
            variables.into_iter().flat_map(|v| v.0).collect(),
            hooks.into_iter().flat_map(|h| h.0).collect(),
        )
    }
}

/// Helper function that does a unzip on three dimensions
fn unzip3<A, B, C>(iter: impl Iterator<Item = (A, B, C)>) -> (Vec<A>, Vec<B>, Vec<C>) {
    let mut a_vec = Vec::new();
    let mut b_vec = Vec::new();
    let mut c_vec = Vec::new();
    for (a, b, c) in iter {
        a_vec.push(a);
        b_vec.push(b);
        c_vec.push(c);
    }
    (a_vec, b_vec, c_vec)
}

impl GlobalConfig {
    /// Set's the global config
    /// in the system to be this config
    pub fn set_config(self: &Self, global_config: Config) {
        ROOT_CONFIG.0.get_or_init(|| global_config);
    }

    /// Get's the root config
    /// or returns an error if it could not succesfully be gotten
    pub fn get_config(self: &Self) -> &'static Config {
        ROOT_CONFIG.0.wait()
    }
}
