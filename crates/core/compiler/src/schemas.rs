//! Canonical upstream provider schema URLs.
//!
//! Used by:
//! - Ship JSON schemas — `$ref` in provider_settings / provider_defaults
//! - Web app — CodeMirror autocomplete for provider_settings
//! - CI — drift detection (schema_drift_tests.rs)
//!
//! When a provider publishes a new schema URL, update this file.

/// Claude Code settings JSON Schema (SchemaStore).
pub const CLAUDE_SCHEMA_URL: &str =
    "https://www.schemastore.org/claude-code-settings.json";

/// OpenAI Codex config JSON Schema (GitHub).
pub const CODEX_SCHEMA_URL: &str =
    "https://raw.githubusercontent.com/openai/codex/main/codex-rs/core/config.schema.json";

/// Gemini CLI settings JSON Schema (GitHub).
pub const GEMINI_SCHEMA_URL: &str =
    "https://raw.githubusercontent.com/google-gemini/gemini-cli/main/schemas/settings.schema.json";

/// OpenCode config JSON Schema (official site).
pub const OPENCODE_SCHEMA_URL: &str = "https://opencode.ai/config.json";

/// Cursor has no published JSON Schema. Config is file-based:
/// `.cursor/rules/*.mdc`, `.cursor/mcp.json`, `.cursor/cli.json`, `.cursor/hooks.json`.
pub const CURSOR_SCHEMA_URL: Option<&str> = None;

/// All provider schema URLs, keyed by provider ID.
/// Returns `(provider_id, Option<schema_url>)`.
pub const PROVIDER_SCHEMAS: &[(&str, Option<&str>)] = &[
    ("claude", Some(CLAUDE_SCHEMA_URL)),
    ("codex", Some(CODEX_SCHEMA_URL)),
    ("gemini", Some(GEMINI_SCHEMA_URL)),
    ("cursor", None),
    ("opencode", Some(OPENCODE_SCHEMA_URL)),
];

/// Look up the upstream schema URL for a provider.
pub fn schema_url(provider_id: &str) -> Option<&'static str> {
    PROVIDER_SCHEMAS
        .iter()
        .find(|(id, _)| *id == provider_id)
        .and_then(|(_, url)| *url)
}

/// Ship-managed keys per provider.
///
/// These keys are written by the compiler from Ship's own schema fields
/// (model, permissions, env, agent_limits, hooks, MCP). They must NOT
/// appear in `provider_settings` / `provider_defaults` — the JSON schemas
/// use `allOf` + `false` to reject them.
///
/// Source of truth: each provider's `build_*` function in `compile/`.
pub const PROVIDER_MANAGED_KEYS: &[(&str, &[&str])] = &[
    (
        "claude",
        &[
            "permissions",
            "hooks",
            "model",
            "env",
            "availableModels",
            "maxCostPerSession",
            "maxTurns",
            "autoMemoryEnabled",
        ],
    ),
    ("codex", &["model", "mcp_servers"]),
    ("gemini", &["model", "hooks", "mcpServers"]),
    ("cursor", &[]),
    ("opencode", &["model", "mcp", "permission"]),
];

/// Look up Ship-managed keys for a provider.
pub fn managed_keys(provider_id: &str) -> &'static [&'static str] {
    PROVIDER_MANAGED_KEYS
        .iter()
        .find(|(id, _)| *id == provider_id)
        .map(|(_, keys)| *keys)
        .unwrap_or(&[])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_url_returns_known_providers() {
        assert!(schema_url("claude").is_some());
        assert!(schema_url("codex").is_some());
        assert!(schema_url("gemini").is_some());
        assert!(schema_url("opencode").is_some());
        assert!(schema_url("cursor").is_none());
    }

    #[test]
    fn schema_url_returns_none_for_unknown() {
        assert!(schema_url("unknown-provider").is_none());
    }
}
