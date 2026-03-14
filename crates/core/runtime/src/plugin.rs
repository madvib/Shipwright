use anyhow::Result;
use std::path::{Path, PathBuf};

/// Core plugin trait. All Ship plugins implement this.
///
/// Default implementations are no-ops so plugins only override what they care about.
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;

    /// Returns the directory where this plugin stores its data.
    fn plugin_dir(&self, project_dir: &Path) -> PathBuf {
        project_dir.join(self.name())
    }

    /// Namespace claim used when registering plugin directories in `.ship/ship.toml`.
    fn namespace_claim(&self) -> crate::config::NamespaceConfig {
        crate::config::NamespaceConfig {
            id: format!("plugin:{}", self.name()),
            path: self.name().to_string(),
            owner: "plugins".to_string(),
        }
    }
}

/// Holds all registered plugins.
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

    pub fn register_with_project(
        &mut self,
        project_dir: &Path,
        plugin: Box<dyn Plugin>,
    ) -> Result<()> {
        crate::project::register_ship_namespace(project_dir, plugin.namespace_claim())?;
        self.plugins.push(plugin);
        Ok(())
    }

    pub fn plugins(&self) -> &[Box<dyn Plugin>] {
        &self.plugins
    }
}
