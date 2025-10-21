//! Responsible for managing the temporary copy component
//! of the application process

use std::{fs, path::PathBuf};

use anyhow::Context;
use log::info;
use path_absolutize::Absolutize;
use serde::Deserialize;

use crate::{apply::strategy::ApplyStrategy, config::ROOT_CONFIG, file::TrackedFile};

/// Which strategy should be used for the temporary
/// copy stage?
#[derive(Deserialize, Debug)]
pub enum TemporaryCopyStrategy {
    // Copy the current working files to the temporary directory
    // with the same name while proceeding through the operation
    #[serde(rename = "copy_current")]
    CopyCurrent,

    // Dont do anything for this stage.. No temporary copying
    #[serde(rename = "disabled")]
    Disabled,
}

impl Default for TemporaryCopyStrategy {
    fn default() -> Self {
        Self::CopyCurrent
    }
}

pub fn rename_to_temp_copy(path: &PathBuf) -> String {
    path.to_string_lossy()
        .replace("/", &ROOT_CONFIG.get_config().apply.temp_copy_path_delim)
}

pub fn copy_current_strategy(file: &TrackedFile) -> anyhow::Result<()> {
    // Make tempdir path for this file
    let mut tempcopy_path = PathBuf::from(
        ROOT_CONFIG
            .get_config()
            .apply
            .apply_metadata_dir
            .absolutize()?,
    );

    fs::create_dir_all(&tempcopy_path)
        .with_context(|| "While trying to make temporary directory for copying")?;

    tempcopy_path.push(rename_to_temp_copy(&file.destination));

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

fn copy_current_strategy_cleanup(file: &TrackedFile) -> anyhow::Result<()> {
    // Path for this tempcopy.
    let mut tempcopy_path = PathBuf::from(
        ROOT_CONFIG
            .get_config()
            .apply
            .apply_metadata_dir
            .absolutize()?,
    );
    tempcopy_path.push(rename_to_temp_copy(&file.destination));
    fs::remove_file(&tempcopy_path)
        .with_context(|| "While trying to remove temporary copy of file in temporary directory")?;

    info!(
        "Deleted temporary copy of file {:?} in temporary directory with name {:?}",
        file.destination, tempcopy_path
    );

    Ok(())
}

impl ApplyStrategy for TemporaryCopyStrategy {
    fn run_before_copy_file(self: &Self, file: &mut TrackedFile) -> anyhow::Result<()> {
        match self {
            TemporaryCopyStrategy::CopyCurrent => copy_current_strategy(file),
            TemporaryCopyStrategy::Disabled => Ok(()),
        }
    }

    fn run_after_copy_file(self: &Self, file: &mut TrackedFile) -> anyhow::Result<()> {
        if !ROOT_CONFIG.get_config().apply.cleanup_files {
            return Ok(());
        }

        // Call cleanup function for copy strategy
        match self {
            TemporaryCopyStrategy::CopyCurrent => copy_current_strategy_cleanup(file),
            TemporaryCopyStrategy::Disabled => Ok(()),
        }
    }
}
