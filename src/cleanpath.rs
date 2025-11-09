use anyhow::{Context, Result};
use path_absolutize::Absolutize;
use std::path::PathBuf;

/// Cleanup paths fully within the system of typewriter
/// should handle ., .., ~, etc.
pub trait CleanPath {
    fn clean_path(&self) -> Result<PathBuf>;
}

impl CleanPath for PathBuf {
    fn clean_path(&self) -> Result<PathBuf> {
        let path_str = self.to_string_lossy();

        // If the path contains a tilde (~), handle expansion.
        let expanded_path = if path_str.contains('~') {
            // Find the last tilde position to expand only the relevant suffix.
            if let Some(last_tilde_pos) = path_str.rfind('~') {
                let from_tilde = &path_str[last_tilde_pos..];
                let expanded = tilde_expand::tilde_expand(from_tilde.as_bytes());
                PathBuf::from(
                    String::from_utf8(expanded)
                        .with_context(|| format!("While trying to tilde expand path {:?}", self))?,
                )
            } else {
                PathBuf::from(&*path_str)
            }
        } else {
            // No tilde, use the original reference.
            self.as_path().to_path_buf()
        };

        // Convert to an absolute path.
        let absolute = expanded_path
            .absolutize()
            .with_context(|| format!("While trying to absolutize path {:?}", self))?;

        Ok(absolute.into_owned())
    }
}
