//! Centralized command execution for typewriter
use anyhow::{Context, Result, bail};
use inquire::Confirm;
use log::info;
use serde::Deserialize;
use std::{
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{Command, Stdio},
    thread,
};

use crate::config::ROOT_CONFIG;

#[derive(Deserialize, Debug)]
pub struct CommandConfig {
    // Shell to run commands in
    #[serde(default = "default_shell")]
    pub shell: String,

    // Argument to provide to the shell to be capable
    // of running the commands
    #[serde(default = "default_shell_command_arg")]
    pub shell_command_arg: String,

    // Confirm on running any shell commands in the
    // config
    #[serde(default = "default_is_true")]
    pub confirm_shell_commands: bool,

    // Inherit stdin to allow user input into ran commands?
    #[serde(default = "default_is_true")]
    pub commands_inherit_stdin: bool,

    // Inherit stdout to allow printing to stdout from commands?
    #[serde(default = "default_is_true")]
    pub commands_inherit_stdout: bool,

    // Inherit stderr to allow printing to stderr from commands?
    #[serde(default = "default_is_true")]
    pub commands_inherit_stderr: bool,
}

impl Default for CommandConfig {
    fn default() -> Self {
        Self {
            shell: default_shell(),
            shell_command_arg: default_shell_command_arg(),
            confirm_shell_commands: default_is_true(),
            commands_inherit_stdin: default_is_true(),
            commands_inherit_stdout: default_is_true(),
            commands_inherit_stderr: default_is_true(),
        }
    }
}

/// Execute a command with optional confirmation, workdir, and environment variables
pub fn execute_command(command: &str, context: &CommandContext) -> Result<String> {
    // Config to pull command related options from
    let command_config = &ROOT_CONFIG.get_config().commands;

    // Confirmation prompt if enabled
    if command_config.confirm_shell_commands {
        let prompt_msg = match &context.description {
            Some(desc) => format!("Run command {} ({})?", command, desc),
            None => format!("Run command {}?", command),
        };
        let to_continue = Confirm::new(&prompt_msg).with_default(true).prompt()?;
        if !to_continue {
            bail!("Command execution cancelled by user");
        }
    }

    info!("Executing command: {}", command);

    // Build command
    let mut cmd = Command::new(&command_config.shell);
    cmd.arg(&command_config.shell_command_arg).arg(command);

    // Set working directory if specified
    if let Some(workdir) = &context.workdir {
        cmd.current_dir(workdir);
    }

    // Set environment variables
    for (key, value) in &context.env_vars {
        cmd.env(key, value);
    }

    // Inheriting stdin so commands can get user input until completion.
    if command_config.commands_inherit_stdin {
        cmd.stdin(Stdio::inherit());
    }

    // Spawn process with piped stdout and stderr -> since we want to
    // both "inherit" and "take"
    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("While spawning command: {}", command))?;

    // Capture and print stdout in a separate thread
    let stdout = child
        .stdout
        .take()
        .with_context(|| format!("Failed to capture stdout while running command {}", command))?;

    // Whether to "inherit" (display) stdout
    let display_stdout = command_config.commands_inherit_stdout;

    // Read from stdout into both the reader and to the actual stdout.
    let stdout_reader = BufReader::new(stdout);
    let stdout_handle = thread::spawn(move || {
        let mut output = String::new();
        for line in stdout_reader.lines() {
            if let Ok(line) = line {
                if display_stdout {
                    println!("{}", line);
                }
                output.push_str(&line);
                output.push('\n');
            }
        }
        output
    });

    // Capture and print stderr in a separate thread
    let stderr = child
        .stderr
        .take()
        .with_context(|| format!("Failed to capture stderr while running command {}", command))?;

    // Whether to "inherit" (display) stderr
    let display_stderr = command_config.commands_inherit_stderr;

    // Read from stderr into both the reader and to the actual stderr.
    let stderr_reader = BufReader::new(stderr);
    let stderr_handle = thread::spawn(move || {
        let mut output = String::new();
        for line in stderr_reader.lines() {
            if let Ok(line) = line {
                if display_stderr {
                    eprintln!("{}", line);
                }

                output.push_str(&line);
                output.push('\n');
            }
        }
        output
    });

    // Wait for the process to complete
    let status = child
        .wait()
        .with_context(|| format!("While waiting for command: {}", command))?;

    // Collect output from threads
    let stdout_output = stdout_handle.join().unwrap_or_default();
    let stderr_output = stderr_handle.join().unwrap_or_default();

    if !status.success() {
        bail!(
            "Command failed with exit code {:?}: {}\nStderr: {}",
            status.code(),
            command,
            stderr_output
        );
    }

    Ok(stdout_output)
}

/// Context for command execution
pub struct CommandContext {
    pub workdir: Option<PathBuf>,
    pub env_vars: Vec<(String, String)>,
    pub description: Option<String>,
}

impl Default for CommandContext {
    fn default() -> Self {
        Self {
            workdir: None,
            env_vars: Vec::new(),
            description: None,
        }
    }
}

/// Defaults for configuration options
fn default_shell_command_arg() -> String {
    String::from("-c")
}

fn default_shell() -> String {
    String::from("bash")
}

fn default_is_true() -> bool {
    true
}
