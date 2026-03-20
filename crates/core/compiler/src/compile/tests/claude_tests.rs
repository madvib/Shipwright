use crate::compile::compile;
use crate::resolve::ResolvedConfig;
use crate::types::{AgentLimits, HookConfig, HookTrigger, Permissions, ToolPermissions};

use super::fixtures::*;

fn assert_memory_only(patch: &Option<serde_json::Value>) {
    let p = patch.as_ref().expect("patch must exist (autoMemoryEnabled)");
    assert_eq!(p["autoMemoryEnabled"], false);
    assert_eq!(p.as_object().unwrap().len(), 1, "expected only autoMemoryEnabled");
}

// ── Safety: permissions must not silently block tools ─────────────────────────

/// Default config still emits a settings patch — at minimum, autoMemoryEnabled: false.
/// Ship is the memory layer; Claude's built-in memories are disabled by default.
#[test]
fn default_permissions_emit_memory_only_patch() {
    let out = compile(&resolved(vec![]), "claude").unwrap();
    assert_memory_only(&out.claude_settings_patch);
}

/// Explicit deny-only is safe: it only restricts what the user asked to restrict.
#[test]
fn deny_only_emits_patch_with_no_allow_field() {
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions {
                allow: vec!["*".to_string()],
                deny: vec!["Bash(rm -rf *)".to_string()],
                ..Default::default()
            },
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "claude").unwrap();
    let patch = out.claude_settings_patch.expect("deny-only must emit a patch");
    let perms = &patch["permissions"];

    let deny = perms["deny"].as_array().unwrap();
    assert_eq!(deny.len(), 1);
    assert_eq!(deny[0], "Bash(rm -rf *)");

    assert!(
        perms.get("allow").is_none() || perms["allow"].as_array().is_some_and(|a| a.is_empty()),
        "allow field must be absent or empty when the user has not restricted the allowlist"
    );
}

/// The "guarded" preset uses allow=["*"] + scoped deny patterns.
#[test]
fn guarded_preset_never_emits_allow_field() {
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions {
                allow: vec!["*".to_string()],
                deny: vec!["mcp__*__delete*".to_string(), "mcp__*__drop*".to_string()],
                ..Default::default()
            },
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "claude").unwrap();
    let patch = out.claude_settings_patch.expect("deny patterns must emit a patch");
    let perms = &patch["permissions"];
    assert!(
        perms.get("allow").is_none() || perms["allow"].as_array().is_some_and(|a| a.is_empty()),
        "guarded preset (allow=[*] + deny) must not emit an allow field"
    );
    let deny = perms["deny"].as_array().unwrap();
    assert_eq!(deny.len(), 2);
}

#[test]
fn allow_star_with_no_deny_emits_memory_only() {
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions { allow: vec!["*".to_string()], deny: vec![], ..Default::default() },
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "claude").unwrap();
    assert_memory_only(&out.claude_settings_patch);
}

#[test]
fn explicit_allow_list_compiles_correctly() {
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions {
                allow: vec!["Read".to_string(), "Glob".to_string()],
                deny: vec![],
                ..Default::default()
            },
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "claude").unwrap();
    let patch = out.claude_settings_patch.expect("explicit allow must emit a patch");
    let allow = patch["permissions"]["allow"].as_array().unwrap();
    assert_eq!(allow.len(), 2);
    assert!(allow.iter().any(|v| v == "Read"));
    assert!(allow.iter().any(|v| v == "Glob"));
}

#[test]
fn settings_patch_only_for_claude() {
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions {
                allow: vec!["*".to_string()],
                deny: vec!["Write".to_string()],
                ..Default::default()
            },
            ..Default::default()
        },
        ..resolved(vec![])
    };
    assert!(compile(&r, "gemini").unwrap().claude_settings_patch.is_none());
    assert!(compile(&r, "codex").unwrap().claude_settings_patch.is_none());
}

// ── Hooks ─────────────────────────────────────────────────────────────────────

#[test]
fn hooks_compile_into_settings_patch() {
    let r = ResolvedConfig {
        hooks: vec![make_hook(HookTrigger::PreToolUse, "ship hooks check", Some("Bash"))],
        ..resolved(vec![])
    };
    let out = compile(&r, "claude").unwrap();
    let patch = out.claude_settings_patch.expect("hooks must emit a patch");
    let hooks = patch["hooks"]["PreToolUse"].as_array().unwrap();
    assert_eq!(hooks.len(), 1);
    // Each entry is { matcher?, hooks: [{ type, command }] }
    assert_eq!(hooks[0]["matcher"], "Bash");
    let inner = hooks[0]["hooks"].as_array().unwrap();
    assert_eq!(inner.len(), 1);
    assert_eq!(inner[0]["command"], "ship hooks check");
}

#[test]
fn multiple_hooks_same_trigger_all_emitted() {
    let r = ResolvedConfig {
        hooks: vec![
            make_hook(HookTrigger::PostToolUse, "ship log tool", Some("*")),
            make_hook(HookTrigger::PostToolUse, "ship analytics flush", None),
        ],
        ..resolved(vec![])
    };
    let out = compile(&r, "claude").unwrap();
    let patch = out.claude_settings_patch.unwrap();
    let hooks = patch["hooks"]["PostToolUse"].as_array().unwrap();
    assert_eq!(hooks.len(), 2);
}

