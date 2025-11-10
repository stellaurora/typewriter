//! Responsible for pre-processing copied files by inserting
//! typewriter variables in them.

use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Write},
};

use anyhow::{Context, bail};
use regex::Regex;
use serde::Deserialize;

use crate::{
    apply::strategy::ApplyStrategy,
    config::ROOT_CONFIG,
    file::{TrackedFile, TrackedFileList},
};

/// Which strategy to use for the variable preprocessing
/// stage?
#[derive(Deserialize, Debug, Clone, Copy)]
pub enum VariableApplyingStrategy {
    // Enabled, will preprocess and replace variables
    // found in file
    #[serde(rename = "replace_variables")]
    ReplaceVariables,

    // Dont preprocess
    #[serde(rename = "disabled")]
    Disabled,
}

/// Wrap the strategy with the variable map for processing
pub struct VariableApplying {
    // Which strategy to use for the pre processing
    strategy: VariableApplyingStrategy,

    // Map of variable name -> value for replacing
    var_map: HashMap<String, String>,
}

impl Default for VariableApplyingStrategy {
    fn default() -> Self {
        Self::ReplaceVariables
    }
}

impl VariableApplying {
    pub fn new(strategy: VariableApplyingStrategy, var_map: HashMap<String, String>) -> Self {
        Self { strategy, var_map }
    }
}

/// Returns the regex for matching to any variable
/// in the supplied the typewriter variable format.
fn get_variable_format_regex() -> anyhow::Result<Regex> {
    // Grab var format and escape it.
    let var_format = &ROOT_CONFIG.get_config().variables.variable_format;
    let escaped = regex::escape(&var_format);

    // Replace where {variable} would've gone with
    // regex match anything
    Ok(Regex::new(&escaped.replace("\\{variable\\}", "([^}]+)"))?)
}

impl VariableApplying {
    /// Checks the passed in files content
    /// contains only valid variables in the variable
    /// format supplied, else errors.
    fn check_file_variables_valid(self: &Self, file: &TrackedFile) -> anyhow::Result<()> {
        // Read in file using a buffered reader (dont exhaust memory on really-large files)
        let open_file = File::open(&file.file).with_context(|| format!(
            "While trying to read file {:?} referenced in configuration file {:?} to check for validity of variables",
            file.file, file.src))?;

        let reader = BufReader::new(open_file);

        // Regex for variable matching
        let variable_regex = get_variable_format_regex()?;

        // Process line by line
        for line in reader.lines() {
            let line = line?;

            // Find all matches in current line
            for capture in variable_regex.captures_iter(&line) {
                // capture[0] is the full match, capture[1] is the variable name
                let var_name = &capture[1];

                // Check if variable exists in var_map
                if self.var_map.contains_key(var_name) {
                    continue;
                }

                bail!(
                    "Variable {} found in file {:?} referenced in configuration file {:?} is undefined, aborting operation",
                    var_name,
                    file.file,
                    file.src
                );
            }
        }

        Ok(())
    }

    /// Replaces all of the variables found in the destination file of the provided file
    /// with the corresponding values found in the variable map.
    fn replace_file_variables(self: &Self, file: &TrackedFile) -> anyhow::Result<()> {
        // Read in file using a buffered reader
        let open_file = File::open(&file.file).with_context(|| {
            format!(
                "While trying to read file {:?} referenced in configuration file {:?} to replace variables",
                file.file, file.src
            )
        })?;

        // Open destination for writing to
        let mut destination_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&file.destination)
            .with_context(|| {
                format!(
                    "While trying to write to file {:?} referenced in configuration file {:?} to replace variables",
                    file.destination, file.src
                )
            })?;

        let reader = BufReader::new(open_file);

        // Regex for variable matching
        let variable_regex = get_variable_format_regex()?;

        // Process line by line
        for line in reader.lines() {
            let line = line?;

            // Replace all variables in this line
            let replaced_line = variable_regex.replace_all(&line, |caps: &regex::Captures| {
                let var_name = &caps[1];
                // We already validated all variables exist in check_file_variables_valid
                // so we can safely unwrap here unless some TOCTOU thing happened
                self.var_map.get(var_name).unwrap().as_str()
            });

            // Write the replaced line to temp file
            writeln!(destination_file, "{}", replaced_line)?;
        }

        Ok(())
    }
}

impl ApplyStrategy for VariableApplying {
    fn run_before_apply(self: &Self, files: &mut TrackedFileList) -> anyhow::Result<()> {
        match self.strategy {
            VariableApplyingStrategy::Disabled => return Ok(()),
            _ => {}
        }

        // Try validate all variables exist before running
        for file in files.iter() {
            self.check_file_variables_valid(file)?;
        }

        Ok(())
    }

    fn run_after_apply_file(self: &Self, file: &mut TrackedFile) -> anyhow::Result<()> {
        match self.strategy {
            VariableApplyingStrategy::Disabled => {
                // Copy file to destination directly, no variabling
                fs::copy(&file.file, &file.destination).with_context(|| {
                    format!(
                        "While trying to apply {:?} to {:?} referenced by config {:?}",
                        file.file, file.destination, file.src
                    )
                })?;

                Ok(())
            }
            _ => self.replace_file_variables(file),
        }
    }
}
