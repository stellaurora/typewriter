//! Strategy trait for unified strategy-handling

use crate::file::{TrackedFile, TrackedFileList};

/// Strategy which can be run at multiple stages of the apply stage
pub trait ApplyStrategy {
    /// This strategy will have this ran
    /// before the overall copy
    fn run_before_copy(self: &Self, files: &mut TrackedFileList) -> anyhow::Result<()> {
        let _ = files;
        Ok(())
    }

    /// This strategy will be ran before an individual
    /// file is copied
    fn run_before_copy_file(self: &Self, file: &mut TrackedFile) -> anyhow::Result<()> {
        let _ = file;
        Ok(())
    }

    /// This strategy will be ran after a the file is copied
    fn run_after_copy_file(self: &Self, file: &mut TrackedFile) -> anyhow::Result<()> {
        let _ = file;
        Ok(())
    }

    /// This strategy will be run after all files are copied.
    fn run_after_copy(self: &Self, files: &mut TrackedFileList) -> anyhow::Result<()> {
        let _ = files;
        Ok(())
    }
}
