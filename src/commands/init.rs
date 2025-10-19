//! Initialises a typewriter system
//! with a basic configuration file

use std::path::PathBuf;

/// Default file just include it as a str..
const DEFAULT_TEMPLATE: &'static str = include_str!("../default.toml");

pub fn init_command(dir: String, file: String) {
    // Path to the file under the supplied directory
    let mut path = PathBuf::from(dir);
    path.push(file);
}
