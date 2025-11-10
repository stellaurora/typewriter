//! Strategy responsible for checking file permissions
//! and optionally creating missing destination files
//! before the apply operation proceeds.

use std::{
    cell::RefCell,
    collections::HashSet,
    fs::{self, File, OpenOptions},
    path::PathBuf,
};

use anyhow::{Context, bail};
use inquire::Confirm;
use log::{error, info};
use serde::Deserialize;

use crate::{
    apply::strategy::ApplyStrategy,
    config::ROOT_CONFIG,
    file::{TrackedFile, TrackedFileList},
};

/// Strategy for checking file permissions and
/// optionally creating missing destination files
#[derive(Deserialize, Debug)]
pub enum FilePermissionStrategy {
    // Only check file permissions, do not create missing files
    #[serde(rename = "check_only")]
    CheckOnly,

    // Create destination files if missing
    #[serde(rename = "create_if_missing")]
    CreateIfMissing,

    // Disable file permission checking entirely
    #[serde(rename = "disabled")]
    Disabled,
}

impl Default for FilePermissionStrategy {
    fn default() -> Self {
        Self::CheckOnly
    }
}

// Track created files for potential cleanup on failure, this
// is thread_local because static declarations need to be Sync
// but we are only using it in a single thread context anyway.
thread_local! {
    static CREATED_FILES: RefCell<Option<HashSet<PathBuf>>> = RefCell::new(None);
}

impl FilePermissionStrategy {
    /// Check permissions for a single file path with given options.
    /// Returns Ok(()) if accessible, otherwise prompts user or errors based on config.
    fn check_path_access(
        path: &PathBuf,
        config_src: &PathBuf,
        perms: OpenOptions,
        access_type: &str,
    ) -> anyhow::Result<()> {
        if let Err(err) = perms.open(path).with_context(|| {
            format!(
                "While checking {} access to file {:?} referenced in configuration file {:?}",
                access_type, path, config_src
            )
        }) {
            error!("{:?}", err);

            if ROOT_CONFIG.get_config().apply.auto_skip_unable_apply {
                bail!("Cannot {} file {:?}", access_type, path);
            }

            let to_skip = Confirm::new(
                format!(
                    "Cannot access file {:?} referenced in configuration file {:?}, abort?",
                    path, config_src
                )
                .as_str(),
            )
            .with_default(true)
            .prompt()?;

            if to_skip {
                bail!("Aborted due to file access error");
            }
        }

        Ok(())
    }

    /// Creates a destination file and tracks it for cleanup on failure.
    ///
    /// Prompts user for confirmation if auto_confirm_file_creation is false.
    /// Creates parent directories if they don't exist.
    fn create_destination_file(file: &TrackedFile) -> anyhow::Result<()> {
        // Prompt user if not auto-confirming
        if !ROOT_CONFIG.get_config().apply.auto_confirm_file_creation {
            let to_create = Confirm::new(
                format!(
                    "Destination file {:?} does not exist. Create it?",
                    file.destination
                )
                .as_str(),
            )
            .with_default(true)
            .prompt()?;

            if !to_create {
                bail!(
                    "Aborted: User declined to create file {:?}",
                    file.destination
                );
            }
        }

        // Create parent directories if needed
        if let Some(parent) = file.destination.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "While creating parent directories for destination file {:?}",
                    file.destination
                )
            })?;
        }

        // Create the file
        File::create(&file.destination).with_context(|| {
            format!(
                "While creating destination file {:?} referenced in configuration file {:?}",
                file.destination, file.src
            )
        })?;

        // Track created file for cleanup on failure
        CREATED_FILES.with(|created| {
            if let Some(ref mut set) = *created.borrow_mut() {
                set.insert(file.destination.clone());
            }
        });

        info!(
            "Created destination file {:?} for source {:?}",
            file.destination, file.file
        );

        Ok(())
    }

    /// Validates file permissions and optionally creates missing files.
    ///
    /// Checks that source file is readable and destination file is writable.
    /// If create_missing is true and destination doesn't exist, creates it
    /// and tracks it for potential cleanup on failure.
    fn check_file_perms(&self, file: &TrackedFile, create_missing: bool) -> anyhow::Result<()> {
        // Check source file read access
        let mut src_options = OpenOptions::new();
        src_options.read(true);
        Self::check_path_access(&file.file, &file.src, src_options, "read")?;

        // Check destination file existence and create if needed
        let dest_exists = file.destination.exists();
        if !dest_exists && create_missing {
            Self::create_destination_file(file)?;
            return Ok(());
        }

        // Check destination file write access
        let mut dest_options = OpenOptions::new();
        dest_options.write(true).read(true);
        Self::check_path_access(&file.destination, &file.src, dest_options, "write to")?;

        Ok(())
    }
}

impl ApplyStrategy for FilePermissionStrategy {
    fn run_before_apply(&self, files: &mut TrackedFileList) -> anyhow::Result<()> {
        // Initialize created files tracking
        CREATED_FILES.with(|created| {
            *created.borrow_mut() = Some(HashSet::new());
        });

        match self {
            FilePermissionStrategy::Disabled => Ok(()),
            FilePermissionStrategy::CheckOnly => {
                for file in files.iter() {
                    self.check_file_perms(file, false)?;
                }
                Ok(())
            }
            FilePermissionStrategy::CreateIfMissing => {
                for file in files.iter() {
                    self.check_file_perms(file, true)?;
                }
                Ok(())
            }
        }
    }

    fn run_on_failure(&self, _files: &mut TrackedFileList) -> anyhow::Result<()> {
        // Cleanup created files on failure
        CREATED_FILES.with(|created| {
            if let Some(ref mut set) = *created.borrow_mut() {
                if !set.is_empty() {
                    log::warn!(
                        "Cleaning up {} file(s) that were created during failed apply",
                        set.len()
                    );
                    for path in set.iter() {
                        // Attempt to remove the created file
                        if let Err(e) = fs::remove_file(path) {
                            log::error!("Failed to remove created file {:?}: {:?}", path, e);
                        } else {
                            info!("Removed created file {:?}", path);
                        }
                    }
                }
                set.clear();
            }
            *created.borrow_mut() = None;
        });
        Ok(())
    }

    fn run_after_apply(&self, _files: &mut TrackedFileList) -> anyhow::Result<()> {
        // Clear created files tracking after successful apply
        CREATED_FILES.with(|created| {
            *created.borrow_mut() = None;
        });
        Ok(())
    }
}
