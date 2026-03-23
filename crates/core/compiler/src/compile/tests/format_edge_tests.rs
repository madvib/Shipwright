//! Edge-case tests for format translation functions.
//!
//! These test the tricky corners of json→toml, glob→regex,
//! and permission translation that aren't covered by the
//! happy-path integration tests in each provider's test file.

use crate::compile::compile;
use crate::resolve::ResolvedConfig;
use crate::types::{Permissions, ToolPermissions};

use super::fixtures::*;

// ── json_to_toml edge cases (tested via Codex settings_extra) ───────────────

#[test]
fn codex_settings_extra_nested_object_becomes_toml_table() {
    let r = ResolvedConfig {
        codex_settings_extra: Some(serde_json::json!({
            "agents": { "max_threads": 8, "max_depth": 5 }
        })),
        ..resolved(vec![])
    };
    let out = compile(&r, "codex").unwrap();
    let patch = out.codex_config_patch.as_ref().unwrap();
    let parsed: toml::Table = toml::from_str(patch).expect("must be valid TOML");
    let agents = parsed["agents"].as_table().unwrap();
    assert_eq!(agents["max_threads"].as_integer().unwrap(), 8);
    assert_eq!(agents["max_depth"].as_integer().unwrap(), 5);
}

#[test]
fn codex_settings_extra_array_becomes_toml_array() {
    let r = ResolvedConfig {
        codex_settings_extra: Some(serde_json::json!({
            "allowed_hosts": ["github.com", "npmjs.org"]
        })),
        ..resolved(vec![])
    };
    let out = compile(&r, "codex").unwrap();
    let patch = out.codex_config_patch.as_ref().unwrap();
    let parsed: toml::Table = toml::from_str(patch).expect("must be valid TOML");
    let hosts = parsed["allowed_hosts"].as_array().unwrap();
    assert_eq!(hosts.len(), 2);
    assert_eq!(hosts[0].as_str().unwrap(), "github.com");
}

#[test]
fn codex_settings_extra_null_value_dropped() {
    let r = ResolvedConfig {
        codex_settings_extra: Some(serde_json::json!({
            "keep": "yes",
            "drop": null
        })),
        ..resolved(vec![])
    };
    let out = compile(&r, "codex").unwrap();
    let patch = out.codex_config_patch.as_ref().unwrap();
    let parsed: toml::Table = toml::from_str(patch).expect("must be valid TOML");
    assert!(parsed.contains_key("keep"));
    assert!(!parsed.contains_key("drop"), "null values must not appear in TOML");
}

#[test]
fn codex_settings_extra_float_preserved() {
    let r = ResolvedConfig {
        codex_settings_extra: Some(serde_json::json!({ "temperature": 0.7 })),
        ..resolved(vec![])
    };
    let out = compile(&r, "codex").unwrap();
    let patch = out.codex_config_patch.as_ref().unwrap();
    let parsed: toml::Table = toml::from_str(patch).expect("must be valid TOML");
    let temp = parsed["temperature"].as_float().unwrap();
    assert!((temp - 0.7).abs() < 0.001);
}

// ── glob_to_regex edge cases (tested via Gemini policy) ─────────────────────

#[test]
fn gemini_policy_regex_metacharacters_escaped() {
    // A path like "src/foo.bar" must escape the dot so it doesn't match any char
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions {
                deny: vec!["Read(src/config.json)".to_string()],
                ..Default::default()
            },
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "gemini").unwrap();
    let toml_str = out.gemini_policy_patch.unwrap();
    let parsed: toml::Table = toml::from_str(&toml_str).unwrap();
    let pattern = parsed["tool_policies"].as_array().unwrap()[0]["pattern"]
        .as_str()
        .unwrap();
    assert!(
        pattern.contains("\\."),
        "dots in paths must be escaped in regex: {pattern}"
    );
    assert!(
        !pattern.contains("config.json") || pattern.contains("config\\.json"),
        "literal dot must be escaped: {pattern}"
    );
}

