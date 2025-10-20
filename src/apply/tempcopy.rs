//! Responsible for managing the temporary copy component
//! of the application process

use std::{fs, path::PathBuf};

use anyhow::Context;
use log::info;
use path_absolutize::Absolutize;
use serde::Deserialize;

use crate::{config::ROOT_CONFIG, file::TrackedFile};

/// Which strategy should be used for the temporary
/// copy stage?
#[derive(Deserialize, Debug)]
pub enum TemporaryCopyStrategy {
    // Copy the current working file to the temporary directory
    // with the same name
    #[serde(rename = "copy_current")]
    CopyCurrent,

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
    println!("{:?}", ROOT_CONFIG.get_config().apply.temp_dir);

    // Make tempdir path for this file
    let mut tempcopy_path = PathBuf::from(ROOT_CONFIG.get_config().apply.temp_dir.absolutize()?);

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

pub fn run_temporary_copy_strategy(
    strategy: &TemporaryCopyStrategy,
    file: &TrackedFile,
) -> anyhow::Result<()> {
    // Different handler per strategy.
    match strategy {
        TemporaryCopyStrategy::CopyCurrent => copy_current_strategy(file)?,
        TemporaryCopyStrategy::Disabled => {}
    };

    Ok(())
}
