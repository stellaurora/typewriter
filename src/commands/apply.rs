//! Applies a configuration file
//! for a typewriter system and all
//! its referenced files to the currnet system

use path_absolutize::Absolutize;
use std::path::PathBuf;

use anyhow::bail;

use crate::parse_config::parse_config;

pub fn apply_command(file: String) -> anyhow::Result<()> {
    // Validate file path
    let path = PathBuf::from(PathBuf::from(file).absolutize()?);

    // Parse file in
    println!("{:?}", parse_config(path)?);

    Ok(())
}
