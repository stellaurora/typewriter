//! Argument Parsing for typewriter using Clap

use std::fmt::Display;

use clap::{Parser, Subcommand};

// Root-arguments for typewriter
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Which operation to run with typewriter
    #[command(subcommand)]
    pub command: Commands,
}

// Enum for commands for different operations within typewriter
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialises a basic template file in the directory
    Init {
        /// Directory to create the template file in
        #[arg(short, long, default_value = ".")]
        dir: String,

        /// Name of the template file
        #[arg(short, long, default_value = "typewriter.toml")]
        file: String,
    },

    /// Applies the supplied typewriter configuration file to the system
    Apply {
        /// Name of the configuration file
        #[arg(short, long)]
        file: String,
    },
}

impl Display for Commands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Should display what type of command.
        match self {
            Commands::Init { .. } => write!(f, "init"),
            Commands::Apply { .. } => write!(f, "apply"),
        }
    }
}

/// Parses arguments to typewriter returning the
/// arguments, exits on error.
pub fn parse_args() -> Args {
    Args::parse()
}
