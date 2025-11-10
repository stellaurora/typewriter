//! Files managed under the typewriter system

use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use anyhow::Context;
use serde::Deserialize;

use crate::cleanpath::CleanPath;

/// List of tracked files with extra methods to help.
#[derive(Deserialize, Default, Debug)]
pub struct TrackedFileList(pub Vec<TrackedFile>);

/// File in typewriter config that should be tracked and updated
/// appropriately on apply.
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct TrackedFile {
    // Source file to read from
    pub file: PathBuf,

    // Allow checkdiff to skip this file
    // if the file == destination content?
    #[serde(default = "default_is_true")]
    pub skip_if_same_content: bool,

    // Destination location to write to
    pub destination: PathBuf,

    // Hooks that are executed before this file is applied
    #[serde(default)]
    pub pre_hook: Vec<String>,

    // Hooks that are executed after this file is applied
    #[serde(default)]
    pub post_hook: Vec<String>,

    // Whether or not to continue applying on an error in hooks
    #[serde(default)]
    pub continue_on_hook_error: bool,

    // Source configuration file for this tracked file
    #[serde(skip)]
    pub src: PathBuf,
}

fn default_is_true() -> bool {
    true
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
        self.file = parent.join(&self.file).clean_path()?;
        self.destination = parent.join(&self.destination).clean_path()?;
        self.src = file_path.clean_path()?;

        Ok(())
    }
}

impl Deref for TrackedFileList {
    type Target = Vec<TrackedFile>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TrackedFileList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromIterator<TrackedFile> for TrackedFileList {
    fn from_iter<T: IntoIterator<Item = TrackedFile>>(iter: T) -> Self {
        // Collect into wrapped form
        let iter_vec: Vec<TrackedFile> = iter.into_iter().collect();
        TrackedFileList(iter_vec)
    }
}
