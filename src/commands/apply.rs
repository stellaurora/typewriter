//! Applies a configuration file
//! for a typewriter system and all
//! its referenced files to the currnet system

use anyhow::bail;
use inquire::Confirm;
use log::info;
use std::path::PathBuf;

use crate::{
    apply::{apply, hooks::HookStrategy, strategy::ApplyStrategy, variables::VariableApplying},
    cleanpath::CleanPath,
    config::ROOT_CONFIG,
    parse_config::parse_config,
};

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

pub fn apply_command(file: String, section: String) -> anyhow::Result<()> {
    // Validate file path
    let path = PathBuf::from(file).clean_path()?;

    // Parse configs to config structs.
    let (root, configs) = parse_config(path, section)?;

    // Fill in global root config from root
    let global_config = root.config.unwrap_or_default();
    ROOT_CONFIG.set_config(global_config);

    let config = ROOT_CONFIG.get_config();

    // Grab data flattened into a list
    let (mut total_files_list, mut total_variables_list, mut total_hooks_list) =
        configs.flatten_data();
    total_files_list.extend(root.files.0.into_iter());
    total_variables_list.extend(root.variables.0.into_iter());
    total_hooks_list.extend(root.hooks.0.into_iter());

    // Deal with variables first
    let var_map = total_variables_list.to_map()?;
    let var_strategy = VariableApplying::new(config.variables.variable_strategy, var_map);

    // Create hook strategy
    let hook_strategy = HookStrategy::new(total_hooks_list)?;

    // Nothing to apply to case.
    if total_files_list.len() < 1 {
        info!("No files referenced to apply to, no operation.");
        return Ok(());
    }

    if !continue_apply_prompt(total_files_list.len())? {
        bail!("Aborting apply operation");
    }

    // ensure order is correct or bad things will happen !!
    let strategies: Vec<&dyn ApplyStrategy> = vec![
        &config.apply.file_permission_strategy,
        &var_strategy,
        &config.apply.checkdiff_strategy,
        &config.apply.temp_copy_strategy,
        &hook_strategy,
    ];

    // Run apply
    apply(total_files_list, strategies)
}
