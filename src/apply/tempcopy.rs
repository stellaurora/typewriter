//! Responsible for managing the temporary copy component
//! of the application process

use std::{fs, path::PathBuf};

use anyhow::Context;
use log::info;
use serde::Deserialize;

use crate::{
    apply::strategy::ApplyStrategy,
    cleanpath::CleanPath,
    config::ROOT_CONFIG,
    file::{TrackedFile, TrackedFileList},
};

/// Which strategy should be used for the temporary
/// copy stage?
#[derive(Deserialize, Debug)]
pub enum TemporaryCopyStrategy {
    // Copy all destination files to the temporary directory
    // for backup before proceeding with the operation
    #[serde(rename = "copy_all")]
    CopyAll,

    // Dont do anything for this stage.. No temporary copying
    #[serde(rename = "disabled")]
    Disabled,
}

impl Default for TemporaryCopyStrategy {
    fn default() -> Self {
        Self::CopyAll
    }
}

pub fn rename_to_temp_copy(path: &PathBuf) -> String {
    path.to_string_lossy()
        .replace("/", &ROOT_CONFIG.get_config().apply.temp_copy_path_delim)
}

pub fn copy_all_strategy(file: &TrackedFile) -> anyhow::Result<()> {
    // Make tempdir path for this file
    let mut tempcopy_path = ROOT_CONFIG
        .get_config()
        .apply
        .apply_metadata_dir
        .clean_path()?;

    fs::create_dir_all(&tempcopy_path)
        .with_context(|| "While trying to make temporary directory for copying")?;

    tempcopy_path.push(rename_to_temp_copy(&file.destination));

    // Only backup if destination exists
    if !file.destination.exists() {
        info!(
            "Skipping backup of {:?} as it does not exist yet",
            file.destination
        );
        return Ok(());
    }

    // Temporary copy file name.
    fs::copy(&file.destination, &tempcopy_path)
        .with_context(|| "While trying to copy file to temporary directory")?;

    // Should be successful?
    info!(
        "Copied file {:?} to temporary copy {:?} for backup",
        file.destination, tempcopy_path
    );

    Ok(())
}

fn copy_all_strategy_cleanup(file: &TrackedFile) -> anyhow::Result<()> {
    // Path for this tempcopy.
    let mut tempcopy_path = ROOT_CONFIG
        .get_config()
        .apply
        .apply_metadata_dir
        .clean_path()?;

    tempcopy_path.push(rename_to_temp_copy(&file.destination));
    fs::remove_file(&tempcopy_path)
        .with_context(|| "While trying to remove temporary copy of file in temporary directory")?;

    info!(
        "Deleted temporary copy of file {:?} in temporary directory with name {:?}",
        file.destination, tempcopy_path
    );

    Ok(())
}

fn restore_from_temp_copy(file: &TrackedFile) -> anyhow::Result<()> {
    let mut tempcopy_path = ROOT_CONFIG
        .get_config()
        .apply
        .apply_metadata_dir
        .clean_path()?;

    tempcopy_path.push(rename_to_temp_copy(&file.destination));

    if !tempcopy_path.exists() {
        info!(
            "No backup found for {:?}, skipping restore",
            file.destination
        );
        return Ok(());
    }

    // Restore the backup
    fs::copy(&tempcopy_path, &file.destination).with_context(|| {
        format!(
            "While trying to restore file {:?} from temporary copy {:?}",
            file.destination, tempcopy_path
        )
    })?;

    info!(
        "Restored file {:?} from temporary copy {:?}",
        file.destination, tempcopy_path
    );

    Ok(())
}

fn restore_all_from_temp_copies(files: &TrackedFileList) -> anyhow::Result<()> {
    let mut restore_errors = Vec::new();
    let mut restore_count = 0;

    for file in files.iter() {
        match restore_from_temp_copy(file) {
            Ok(_) => {
                if get_temp_copy_path(&file.destination)?.exists() {
                    restore_count += 1;
                }
            }
            Err(e) => {
                log::error!(
                    "Failed to restore file {:?} from backup: {:?}",
                    file.destination,
                    e
                );
                restore_errors.push((&file.destination, e));
            }
        }
    }

    if restore_count > 0 {
        log::warn!("Rolled back {} file(s) to previous state", restore_count);
    }

    if !restore_errors.is_empty() {
        log::error!(
            "Failed to restore {} file(s) during rollback",
            restore_errors.len()
        );
    }

    Ok(())
}

fn get_temp_copy_path(destination: &PathBuf) -> anyhow::Result<PathBuf> {
    let mut tempcopy_path = ROOT_CONFIG
        .get_config()
        .apply
        .apply_metadata_dir
        .clean_path()?;

    tempcopy_path.push(rename_to_temp_copy(destination));
    Ok(tempcopy_path)
}

impl ApplyStrategy for TemporaryCopyStrategy {
    fn run_before_apply_file(self: &Self, file: &mut TrackedFile) -> anyhow::Result<()> {
        match self {
            TemporaryCopyStrategy::CopyAll => copy_all_strategy(file),
            TemporaryCopyStrategy::Disabled => Ok(()),
        }
    }

    fn run_after_apply(self: &Self, files: &mut TrackedFileList) -> anyhow::Result<()> {
        if !ROOT_CONFIG.get_config().apply.cleanup_files {
            return Ok(());
        }

        // Cleanup all temporary backups after successful apply
        match self {
            TemporaryCopyStrategy::CopyAll => {
                for file in files.iter() {
                    if let Err(e) = copy_all_strategy_cleanup(file) {
                        log::warn!(
                            "Failed to cleanup temporary backup for {:?}: {:?}",
                            file.destination,
                            e
                        );
                    }
                }
                Ok(())
            }
            TemporaryCopyStrategy::Disabled => Ok(()),
        }
    }

    fn run_on_failure(self: &Self, files: &mut TrackedFileList) -> anyhow::Result<()> {
        match self {
            TemporaryCopyStrategy::CopyAll => {
                log::warn!("Apply operation failed, attempting to restore all files from backup");
                restore_all_from_temp_copies(files)
            }
            TemporaryCopyStrategy::Disabled => Ok(()),
        }
    }
}
