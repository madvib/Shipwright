use crate::types::{PluginEntry, PluginsManifest};

/// Build the plugins manifest for a given provider.
/// Plugin entries inherit the provider from the manifest scope or default to
/// the compile target when no provider hint is encoded in the plugin ID.
pub(super) fn build_plugins_manifest(
    plugins: &PluginsManifest,
    provider_id: &str,
) -> PluginsManifest {
    if plugins.install.is_empty() {
        return PluginsManifest::default();
    }

    let entries = plugins
        .install
        .iter()
        .map(|entry| PluginEntry {
            id: entry.id.clone(),
            provider: if entry.provider.is_empty() {
                provider_id.to_string()
            } else {
                entry.provider.clone()
            },
        })
        .collect();

    PluginsManifest {
        install: entries,
        scope: if plugins.scope.is_empty() {
            "project".to_string()
        } else {
            plugins.scope.clone()
        },
    }
}
