//! Hook management and execution for typewriter

use anyhow::{Context, Result, bail};
use log::{error, info, warn};
use serde::Deserialize;
use std::path::{Path, PathBuf};

use crate::{
    apply::strategy::ApplyStrategy,
    cleanpath::CleanPath,
    command::{CommandContext, execute_command},
    config::ROOT_CONFIG,
    file::{TrackedFile, TrackedFileList},
};

/// Hook execution stages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookStage {
    PreApply,
    PostApply,
}

/// Definition of a hook from configuration
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct HookDefinition {
    // The command to execute
    pub command: String,

    // What stage of the global apply process should this hook be ran in?
    pub stage: String,

    // Should the apply process continue even on an error in the command?
    #[serde(default)]
    pub continue_on_error: bool,

    // Source file tracking (added during parsing)
    #[serde(skip)]
    pub src: PathBuf,
}

/// Failure strategy for hooks
#[derive(Debug, Clone, Deserialize)]
pub enum FailureStrategy {
    // Stop entire apply on hook failure
    #[serde(rename = "abort")]
    Abort,

    // Log and continue
    #[serde(rename = "continue")]
    Continue,
}

impl Default for FailureStrategy {
    fn default() -> Self {
        Self::Abort
    }
}

/// Wrapper list for hooks
#[derive(Deserialize, Default, Debug)]
pub struct HookList(pub Vec<HookDefinition>);

impl std::ops::Deref for HookList {
    type Target = Vec<HookDefinition>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for HookList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromIterator<HookDefinition> for HookList {
    fn from_iter<T: IntoIterator<Item = HookDefinition>>(iter: T) -> Self {
        HookList(iter.into_iter().collect())
    }
}

/// Hook configuration options
#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct HooksConfig {
    // Whether or not hooks should be enabled in typewriter
    #[serde(default = "default_true")]
    pub hooks_enabled: bool,

    // Strategy to use on failure of hooks
    #[serde(default)]
    pub failure_strategy: FailureStrategy,
}

impl Default for HooksConfig {
    fn default() -> Self {
        Self {
            hooks_enabled: default_true(),
            failure_strategy: FailureStrategy::default(),
        }
    }
}

fn default_true() -> bool {
    true
}

impl HookDefinition {
    /// Add source file tracking and clean paths
    pub fn add_typewriter_dir(&mut self, file_path: &PathBuf) -> Result<()> {
        self.src = file_path.clean_path()?;
        Ok(())
    }

    /// Parse and validate stage string
    pub fn parse_stage(&self) -> Result<HookStage> {
        match self.stage.as_str() {
            "pre_apply" => Ok(HookStage::PreApply),
            "post_apply" => Ok(HookStage::PostApply),
            _ => bail!(
                "Invalid hook stage '{}' in {:?}. Must be 'pre_apply' or 'post_apply'",
                self.stage,
                self.src
            ),
        }
    }
}

/// Strategy wrapper for hooks integration with ApplyStrategy trait
pub struct HookStrategy {
    pre_apply_hooks: Vec<HookDefinition>,
    post_apply_hooks: Vec<HookDefinition>,
}

impl HookStrategy {
    pub fn new(hooks: HookList) -> Result<Self> {
        // Group hooks by stage, validating stages
        let mut pre_apply_hooks = Vec::new();
        let mut post_apply_hooks = Vec::new();

        for hook in hooks.0 {
            match hook.parse_stage()? {
                HookStage::PreApply => pre_apply_hooks.push(hook),
                HookStage::PostApply => post_apply_hooks.push(hook),
            }
        }

        Ok(Self {
            pre_apply_hooks,
            post_apply_hooks,
        })
    }

    /// Execute hooks for a specific stage
    fn execute_stage_hooks(&self, hooks: &[HookDefinition]) -> Result<()> {
        if !ROOT_CONFIG.get_config().hooks.hooks_enabled || hooks.is_empty() {
            return Ok(());
        }

        for hook in hooks {
            if let Err(e) = self.execute_hook(hook, None) {
                self.handle_hook_error(&hook.command, &hook.src, e, hook.continue_on_error)?;
            }
        }

        Ok(())
    }

