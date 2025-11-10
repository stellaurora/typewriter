//! Strategy trait for unified strategy-handling

use crate::file::{TrackedFile, TrackedFileList};

/// Strategy which can be run at multiple stages of the apply stage
pub trait ApplyStrategy {
    /// This strategy will have this ran
    /// before the overall copy
    fn run_before_apply(self: &Self, files: &mut TrackedFileList) -> anyhow::Result<()> {
        let _ = files;
        Ok(())
    }

    /// This strategy will be ran before an individual
    /// file is copied
    fn run_before_apply_file(self: &Self, file: &mut TrackedFile) -> anyhow::Result<()> {
        let _ = file;
        Ok(())
    }

    /// This strategy will be ran after a the file is copied
    fn run_after_apply_file(self: &Self, file: &mut TrackedFile) -> anyhow::Result<()> {
        let _ = file;
        Ok(())
    }

    /// This strategy will be run after all files are copied.
    fn run_after_apply(self: &Self, files: &mut TrackedFileList) -> anyhow::Result<()> {
        let _ = files;
        Ok(())
    }

    /// This strategy will be run if the apply operation fails
    /// to allow for cleanup or rollback
    fn run_on_failure(self: &Self, files: &mut TrackedFileList) -> anyhow::Result<()> {
        let _ = files;
        Ok(())
    }
}
