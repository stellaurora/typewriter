use ::log::{debug, error};

use crate::{commands::init, log::setup_logging};

// Argument parsing from cli
mod args;

// Configuration parsing from toml
mod config;
mod parse_config;

// Variables handling in the config files
mod vars;

// File management
mod file;

// Different commands
mod commands;

// Logging handling/setup
mod log;

// Applying operation
mod apply;

fn main() {
    setup_logging();

    // Parse arguments from CLI
    let args = args::parse_args();
    debug!("typewriter running command: {}", args.command);

    // Run correct command handler.
    let command_result = match args.command {
        args::Commands::Init { file } => init::init_command(file),
        args::Commands::Apply { file } => commands::apply::apply_command(file),
    };

    // Use error logger to print error..
    let _ = command_result.inspect_err(|err| {
        error!("{:?}", err);
    });
}
