//! Initialises a typewriter system
//! with a basic configuration file

use anyhow::bail;
use inquire::Confirm;
use std::{fs, path::PathBuf};

/// Default file just include it as a str..
const DEFAULT_TEMPLATE: &'static str = include_str!("../default.toml");

pub fn init_command(file: String) -> anyhow::Result<()> {
    // Path to the file
    let path = PathBuf::from(file);

    // Whether or not we should generate the output file
    // set to false to disable at the end
    let mut generate_output = true;

    // File already exists, prompt user
    if path.exists() {
        let exists_prompt =
            Confirm::new("Supplied template path already exists, overwrite this file?")
                .with_default(false)
                .prompt();

        generate_output = match exists_prompt {
            Ok(should_overwrite) => should_overwrite,
            Err(error) => bail!(error),
        }
    }

    if !generate_output {
        bail!("Not generating template to {:?}, file already exists", path);
    }

    // Write default template
    fs::write(&path, DEFAULT_TEMPLATE)?;
    println!("Wrote default template file to {:?}", path);

    Ok(())
}
