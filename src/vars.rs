//! Variable file parsing/handling in typewriter

use std::{
    collections::{HashMap, HashSet},
    env,
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use anyhow::{Context, bail};
use regex::Regex;
use serde::{Deserialize, de};

use crate::{
    apply::variables::VariableApplyingStrategy,
    cleanpath::CleanPath,
    command::{CommandContext, execute_command},
    config::ROOT_CONFIG,
};

/// Helper list for interfacing with a list of variables
#[derive(Deserialize, Debug, Default)]
pub struct VariableList(pub Vec<Variable>);

/// Global variable related configuration options
/// (or preprocessor)
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct VariableConfig {
    // Variable format string to look for in the
    // files to replace entirely with the variable value
    //
    // Format specifiers:
    // {variable} - Name of the variable being replaced
    //
    // Example:
    // $TYPEWRITER{{variable}}
    #[serde(default = "default_variable_format")]
    pub variable_format: String,

    // Strategy to use for variable pre processing
    #[serde(default)]
    pub variable_strategy: VariableApplyingStrategy,
}

/// An individual "variable" which can be inserted
/// by the preprocessor of typewriter into config
/// files
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Variable {
    // Source file that contains this variable
    #[serde(skip)]
    pub src: PathBuf,

    // Name of this variable, this should be unique
    // Non-unique variables in a system will cause Error
    // and abort the operation.
    #[serde(deserialize_with = "deserialize_variable_name")]
    pub name: String,

    // Type of this variable, which
    // should be reflected in the value.
    #[serde(default, rename = "type")]
    pub var_type: VariableType,

    // Value which will be inserted in preprocess-time
    // into config files.
    pub value: String,
}

/// Types of variables supported
/// in typewriter
#[derive(Deserialize, Debug, Clone, Copy)]
pub enum VariableType {
    // Directly insert the value
    // as a string in all references to the variable
    #[serde(rename = "literal")]
    Literal,

    // Execute the value as a command before preprocess occurs,
    // and insert its output in the references to the variable
    #[serde(rename = "command")]
    Command,

    // Read in the value as an environment variable and insert
    // the environment variables value in all references to the variable.
    #[serde(rename = "environment")]
    Environment,
}

impl Default for VariableType {
    fn default() -> Self {
        Self::Literal
    }
}

impl Default for VariableConfig {
    fn default() -> Self {
        Self {
            variable_format: default_variable_format(),
            variable_strategy: Default::default(),
        }
    }
}

/// Defaults for the variable config.
fn default_variable_format() -> String {
    String::from("$TYPEWRITER{{variable}}")
}

/// Special deserialize for variable names to ensure
/// they're correct.
fn deserialize_variable_name<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: de::Deserializer<'de>,
{
    let name = String::deserialize(deserializer)?;

    // Validate variable name is valid.
    if name.is_empty() {
        return Err("Typewriter Variable name cannot be empty").map_err(de::Error::custom);
    }

    // Whitespace is not permitted.
    if name.contains(char::is_whitespace) {
        return Err("Typewriter Variable name cannot contain any whitespace")
            .map_err(de::Error::custom);
    }

    Ok(name)
}

impl Variable {
    /// Adds a supplied path to the path
    /// fields of the variable for keeping track
    /// of source file for debugging info
    pub fn add_typewriter_dir(self: &mut Self, file_path: &PathBuf) -> anyhow::Result<()> {
        // Absolutize the joined file path for both fields.
        self.src = file_path.clean_path()?;
        Ok(())
    }
}

impl Deref for VariableList {
    type Target = Vec<Variable>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VariableList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromIterator<Variable> for VariableList {
    fn from_iter<T: IntoIterator<Item = Variable>>(iter: T) -> Self {
        // Collect into wrapped form
        let iter_vec: Vec<Variable> = iter.into_iter().collect();
        VariableList(iter_vec)
    }
}

/// Executes a command using the ROOT_CONF variable shell
/// configs and waits for the stdin and returns it

/// Executes a command using the ROOT_CONF variable shell
/// configs and waits for the stdout and returns it
fn execute_command_conf_shell(
    var_name: &String,
    var_src: &PathBuf,
    command: &String,
) -> anyhow::Result<String> {
    // Set context of this command's execution
    let mut context = CommandContext::default();
    context.description = Some(format!(
        "for variable {} defined in configuration file {:?}",
        var_name, var_src
    ));

    context.workdir = Some(var_src.parent().with_context(
        || format!("Could not find parent directory for working directory of command execution for variable {} defined in configuration file {:?}",
            var_name, var_src
        )
    )?.to_path_buf());

    execute_command(command, &context)
}

/// Extracts variable references from a string based on the variable format
/// Returns a vector of variable names found
fn extract_variable_references(text: &str) -> anyhow::Result<Vec<String>> {
    let var_conf = &ROOT_CONFIG.get_config().variables;
    let format = &var_conf.variable_format;

    // Escape the format string and replace {variable} with a capture group
    let pattern = regex::escape(format).replace(r"\{variable\}", r"([^\s{}]+)");

    let re = Regex::new(&pattern).with_context(|| {
        format!(
            "While trying to make regex format {} for variable resolving",
            pattern
        )
    })?;

    Ok(re
        .captures_iter(text)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect())
}

/// Resolves variable references within a value string
/// Returns the resolved string with all variable references replaced
fn resolve_variable_references(value: &str, resolved_vars: &HashMap<String, String>) -> String {
    let var_conf = &ROOT_CONFIG.get_config().variables;
    let format = &var_conf.variable_format;

    let mut result = value.to_string();

    // Replace variables inside the string
    for (var_name, var_value) in resolved_vars {
        let placeholder = format.replace("{variable}", var_name);
        result = result.replace(&placeholder, var_value);
    }

    result
}

/// Returns the string-to-insert value of this variable
/// gotten from the type
/// Name & Src fields are for debugging info for the user.
fn get_true_value(
    var_name: &String,
    var_src: &PathBuf,
    var_type: VariableType,
    var_value: String,
) -> anyhow::Result<String> {
    match var_type {
        VariableType::Literal => Ok(var_value),
        VariableType::Command => execute_command_conf_shell(var_name, var_src, &var_value),
        VariableType::Environment => env::var(&var_value).with_context(|| {
            format!("While trying to get environment variable {} for variable {} defined in configuration file {:?}", var_value, var_name, var_src)
        }),
    }
}

/// Resolves a single variable, checking for circular dependencies
fn resolve_variable(
    var_name: &str,
    variables: &HashMap<String, Variable>,
    resolved: &mut HashMap<String, String>,
    resolving: &mut HashSet<String>,
) -> anyhow::Result<String> {
    // Check if already resolved
    if let Some(value) = resolved.get(var_name) {
        return Ok(value.clone());
    }

    // Check for circular dependency
    if resolving.contains(var_name) {
        let cycle: Vec<&str> = resolving.iter().map(|string| string.as_str()).collect();
        bail!(
            "Circular dependency detected in variable resolution: {} <-> {} (full chain: {:?})",
            cycle.join(" <-> "),
            var_name,
            cycle
        );
    }

    // Get the variable
    let variable = variables
        .get(var_name)
        .with_context(|| format!("Variable '{}' referenced but not defined", var_name))?;

    // Mark as currently resolving
    resolving.insert(var_name.to_string());

    // Extract references from the variable's value
    let references = extract_variable_references(&variable.value)?;

    // Recursively resolve all dependencies first
    for ref_name in &references {
        resolve_variable(ref_name, variables, resolved, resolving)?;
    }

    // Now resolve this variable's value with resolved dependencies
    let resolved_value = resolve_variable_references(&variable.value, resolved);

    // Get the true value (execute commands, read env vars, etc.)
    let final_value = get_true_value(
        &variable.name,
        &variable.src,
        variable.var_type,
        resolved_value,
    )?;

    // Remove from resolving set and add to resolved
    resolving.remove(var_name);
    resolved.insert(var_name.to_string(), final_value.clone());

    Ok(final_value)
}

impl VariableList {
    // Turns a list of variables and get's the final
    // value of each variable as the string-to-insert
    // into a map of the variable name to it's intended
    // final value.
    //
    // This can cause other programs to be executed, or
    // environment variables to be read e.g depending on
    // variable types so it can fail.
    //
    // Now supports nested variable references and detects
    // circular dependencies.
    pub fn to_map(self: Self) -> anyhow::Result<HashMap<String, String>> {
        // Build a map of variable names to Variable structs
        let mut var_map: HashMap<String, Variable> = HashMap::new();

        for variable in self.0 {
            // Check for duplicates
            if let Some(existing) = var_map.get(&variable.name) {
                bail!(
                    "Variable {} referenced in file {:?} was found to be already declared in file {:?}",
                    variable.name,
                    variable.src,
                    existing.src
                );
            }

            var_map.insert(variable.name.clone(), variable);
        }

        // Resolve all variables with dependency tracking
        let mut resolved: HashMap<String, String> = HashMap::new();
        let var_names: Vec<String> = var_map.keys().cloned().collect();

        for var_name in var_names {
            let mut resolving = HashSet::new();
            resolve_variable(&var_name, &var_map, &mut resolved, &mut resolving)?;
        }

        Ok(resolved)
    }
}
