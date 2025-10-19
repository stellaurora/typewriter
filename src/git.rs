//! Git integration with typewriter

use serde::Deserialize;

/// Configuration option for git-related
/// options under typewriter
#[derive(Deserialize, Debug)]
pub struct Git {
    // Whether or not on every typewriter ``apply`` command to create
    // a new commit of ``apply_commit_format`` format in the git repository
    // containing this root configuration file.
    #[serde(default)]
    apply_commit: bool,

    // The format of the message for apply_commit
    // Supports ``chrono`` format specifiers for now.
    #[serde(default)]
    apply_commit_format: GitCommitFormat,

    // Whether or not include what files were changed
    // in the commit message body
    #[serde(default)]
    apply_commit_changed: bool,
}

/// Git commit format for commits under
/// typewriter helper struct
///
/// Will be filled on commit with formatted
/// data.
#[derive(Deserialize, Debug)]
struct GitCommitFormat(String);

impl Default for GitCommitFormat {
    fn default() -> Self {
        // Default format should contain time and mention of typewriter apply
        // causing the commit.
        Self(String::from("feat: typewriter apply on %Y-%m-%d %H:%M:%S"))
    }
}
