use crate::Issue;
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Core plugin trait. All Ship plugins implement this.
///
/// Default implementations are no-ops so plugins only override what they care about.
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;

    /// Called after an issue is successfully created.
    fn on_issue_created(&self, _project_dir: &Path, _issue: &Issue) -> Result<()> {
        Ok(())
    }

    /// Called after an issue is moved to a new status.
    fn on_issue_moved(
        &self,
        _project_dir: &Path,
        _issue: &Issue,
        _from: &str,
        _to: &str,
    ) -> Result<()> {
        Ok(())
    }

    /// Called after an issue is deleted.
    fn on_issue_deleted(&self, _project_dir: &Path, _issue_name: &str) -> Result<()> {
        Ok(())
    }

    /// Returns the directory where this plugin stores its data.
    fn plugin_dir(&self, project_dir: &Path) -> PathBuf {
        project_dir.join("plugins").join(self.name())
    }
}

/// Holds all registered plugins and dispatches lifecycle events to them.
#[derive(Default)]
pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn plugins(&self) -> &[Box<dyn Plugin>] {
        &self.plugins
    }

    pub fn on_issue_created(&self, project_dir: &Path, issue: &Issue) {
        for plugin in &self.plugins {
            if let Err(e) = plugin.on_issue_created(project_dir, issue) {
                eprintln!("[ship:{}] on_issue_created: {}", plugin.name(), e);
            }
        }
    }

    pub fn on_issue_moved(&self, project_dir: &Path, issue: &Issue, from: &str, to: &str) {
        for plugin in &self.plugins {
            if let Err(e) = plugin.on_issue_moved(project_dir, issue, from, to) {
                eprintln!("[ship:{}] on_issue_moved: {}", plugin.name(), e);
            }
        }
    }

    pub fn on_issue_deleted(&self, project_dir: &Path, issue_name: &str) {
        for plugin in &self.plugins {
            if let Err(e) = plugin.on_issue_deleted(project_dir, issue_name) {
                eprintln!("[ship:{}] on_issue_deleted: {}", plugin.name(), e);
            }
        }
    }
}
