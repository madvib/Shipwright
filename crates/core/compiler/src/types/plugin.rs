use serde::{Deserialize, Serialize};

/// A single plugin to install.
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginEntry {
    /// Plugin ID, e.g. `"superpowers@claude-plugins-official"`.
    pub id: String,
    /// Provider this plugin belongs to, e.g. `"claude"`.
    pub provider: String,
}

impl Default for PluginEntry {
    fn default() -> Self {
        Self {
            id: String::new(),
            provider: "claude".into(),
        }
    }
}

/// Declared plugin intent produced by the compiler.
/// The CLI/runtime reads this and executes installs — the compiler never does.
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct PluginsManifest {
    /// Plugins that should be installed.
    pub install: Vec<PluginEntry>,
    /// Installation scope: `"project"` | `"user"`.
    pub scope: String,
}

impl PluginsManifest {
    pub fn is_empty(&self) -> bool {
        self.install.is_empty()
    }
}
