//! Variable file parsing/handling in typewriter

use std::{
    collections::HashMap,
    env,
    ops::{Deref, DerefMut},
    path::PathBuf,
    process::Command,
};

use anyhow::{Context, bail};
use inquire::Confirm;
use path_absolutize::Absolutize;
use serde::{Deserialize, de};

use crate::{apply::variables::VariableApplyingStrategy, config::ROOT_CONFIG};

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

    // Shell to run variables of type command in
    #[serde(default = "default_shell")]
    pub variable_shell: String,

    // Argument to provide to the shell to be capable
    // of running the variable command
    #[serde(default = "default_shell_command_arg")]
    pub shell_command_arg: String,

    // Confirm on running any shell commands in the
    // config
    #[serde(default = "default_is_true")]
    pub confirm_shell_commands: bool,
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
#[derive(Deserialize, Debug)]
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
            variable_shell: default_shell(),
            shell_command_arg: default_shell_command_arg(),
            confirm_shell_commands: default_is_true(),
            variable_strategy: Default::default(),
        }
    }
}

/// Defaults for the variable config.
fn default_shell_command_arg() -> String {
    String::from("-c")
}

fn default_is_true() -> bool {
    true
}

fn default_shell() -> String {
    String::from("bash")
}

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
        self.src = PathBuf::from(file_path.absolutize()?);
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
fn execute_command_conf_shell(
    var_name: &String,
    var_src: &PathBuf,
    command: &String,
) -> anyhow::Result<String> {
    // Config to pull shell/arg from
    let var_conf = &ROOT_CONFIG.get_config().variables;

    // Confirmation check
    if var_conf.confirm_shell_commands {
        let to_continue = Confirm::new(
            format!(
                "Run command {} for variable {} defined in configuration file {:?}?",
                command, var_name, var_src
            )
            .as_str(),
        )
        .with_default(false)
        .prompt()?;

        if !to_continue {
            bail!("Not running, aborting operation")
        }
    }

    // Run the command under that shell
    let output = Command::new(&var_conf.variable_shell)
        .arg(&var_conf.shell_command_arg)
        .arg(command)
        .output()
        .with_context(|| format!(
            "While trying to execute command {} to get value for variable {} defined in configuration file {:?}",
            command,
            var_name,
            var_src
        ))?;

    // Grab string and return
    Ok(String::from_utf8(output.stdout)
    .with_context(|| format!(
            "While trying to convert stdout to utf-8 String of command {} to get value for variable {} defined in configuration file {:?}",
            command,
            var_name,
            var_src
        ))?)
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

impl VariableList {
    // Turns a list of variables and get's the final
    // value of each variable as the string-to-insert
    // into a map of the variable name to it's intended
    // final value.
    //
    // This can cause other programs to be executed, or
    // environment variables to be read e.g depending on
    // variable types so it can fail.
    pub fn to_map(self: Self) -> anyhow::Result<HashMap<String, String>> {
        // For now, keep the original source as second element in tuple for tracing information
        let mut var_map: HashMap<String, (String, PathBuf)> = HashMap::new();

        for variable in self.0 {
            // Check for duplicates.
            if let Some((_, source)) = var_map.get(&variable.name) {
                bail!(
                    "Variable {} referenced in file {:?} was found to be already declared in file {:?}",
                    variable.name,
                    variable.src,
                    source
                );
            }

            // Get the true value from execution/env or whatever of the variable
            let value = get_true_value(
                &variable.name,
                &variable.src,
                variable.var_type,
                variable.value,
            )?;

            // No duplicate yay! Now try get this variables content.
            var_map.insert(variable.name, (value, variable.src));
        }

        // Drop all the now unecessary omfo since they've all passed/been executed successfully.
        // so hopefully tracing information now not required
        Ok(var_map
            .into_iter()
            .map(|(key, (value, _))| (key, value))
            .collect())
    }
}
