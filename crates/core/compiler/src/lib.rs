pub mod compile;
pub mod resolve;
pub mod types;

// ─── Top-level re-exports ─────────────────────────────────────────────────────

pub use compile::{
    CompileOutput, ContextFile, McpKey, ProviderDescriptor, SkillsDir,
    build_claude_settings_patch, compile, get_provider, list_providers,
};
pub use resolve::{FeatureOverrides, ResolvedConfig, resolve};
pub use types::{
    AgentLayerConfig, AiConfig, CatalogEntry, CatalogKind, GitConfig, HookConfig, HookTrigger,
    McpServerConfig, McpServerType, ModeConfig, NamespaceConfig, PermissionConfig, Permissions,
    ProjectConfig, Rule, Skill, SkillSource, StatusConfig,
    list_catalog, list_catalog_by_kind, search_catalog,
};

/// Generate a nanoid using Ship's 56-character alphabet (no ambiguous chars).
pub fn gen_nanoid() -> String {
    let alphabet: [char; 56] = [
        '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J',
        'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b',
        'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'm', 'n', 'p', 'q', 'r', 's', 't', 'u',
        'v', 'w', 'x', 'y', 'z',
    ];
    nanoid::format(nanoid::rngs::default, &alphabet, 8)
}
