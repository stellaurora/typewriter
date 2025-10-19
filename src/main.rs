use ansi_term::Colour::White;

use crate::commands::{apply, init};

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

fn main() -> anyhow::Result<()> {
    // Parse arguments from CLI
    let args = args::parse_args();
    println!(
        "typewriter running command: {}\n",
        White.underline().paint(format!("{}", args.command))
    );

    // Run correct command handler.
    match args.command {
        args::Commands::Init { file } => init::init_command(file),
        args::Commands::Apply { file } => apply::apply_command(file),
    }
}
