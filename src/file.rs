//! Files managed under the typewriter system

use std::path::PathBuf;

use anyhow::Context;
use path_absolutize::Absolutize;
use serde::Deserialize;

/// File in typewriter config that should be tracked and updated
/// appropriately on apply.
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct TrackedFile {
    // Source file to read from
    file: PathBuf,

    // Destination location to write to
    destination: PathBuf,
}

impl TrackedFile {
    /// Adds a supplied path to the path
    /// fields of the tracked file to make it relative
    /// to the supplied path
    pub fn add_typewriter_dir(self: &mut Self, file_path: &PathBuf) -> anyhow::Result<()> {
        // Parent of the file path passed din
        let parent = file_path
            .parent()
            .context("Configuration file has no parent directory")?;

        // Absolutize the joined file path for both fields.
        self.file = PathBuf::from(parent.join(&self.file).absolutize()?);
        self.destination = PathBuf::from(parent.join(&self.destination).absolutize()?);
        Ok(())
    }
}
