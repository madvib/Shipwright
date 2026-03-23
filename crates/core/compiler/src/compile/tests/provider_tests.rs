use crate::compile::{compile, get_provider, list_providers};
use crate::resolve::ResolvedConfig;
use crate::types::{PluginEntry, PluginsManifest};

use super::fixtures::*;

// ── Context file audit ────────────────────────────────────────────────────────

#[test]
fn claude_context_content_contains_only_rules_not_skills() {
    let r = ResolvedConfig {
        rules: vec![make_rule("rule.md", "Rule content here.")],
        skills: vec![make_skill("my-skill")],
        ..resolved(vec![])
    };
    let out = compile(&r, "claude").unwrap();
    let content = out
        .context_content
        .expect("claude must have context_content when rules present");
    assert!(
        content.contains("Rule content here."),
        "rules must appear in context"
    );
    assert!(
        !content.contains("Do the thing."),
        "skill content must NOT appear in context file"
    );
}

#[test]
fn gemini_context_content_contains_only_rules() {
    let r = ResolvedConfig {
        rules: vec![make_rule("rule.md", "Gemini rule content.")],
        skills: vec![make_skill("my-skill")],
        ..resolved(vec![])
    };
    let out = compile(&r, "gemini").unwrap();
    let content = out
        .context_content
        .expect("gemini must have context_content");
    assert!(content.contains("Gemini rule content."));
    assert!(!content.contains("Do the thing."));
}

#[test]
fn codex_context_content_contains_only_rules() {
    let r = ResolvedConfig {
        rules: vec![make_rule("rule.md", "Codex rule content.")],
        skills: vec![make_skill("my-skill")],
        ..resolved(vec![])
    };
    let out = compile(&r, "codex").unwrap();
    let content = out
        .context_content
        .expect("codex must have context_content");
    assert!(content.contains("Codex rule content."));
    assert!(!content.contains("Do the thing."));
}

#[test]
fn context_file_is_none_when_all_rule_content_empty() {
    let r = ResolvedConfig {
        rules: vec![
            make_rule("blank.md", ""),
            make_rule("whitespace.md", "   \n  "),
        ],
        ..resolved(vec![])
    };
    assert!(compile(&r, "claude").unwrap().context_content.is_none());
    assert!(compile(&r, "gemini").unwrap().context_content.is_none());
    assert!(compile(&r, "codex").unwrap().context_content.is_none());
}

// ── ProviderFeatureFlags ──────────────────────────────────────────────────────

#[test]
fn claude_feature_flags() {
    let flags = get_provider("claude").unwrap().feature_flags();
    assert!(flags.supports_mcp);
    assert!(flags.supports_hooks);
    assert!(flags.supports_tool_permissions);
    assert!(flags.supports_memory);
}

#[test]
fn gemini_feature_flags() {
    let flags = get_provider("gemini").unwrap().feature_flags();
    assert!(flags.supports_mcp);
    assert!(flags.supports_hooks);
    assert!(flags.supports_tool_permissions);
    assert!(flags.supports_memory);
}

#[test]
fn codex_feature_flags() {
    let flags = get_provider("codex").unwrap().feature_flags();
    assert!(flags.supports_mcp);
    assert!(!flags.supports_hooks);
    assert!(!flags.supports_tool_permissions);
    assert!(flags.supports_memory);
}

#[test]
fn cursor_feature_flags() {
    let flags = get_provider("cursor").unwrap().feature_flags();
    assert!(flags.supports_mcp);
    assert!(flags.supports_hooks);
    assert!(flags.supports_tool_permissions);
    assert!(!flags.supports_memory);
}

#[test]
fn feature_flags_govern_mcp_emission() {
    let r = resolved(vec![make_server("github")]);
    for desc in list_providers() {
        let flags = desc.feature_flags();
        let out = compile(&r, desc.id).unwrap();
        if flags.supports_mcp {
            assert!(
                out.mcp_servers
                    .as_object()
                    .is_some_and(|o| o.contains_key("ship")),
                "provider {} (supports_mcp=true) must include ship server",
                desc.id
            );
        } else {
            assert!(
                out.mcp_servers.as_object().is_some_and(|o| o.is_empty()),
                "provider {} (supports_mcp=false) must not emit MCP entries",
                desc.id
            );
            assert!(
                out.mcp_config_path.is_none(),
                "provider {} must not emit mcp_config_path",
                desc.id
            );
        }
    }
}

// ── Plugins manifest ──────────────────────────────────────────────────────────

#[test]
fn no_plugins_gives_empty_manifest() {
    let out = compile(&resolved(vec![]), "claude").unwrap();
    assert!(out.plugins_manifest.install.is_empty());
    assert!(out.plugins_manifest.scope.is_empty());
}

#[test]
fn plugins_manifest_populated_from_resolved() {
    let r = ResolvedConfig {
        plugins: PluginsManifest {
            install: vec![
                PluginEntry {
                    id: "superpowers@claude-plugins-official".into(),
                    provider: "claude".into(),
                },
                PluginEntry {
                    id: "rust-analyzer-lsp@claude-plugins-official".into(),
                    provider: "claude".into(),
                },
            ],
            scope: "project".to_string(),
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "claude").unwrap();
    assert_eq!(out.plugins_manifest.install.len(), 2);
    assert_eq!(
        out.plugins_manifest.install[0].id,
        "superpowers@claude-plugins-official"
    );
    assert_eq!(out.plugins_manifest.scope, "project");
}

#[test]
fn plugins_manifest_empty_provider_defaults_to_compile_target() {
    let r = ResolvedConfig {
        plugins: PluginsManifest {
            install: vec![PluginEntry {
                id: "my-plugin@registry".into(),
                provider: String::new(),
            }],
            scope: "user".to_string(),
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "gemini").unwrap();
    assert_eq!(out.plugins_manifest.install[0].provider, "gemini");
    assert_eq!(out.plugins_manifest.scope, "user");
}

#[test]
fn plugins_manifest_empty_scope_defaults_to_project() {
    let r = ResolvedConfig {
        plugins: PluginsManifest {
            install: vec![PluginEntry {
                id: "p@reg".into(),
                provider: "claude".into(),
            }],
            scope: String::new(),
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "claude").unwrap();
    assert_eq!(out.plugins_manifest.scope, "project");
}

#[test]
fn plugins_manifest_is_same_across_providers() {
    let manifest = PluginsManifest {
        install: vec![PluginEntry {
            id: "tool@reg".into(),
            provider: "claude".into(),
        }],
        scope: "project".to_string(),
    };
    let r = ResolvedConfig {
        plugins: manifest.clone(),
        ..resolved(vec![])
    };
    for provider in &["claude", "gemini", "codex", "cursor"] {
        let out = compile(&r, provider).unwrap();
        assert_eq!(
            out.plugins_manifest.install.len(),
            1,
            "provider {} must include manifest",
            provider
        );
    }
}
