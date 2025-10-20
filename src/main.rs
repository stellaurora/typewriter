use ::log::error;

use crate::{
    commands::{apply, init},
    log::setup_logging,
};

// Argument parsing from cli
mod args;

// Configuration parsing from toml
mod parse_config;

// Git integration
mod git;

// File management
mod file;

// Different commands
mod commands;

// Logging handling/setup
mod log;

fn main() {
    setup_logging();

    // Parse arguments from CLI
    let args = args::parse_args();
    println!("typewriter running command: {}", args.command);

    // Run correct command handler.
    let command_result = match args.command {
        args::Commands::Init { file } => init::init_command(file),
        args::Commands::Apply { file } => apply::apply_command(file),
    };

    // Use error logger to print error..
    let _ = command_result.inspect_err(|err| {
        error!("{:?}", err);
    });
}
