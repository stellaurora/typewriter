//! Applies a configuration file
//! for a typewriter system and all
//! its referenced files to the currnet system

use anyhow::bail;
use inquire::Confirm;
use log::info;
use path_absolutize::Absolutize;
use std::path::PathBuf;

use crate::{apply::apply, config::ROOT_CONFIG, file::TrackedFileList, parse_config::parse_config};

/// Questions the user whether or not to continue the apply based on
/// the configuration
fn continue_apply_prompt(num_applications: usize) -> anyhow::Result<bool> {
    if !ROOT_CONFIG.get_config().apply.confirm_apply {
        info!("Running {} apply operations", num_applications);
        return Ok(true);
    }

    Ok(
        Confirm::new(format!("Run {} apply operations?", num_applications).as_str())
            .with_default(true)
            .prompt()?,
    )
}

pub fn apply_command(file: String) -> anyhow::Result<()> {
    // Validate file path
    let path = PathBuf::from(PathBuf::from(file).absolutize()?);

    // Parse configs to config structs.
    let (root, configs) = parse_config(path)?;

    // Fill in global root config from root
    let global_config = root.config.unwrap_or_default();
    ROOT_CONFIG.set_config(global_config);

    // Combile all files in all typewriters into one list
    let mut total_list: TrackedFileList = root.files;
    total_list.extend(configs.all_files().0.into_iter());

    // Skip all the files with no permissions
    // TOCTOU, ensure can still handle case with no permissions later.
    total_list = total_list.question_skip_files_no_perms()?;

    // Nothing to apply to case.
    if total_list.len() < 1 {
        info!("No files referenced to apply to, operation complete.");
        return Ok(());
    }

    if !continue_apply_prompt(total_list.len())? {
        bail!("Aborting apply operation");
    }

    // Run apply
    apply(total_list)
}