#[test]
fn hooks_grouped_by_trigger_type() {
    let r = ResolvedConfig {
        hooks: vec![
            make_hook(HookTrigger::PreToolUse, "before", Some("Bash")),
            make_hook(HookTrigger::PostToolUse, "after", None),
            make_hook(HookTrigger::Stop, "on-stop", None),
        ],
        ..resolved(vec![])
    };
    let out = compile(&r, "claude").unwrap();
    let patch = out.claude_settings_patch.unwrap();
    assert_eq!(patch["hooks"]["PreToolUse"].as_array().unwrap().len(), 1);
    assert_eq!(patch["hooks"]["PostToolUse"].as_array().unwrap().len(), 1);
    assert_eq!(patch["hooks"]["Stop"].as_array().unwrap().len(), 1);
    assert!(patch["hooks"].get("PreCompact").is_none());
}

#[test]
fn hook_without_matcher_omits_matcher_field() {
    let r = ResolvedConfig {
        hooks: vec![make_hook(HookTrigger::Stop, "ship notify", None)],
        ..resolved(vec![])
    };
    let out = compile(&r, "claude").unwrap();
    let patch = out.claude_settings_patch.unwrap();
    let hook = &patch["hooks"]["Stop"][0];
    assert!(hook.get("matcher").is_none());
}

// ── Agent limits ──────────────────────────────────────────────────────────────

#[test]
fn max_cost_per_session_emits_to_settings() {
    let r = ResolvedConfig {
        permissions: Permissions {
            agent: AgentLimits { max_cost_per_session: Some(5.0), ..Default::default() },
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "claude").unwrap();
    let patch = out.claude_settings_patch.expect("cost limit must emit a patch");
    assert_eq!(patch["maxCostPerSession"], 5.0);
}

#[test]
fn max_turns_emits_to_settings() {
    let r = ResolvedConfig {
        permissions: Permissions {
            agent: AgentLimits { max_turns: Some(20), ..Default::default() },
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "claude").unwrap();
    let patch = out.claude_settings_patch.expect("turn limit must emit a patch");
    assert_eq!(patch["maxTurns"], 20);
}

// ── claude_settings_extra passthrough ────────────────────────────────────────

#[test]
fn claude_settings_extra_merged_into_patch() {
    let r = ResolvedConfig {
        claude_settings_extra: Some(serde_json::json!({ "customFeature": true, "experimentalMode": "fast" })),
        ..resolved(vec![])
    };
    let out = compile(&r, "claude").unwrap();
    let patch = out.claude_settings_patch.expect("extra settings must trigger patch");
    assert_eq!(patch["customFeature"], true);
    assert_eq!(patch["experimentalMode"], "fast");
}

#[test]
fn claude_settings_extra_null_no_patch() {
    let r = ResolvedConfig {
        claude_settings_extra: Some(serde_json::Value::Null),
        ..resolved(vec![])
    };
    let out = compile(&r, "claude").unwrap();
    assert_memory_only(&out.claude_settings_patch);
}

// ── Ask tier ──────────────────────────────────────────────────────────────────

#[test]
fn ask_tier_compiles_to_permissions_ask() {
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions { ask: vec!["mcp__*__delete*".to_string()], ..Default::default() },
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "claude").unwrap();
    let patch = out.claude_settings_patch.expect("ask tier must emit a patch");
    let ask = patch["permissions"]["ask"].as_array().unwrap();
    assert_eq!(ask.len(), 1);
    assert_eq!(ask[0], "mcp__*__delete*");
}

#[test]
fn ask_tier_default_empty_memory_only() {
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions { allow: vec!["*".to_string()], ask: vec![], deny: vec![] },
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "claude").unwrap();
    assert_memory_only(&out.claude_settings_patch);
}

#[test]
fn ask_with_deny_both_emitted() {
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions {
                ask: vec!["mcp__*__write*".to_string()],
                deny: vec!["Bash(rm -rf *)".to_string()],
                ..Default::default()
            },
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "claude").unwrap();
    let patch = out.claude_settings_patch.expect("ask + deny must emit a patch");
    let perms = &patch["permissions"];
    assert!(perms["ask"].as_array().unwrap().iter().any(|v| v == "mcp__*__write*"));
    assert!(perms["deny"].as_array().unwrap().iter().any(|v| v == "Bash(rm -rf *)"));
}

#[test]
fn ask_tier_not_emitted_for_other_providers() {
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions { ask: vec!["mcp__*__delete*".to_string()], ..Default::default() },
            ..Default::default()
        },
        ..resolved(vec![])
    };
    assert!(compile(&r, "gemini").unwrap().claude_settings_patch.is_none());
    assert!(compile(&r, "codex").unwrap().claude_settings_patch.is_none());
    assert!(compile(&r, "cursor").unwrap().claude_settings_patch.is_none());
}