    /// Execute a single hook
    fn execute_hook(
        &self,
        hook: &HookDefinition,
        file_context: Option<(&Path, &Path)>,
    ) -> Result<()> {
        let mut context = CommandContext::default();
        context.workdir = Some(hook.src.parent().with_context(
        || format!("Could not find parent directory for working directory of command execution for hook defined in configuration file {:?}",
            hook.src
        )
    )?.to_path_buf());
        context.description = Some(format!("from {:?}", hook.src));

        // Add file context environment variables if provided
        if let Some((src, dest)) = file_context {
            context.env_vars.push((
                "TYPEWRITER_FILE_SRC".to_string(),
                src.to_string_lossy().to_string(),
            ));
            context.env_vars.push((
                "TYPEWRITER_FILE_DEST".to_string(),
                dest.to_string_lossy().to_string(),
            ));
        }

        execute_command(&hook.command, &context)?;
        Ok(())
    }

    /// Execute a file-specific hook
    pub fn execute_file_hook(
        &self,
        command: &str,
        src: &Path,
        dest: &Path,
        src_config: &Path,
        continue_on_error: bool,
    ) -> Result<()> {
        if !ROOT_CONFIG.get_config().hooks.hooks_enabled {
            return Ok(());
        }

        let mut context = CommandContext::default();
        context.env_vars.push((
            "TYPEWRITER_FILE_SRC".to_string(),
            src.to_string_lossy().to_string(),
        ));
        context.env_vars.push((
            "TYPEWRITER_FILE_DEST".to_string(),
            dest.to_string_lossy().to_string(),
        ));
        context.description = Some(format!("file hook from {:?}", src_config));

        if let Err(e) = execute_command(command, &context) {
            self.handle_hook_error(command, src_config, e, continue_on_error)?;
        }

        Ok(())
    }

    /// Handle hook execution error based on strategy
    fn handle_hook_error(
        &self,
        command: &str,
        src: &Path,
        error: anyhow::Error,
        continue_on_error: bool,
    ) -> Result<()> {
        error!("Hook failed in {:?}: {}\nError: {:?}", src, command, error);

        // Per-hook override takes precedence
        if continue_on_error {
            warn!("Continuing despite hook failure (continue_on_error=true)");
            return Ok(());
        }

        // Use global strategy
        match ROOT_CONFIG.get_config().hooks.failure_strategy {
            FailureStrategy::Abort => {
                bail!("Aborting apply operation due to hook failure");
            }
            FailureStrategy::Continue => {
                warn!("Continuing despite hook failure");
                Ok(())
            }
        }
    }
}

impl ApplyStrategy for HookStrategy {
    fn run_before_apply(&self, _files: &mut TrackedFileList) -> Result<()> {
        info!(
            "Executing pre_apply hooks ({} hooks)",
            self.pre_apply_hooks.len()
        );
        self.execute_stage_hooks(&self.pre_apply_hooks)
    }

    fn run_before_apply_file(&self, file: &mut TrackedFile) -> Result<()> {
        // Execute file's pre_hook if it exists
        for pre_hook in &file.pre_hook {
            self.execute_file_hook(
                pre_hook,
                &file.file,
                &file.destination,
                &file.src,
                file.continue_on_hook_error,
            )?;
        }
        Ok(())
    }

    fn run_after_apply_file(&self, file: &mut TrackedFile) -> Result<()> {
        // Execute file's post_hook if it exists
        for post_hook in &file.post_hook {
            self.execute_file_hook(
                post_hook,
                &file.file,
                &file.destination,
                &file.src,
                file.continue_on_hook_error,
            )?;
        }
        Ok(())
    }

    fn run_after_apply(&self, _files: &mut TrackedFileList) -> Result<()> {
        info!(
            "Executing post_apply hooks ({} hooks)",
            self.post_apply_hooks.len()
        );
        self.execute_stage_hooks(&self.post_apply_hooks)
    }
}
