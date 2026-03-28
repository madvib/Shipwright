mod artifact_registry;
mod crud;
mod discover;
mod git;
mod io;
mod mcp;
mod merge;
mod modes;
mod project;
mod runtime_settings;
mod types;

#[cfg(test)]
mod tests_crud;
#[cfg(test)]
mod tests_io;

// ─── Public API — mirrors the original config.rs public surface ───────────────

pub use types::{
    AgentLayerConfig, AiConfig, GitConfig, HookConfig, HookTrigger, LEGACY_CONFIG_FILE,
    NamespaceConfig, PRIMARY_CONFIG_FILE, PermissionConfig, StatusConfig,
};

pub use project::{
    AgentProfile, McpConfig, McpSection, McpServerConfig, McpServerType, ProjectConfig,
    ProjectDiscovery,
};

pub use mcp::get_mcp_config;

pub use io::{get_config, get_effective_config, save_config};

pub use git::{
    generate_gitignore, get_git_config, is_category_committed, set_category_committed,
    set_git_config,
};

pub use crud::{
    add_agent, add_hook, add_mcp_server, add_status, ensure_registered_namespaces,
    get_active_agent, get_project_statuses, list_hooks, list_mcp_servers, remove_agent,
    remove_hook, remove_mcp_server, remove_status, set_active_agent,
};

pub use discover::discover_projects;
