//! Files managed under the typewriter system

use std::{
    fs::OpenOptions,
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use anyhow::{Context, bail};
use inquire::Confirm;
use log::{error, info};
use serde::Deserialize;

use crate::{cleanpath::CleanPath, config::ROOT_CONFIG};

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

    // Destination location to write to
    pub destination: PathBuf,

    // Source configuration file for this tracked file
    #[serde(skip)]
    pub src: PathBuf,
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

trait PermSkip {
    /// Questions if this file should be kept or not
    /// true if it should, else false
    ///
    /// SRC is the file which references this file for easier debugging by
    /// user
    fn question_keep_check_perms(
        self: &Self,
        src: &PathBuf,
        perms: OpenOptions,
    ) -> anyhow::Result<bool>;
}

impl PermSkip for PathBuf {
    fn question_keep_check_perms(
        self: &Self,
        src: &PathBuf,
        perms: OpenOptions,
    ) -> anyhow::Result<bool> {
        // Open file for reading for permission check.
        let read_result = match perms.open(&self).with_context(|| {
            format!(
                "While trying to check access of file {:?} referenced in configuration file {:?}",
                self, src
            )
        }) {
            Ok(_) => return Ok(true),
            Err(err) => err,
        };

        error!("{:?}", read_result);

        // Config opt support
        if ROOT_CONFIG.get_config().apply.auto_skip_unable_apply {
            info!(
                "Automatically skipping file {:?} referenced in configuration file {:?} due to auto_skip_unable_apply",
                self, self
            );

            return Ok(false);
        }

        // Question user whether to skip
        let to_skip = Confirm::new(
            format!(
                "Skip file {:?} referenced in configuration file {:?}?",
                self, src
            )
            .as_str(),
        )
        .with_default(true)
        .prompt()?;

        // Print if we skipped the file since the user would probably like to know.
        if to_skip {
            info!(
                "Skipped file {:?} referenced in configuration file {:?}",
                self, src
            );

            // Skip -> dont keep
            return Ok(false);
        }

        // Cancel operation, since we cant really keep going now.
        bail!(
            "Aborted apply operation due to lacking access to file {:?} referenced in configuration file {:?}",
            self,
            src
        );
    }
}

impl TrackedFile {
    /// Questions the user whether or not this file should be
    /// kept (also based on config) based on accessibility to file.
    pub fn question_keep_perms(self: &Self) -> anyhow::Result<bool> {
        // Permissions required  for source file
        let mut src_options = OpenOptions::new();
        src_options.read(true);

        // Permissions required for destination file
        let mut dest_options = OpenOptions::new();
        dest_options.write(true).read(true);

        // Check destination and source file perms.
        Ok(self
            .file
            .question_keep_check_perms(&self.src, src_options)?
            && self
                .destination
                .question_keep_check_perms(&self.src, dest_options)?)
    }
}

impl TrackedFileList {
    /// Question the user to skip files
    /// where the user does not have the ability
    /// to access the file properly with permissions
    pub fn question_skip_files_no_perms(self: Self) -> anyhow::Result<Self> {
        self.0
            .into_iter()
            .try_fold(Vec::new(), |mut acc, file| {
                // Check if we are skipping this file?
                let should_keep = file.question_keep_perms()?;

                if should_keep {
                    acc.push(file);
                }
                Ok(acc)
            })
            .map(|files| TrackedFileList(files))
    }
}
