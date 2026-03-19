use crate::compile::compile;
use crate::resolve::ResolvedConfig;
use crate::types::{Permissions, ToolPermissions};

use super::fixtures::*;

// ── Gemini policy patch ───────────────────────────────────────────────────────

/// Default permissions → no policy patch.
#[test]
fn gemini_policy_default_permissions_emit_none() {
    let out = compile(&resolved(vec![]), "gemini").unwrap();
    assert!(out.gemini_policy_patch.is_none());
}

#[test]
fn gemini_policy_allow_star_no_deny_is_none() {
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions { allow: vec!["*".to_string()], deny: vec![], ..Default::default() },
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "gemini").unwrap();
    assert!(out.gemini_policy_patch.is_none());
}

/// Deny patterns → `decision = "deny"` in TOML output.
#[test]
fn gemini_policy_deny_translates_to_deny_decision() {
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions { deny: vec!["Bash(rm -rf *)".to_string()], ..Default::default() },
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "gemini").unwrap();
    let toml_str = out.gemini_policy_patch.expect("deny must emit gemini policy patch");
    let parsed: toml::Table = toml::from_str(&toml_str).expect("must be valid TOML");
    let policies = parsed["tool_policies"].as_array().expect("must have tool_policies array");
    assert_eq!(policies.len(), 1);
    assert_eq!(policies[0]["tool"].as_str().unwrap(), "shell");
    assert_eq!(policies[0]["decision"].as_str().unwrap(), "deny");
    let pattern = policies[0]["pattern"].as_str().unwrap();
    assert!(pattern.contains("rm"), "pattern must encode the command");
}

/// Ask patterns → `decision = "ask_user"` in TOML output.
#[test]
fn gemini_policy_ask_translates_to_ask_user() {
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions { ask: vec!["mcp__*__delete*".to_string()], ..Default::default() },
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "gemini").unwrap();
    let toml_str = out.gemini_policy_patch.expect("ask must emit gemini policy patch");
    let parsed: toml::Table = toml::from_str(&toml_str).expect("valid TOML");
    let policies = parsed["tool_policies"].as_array().unwrap();
    assert_eq!(policies.len(), 1);
    assert_eq!(policies[0]["tool"].as_str().unwrap(), "mcp");
    assert_eq!(policies[0]["decision"].as_str().unwrap(), "ask_user");
}

/// Non-default allow list → `decision = "allow"` entries.
#[test]
fn gemini_policy_explicit_allow_translates_to_allow_decision() {
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions { allow: vec!["Bash(git *)".to_string()], ..Default::default() },
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "gemini").unwrap();
    let toml_str = out.gemini_policy_patch.expect("explicit allow must emit gemini policy patch");
    let parsed: toml::Table = toml::from_str(&toml_str).expect("valid TOML");
    let policies = parsed["tool_policies"].as_array().unwrap();
    assert_eq!(policies.len(), 1);
    assert_eq!(policies[0]["tool"].as_str().unwrap(), "shell");
    assert_eq!(policies[0]["decision"].as_str().unwrap(), "allow");
    let pattern = policies[0]["pattern"].as_str().unwrap();
    assert!(pattern.contains("git"), "pattern must encode the git command");
}

/// MCP patterns (mcp__server__tool) translate to `tool = "mcp"` with a regex pattern.
#[test]
fn gemini_policy_mcp_pattern_translated() {
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions { deny: vec!["mcp__github__delete_issue".to_string()], ..Default::default() },
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "gemini").unwrap();
    let toml_str = out.gemini_policy_patch.expect("mcp deny must emit gemini policy patch");
    let parsed: toml::Table = toml::from_str(&toml_str).expect("valid TOML");
    let policies = parsed["tool_policies"].as_array().unwrap();
    assert_eq!(policies[0]["tool"].as_str().unwrap(), "mcp");
    let pattern = policies[0]["pattern"].as_str().unwrap();
    assert!(pattern.contains("github"), "mcp pattern must include server name");
    assert!(pattern.contains("delete_issue"), "mcp pattern must include tool name");
}

/// Read/Glob → file_read, Write/Edit → file_write, WebFetch → web_fetch.
#[test]
fn gemini_policy_file_and_web_tool_mapping() {
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions {
                deny: vec!["Read".to_string(), "Write".to_string(), "WebFetch".to_string()],
                ..Default::default()
            },
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "gemini").unwrap();
    let toml_str = out.gemini_policy_patch.expect("must emit gemini policy patch");
    let parsed: toml::Table = toml::from_str(&toml_str).expect("valid TOML");
    let policies = parsed["tool_policies"].as_array().unwrap();
    assert_eq!(policies.len(), 3);
    let tools: Vec<&str> = policies.iter().map(|p| p["tool"].as_str().unwrap()).collect();
    assert!(tools.contains(&"file_read"),  "Read → file_read");
    assert!(tools.contains(&"file_write"), "Write → file_write");
    assert!(tools.contains(&"web_fetch"),  "WebFetch → web_fetch");
}

/// Bare `*` wildcard in deny must not produce a policy entry.
#[test]
fn gemini_policy_bare_wildcard_dropped() {
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions { deny: vec!["*".to_string()], ..Default::default() },
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "gemini").unwrap();
    assert!(out.gemini_policy_patch.is_none());
}

/// Policy patch is valid TOML with the `[[tool_policies]]` array-of-tables structure.
#[test]
fn gemini_policy_output_is_valid_toml() {
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions {
                deny: vec!["Bash(rm -rf *)".to_string(), "mcp__*__delete*".to_string()],
                ask: vec!["Write".to_string()],
                ..Default::default()
            },
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "gemini").unwrap();
    let toml_str = out.gemini_policy_patch.expect("must emit policy patch");
    let parsed: toml::Table = toml::from_str(&toml_str).expect("must be valid TOML");
    assert!(parsed.contains_key("tool_policies"), "must use [[tool_policies]] array");
    let policies = parsed["tool_policies"].as_array().unwrap();
    assert_eq!(policies.len(), 3, "deny(2) + ask(1) = 3 entries");
}

/// Gemini policy patch is only emitted for the gemini provider.
#[test]
fn gemini_policy_only_for_gemini_not_other_providers() {
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions { deny: vec!["Bash(rm -rf *)".to_string()], ..Default::default() },
            ..Default::default()
        },
        ..resolved(vec![])
    };
    assert!(compile(&r, "claude").unwrap().gemini_policy_patch.is_none());
    assert!(compile(&r, "codex").unwrap().gemini_policy_patch.is_none());
    assert!(compile(&r, "cursor").unwrap().gemini_policy_patch.is_none());
    assert!(compile(&r, "gemini").unwrap().gemini_policy_patch.is_some());
}

/// Glob wildcard `*` is converted to `.*` in the regex output.
#[test]
fn gemini_policy_glob_star_converts_to_regex_dotstar() {
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions { deny: vec!["Bash(git *)".to_string()], ..Default::default() },
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "gemini").unwrap();
    let toml_str = out.gemini_policy_patch.unwrap();
    let parsed: toml::Table = toml::from_str(&toml_str).unwrap();
    let pattern = parsed["tool_policies"].as_array().unwrap()[0]["pattern"].as_str().unwrap();
    assert!(pattern.contains(".*"), "glob * must become .* in regex");
    assert!(!pattern.contains(" *"), "raw glob space-star must not remain");
}
