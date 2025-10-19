//! Parsing configuration file for typewriter

use serde::Deserialize;

use crate::git::Git;

/// Configuration for the a file in the typewriter system
///
/// config is not utilised outside of the root
/// file referenced directly by commands.
#[derive(Deserialize, Debug)]
pub struct Typewriter {
    // Global typewriter configuration options.
    config: Config,

    // Links to other files to include in the configuration
    #[serde(rename = "link")]
    links: Vec<ConfigLink>,
}

/// Global typewriter configuration options.
///
/// Can only be used by the root typewriter
/// configuration file referenced in commands
/// in order to keep tracking configuration simple
#[derive(Deserialize, Debug)]
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
pub struct ConfigLink {
    // The file that is being linked to
    file: String,
}
