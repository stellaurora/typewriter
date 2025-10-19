use ansi_term::Colour::Yellow;

use crate::commands::init::init_command;

// Argument parsing from cli
mod args;

// Configuration parsing from toml
mod parse_config;

// Git integration
mod git;

// Different commands
mod commands;

fn main() {
    // Parse arguments from CLI
    let args = args::parse_args();
    println!(
        "typewriter running command: {}",
        Yellow.underline().paint(format!("{}", args.command))
    );

    // Run correct command handler.
    match args.command {
        args::Commands::Init { dir, file } => init_command(dir, file),
        args::Commands::Apply { file } => todo!(),
    }
}
