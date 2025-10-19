//! Applies a configuration file
//! for a typewriter system and all
//! its referenced files to the currnet system

use ansi_term::Colour::Red;
use std::path::PathBuf;

use anyhow::bail;

use crate::parse_config::parse_config;

pub fn apply_command(file: String) -> anyhow::Result<()> {
    // Validate file path
    let path = PathBuf::from(file);

    if !path.exists() {
        bail!(
            "{} {} {}",
            Red.paint(format!(
                "Supplied file {:?} doesn't exist, maybe try running",
                path
            )),
            Red.underline().paint("init"),
            Red.paint("first")
        );
    }

    // Parse file in
    println!("{:?}", parse_config(path)?);

    Ok(())
}