#[test]
fn gemini_policy_bash_glob_with_braces_escaped() {
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions {
                deny: vec!["Bash(echo ${HOME})".to_string()],
                ..Default::default()
            },
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "gemini").unwrap();
    let toml_str = out.gemini_policy_patch.unwrap();
    let parsed: toml::Table = toml::from_str(&toml_str).unwrap();
    let pattern = parsed["tool_policies"].as_array().unwrap()[0]["pattern"]
        .as_str()
        .unwrap();
    assert!(
        pattern.contains("\\{") && pattern.contains("\\}"),
        "braces must be escaped in regex: {pattern}"
    );
}

// ── Gemini permission translation edge cases ────────────────────────────────

#[test]
fn gemini_policy_edit_maps_to_file_write() {
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions {
                deny: vec!["Edit".to_string()],
                ..Default::default()
            },
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "gemini").unwrap();
    let toml_str = out.gemini_policy_patch.unwrap();
    let parsed: toml::Table = toml::from_str(&toml_str).unwrap();
    let tool = parsed["tool_policies"].as_array().unwrap()[0]["tool"]
        .as_str()
        .unwrap();
    assert_eq!(tool, "file_write", "Edit → file_write");
}

#[test]
fn gemini_policy_glob_maps_to_file_read() {
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions {
                deny: vec!["Glob".to_string()],
                ..Default::default()
            },
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "gemini").unwrap();
    let toml_str = out.gemini_policy_patch.unwrap();
    let parsed: toml::Table = toml::from_str(&toml_str).unwrap();
    let tool = parsed["tool_policies"].as_array().unwrap()[0]["tool"]
        .as_str()
        .unwrap();
    assert_eq!(tool, "file_read", "Glob → file_read");
}

#[test]
fn gemini_policy_parameterized_write_has_pattern() {
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions {
                deny: vec!["Write(*.env)".to_string()],
                ..Default::default()
            },
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "gemini").unwrap();
    let toml_str = out.gemini_policy_patch.unwrap();
    let parsed: toml::Table = toml::from_str(&toml_str).unwrap();
    let entry = &parsed["tool_policies"].as_array().unwrap()[0];
    assert_eq!(entry["tool"].as_str().unwrap(), "file_write");
    let pattern = entry["pattern"].as_str().unwrap();
    assert!(
        pattern.contains(".*\\.env"),
        "Write(*.env) pattern must have glob expansion: {pattern}"
    );
}

#[test]
fn gemini_policy_webfetch_with_url_pattern() {
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions {
                allow: vec!["WebFetch(https://api.example.com/*)".to_string()],
                ..Default::default()
            },
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "gemini").unwrap();
    let toml_str = out.gemini_policy_patch.unwrap();
    let parsed: toml::Table = toml::from_str(&toml_str).unwrap();
    let entry = &parsed["tool_policies"].as_array().unwrap()[0];
    assert_eq!(entry["tool"].as_str().unwrap(), "web_fetch");
    assert_eq!(entry["decision"].as_str().unwrap(), "allow");
}

// ── OpenCode permission translation edge cases ──────────────────────────────

#[test]
fn opencode_unknown_tool_pattern_dropped() {
    // A tool pattern that doesn't map to any known OpenCode permission key
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions {
                deny: vec!["UnknownTool".to_string()],
                ..Default::default()
            },
            ..Default::default()
        },
        ..resolved(vec![make_server("test")])
    };
    let out = compile(&r, "opencode").unwrap();
    let patch = out
        .opencode_config_patch
        .expect("servers must trigger patch");
    // Unknown patterns are silently dropped — permission key should not appear
    assert!(
        patch.get("permission").is_none(),
        "unknown tool pattern should not produce a permission entry"
    );
}

#[test]
fn opencode_mixed_allow_deny_same_tool_produces_granular() {
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions {
                allow: vec!["Bash(git commit *)".to_string()],
                deny: vec!["Bash(git push --force *)".to_string()],
                ..Default::default()
            },
            ..Default::default()
        },
        ..resolved(vec![make_server("test")])
    };
    let out = compile(&r, "opencode").unwrap();
    let patch = out.opencode_config_patch.unwrap();
    let bash = &patch["permission"]["bash"];
    assert!(bash.is_object(), "mixed bash patterns → granular object");
    assert_eq!(bash["git commit *"].as_str().unwrap(), "allow");
    assert_eq!(bash["git push --force *"].as_str().unwrap(), "deny");
}
