use std::path::PathBuf;

use serde::Deserialize;

use crate::{
    apply::tempcopy::{TemporaryCopyStrategy, run_temporary_copy_strategy},
    config::ROOT_CONFIG,
    file::TrackedFileList,
};

// Temporary copy handling
pub mod tempcopy;

/// Configuration options to apply command
/// files
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Apply {
    // Whether or not to automatically
    // skip files which cannot be properly
    // applied using typewriter
    #[serde(default)]
    pub auto_skip_unable_apply: bool,

    // Whether or not to skip the confirmation
    // prompt for the apply command
    #[serde(default = "default_is_true")]
    pub confirm_apply: bool,

    // Directory to place temporary files in
    // this is relative to the root config file.
    #[serde(default = "default_tempfile_dir")]
    pub temp_dir: PathBuf,

    // Strategy for temporary copy
    // Will only be used if disable_temporary_copy
    // is not set to true
    #[serde(default)]
    pub temp_copy_strategy: TemporaryCopyStrategy,

    // Delimiter for temp copy file names
    // to replace path seperator with
    #[serde(default = "default_temp_copy_path_delim")]
    pub temp_copy_path_delim: String,

    // Should we clean up temporary copy files at
    // the end of the apply if it succeeded?
    #[serde(default = "default_is_true")]
    pub cleanup_files: bool,
}

/// I think we have to sadly re-duplicate serde default here
/// for if the struct itself is missing.
impl Default for Apply {
    fn default() -> Self {
        Self {
            auto_skip_unable_apply: Default::default(),
            confirm_apply: default_is_true(),
            temp_dir: default_tempfile_dir(),
            temp_copy_strategy: Default::default(),
            temp_copy_path_delim: default_temp_copy_path_delim(),
            cleanup_files: default_is_true(),
        }
    }
}

fn default_is_true() -> bool {
    true
}

/// Default delimiter for directory path in tempcopy file names
fn default_temp_copy_path_delim() -> String {
    String::from("-")
}

/// Default directory for tempfiles
fn default_tempfile_dir() -> PathBuf {
    PathBuf::from(".typewriter")
}

/// Run apply copy
pub fn apply(file_list: TrackedFileList) -> anyhow::Result<()> {
    // Sequential order.. Pray nothing goes wrong :prayge:
    for file in file_list.0 {
        // Step 1: Temporary copy for backup <- hopefully saves
        // stuff if things do go wrong/power off etc.
        run_temporary_copy_strategy(&ROOT_CONFIG.get_config().apply.temp_copy_strategy, &file)?;
    }

    Ok(())
}
