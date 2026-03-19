use crate::compile::compile;
use crate::resolve::ResolvedConfig;

use super::fixtures::*;

/// Source: https://developers.openai.com/codex/mcp
/// Codex config is TOML — codex_config_patch must be a valid TOML string.
#[test]
fn codex_produces_toml_patch_not_json() {
    let r = resolved(vec![make_server("github")]);
    let out = compile(&r, "codex").unwrap();
    let toml_str = out.codex_config_patch.expect("codex must emit a TOML config patch");
    let parsed: toml::Table = toml::from_str(&toml_str).expect("must be valid TOML");
    assert!(parsed.contains_key("mcp_servers"), "must have mcp_servers table");
}

#[test]
fn codex_toml_ship_server_first() {
    let r = resolved(vec![make_server("github")]);
    let out = compile(&r, "codex").unwrap();
    let toml_str = out.codex_config_patch.unwrap();
    let parsed: toml::Table = toml::from_str(&toml_str).unwrap();
    let mcp = parsed["mcp_servers"].as_table().unwrap();
    let keys: Vec<&str> = mcp.keys().map(|k| k.as_str()).collect();
    assert_eq!(keys[0], "ship", "ship server must be first in Codex TOML output");
}

#[test]
fn codex_toml_stdio_entry_shape() {
    let mut s = make_server("context7");
    s.args = vec!["-y".into(), "@upstash/context7-mcp".into()];
    let r = resolved(vec![s]);
    let out = compile(&r, "codex").unwrap();
    let toml_str = out.codex_config_patch.unwrap();
    let parsed: toml::Table = toml::from_str(&toml_str).unwrap();
    let entry = parsed["mcp_servers"]["context7"].as_table().unwrap();
    assert_eq!(entry["command"].as_str().unwrap(), "npx");
    assert_eq!(entry["args"].as_array().unwrap().len(), 2);
    assert!(entry.get("type").is_none(), "no type field in Codex TOML");
}

#[test]
fn codex_toml_disabled_server_excluded() {
    let mut s = make_server("disabled");
    s.disabled = true;
    let r = resolved(vec![s]);
    let out = compile(&r, "codex").unwrap();
    let toml_str = out.codex_config_patch.unwrap();
    let parsed: toml::Table = toml::from_str(&toml_str).unwrap();
    let mcp = parsed["mcp_servers"].as_table().unwrap();
    assert!(!mcp.contains_key("disabled"));
}

#[test]
fn codex_toml_timeout_uses_startup_timeout_sec() {
    let mut s = make_server("slow");
    s.timeout_secs = Some(30);
    let r = resolved(vec![s]);
    let out = compile(&r, "codex").unwrap();
    let toml_str = out.codex_config_patch.unwrap();
    let parsed: toml::Table = toml::from_str(&toml_str).unwrap();
    assert_eq!(parsed["mcp_servers"]["slow"]["startup_timeout_sec"].as_integer().unwrap(), 30);
}

#[test]
fn codex_no_json_mcp_key_confusion() {
    let r = resolved(vec![make_server("x")]);
    assert!(compile(&r, "codex").unwrap().codex_config_patch.is_some());
    assert!(compile(&r, "claude").unwrap().codex_config_patch.is_none());
    assert!(compile(&r, "cursor").unwrap().codex_config_patch.is_none());
    assert!(compile(&r, "gemini").unwrap().codex_config_patch.is_none());
}

#[test]
fn codex_model_emitted_in_config_patch() {
    let r = ResolvedConfig {
        model: Some("o3".to_string()),
        ..resolved(vec![])
    };
    let out = compile(&r, "codex").unwrap();
    let patch = out.codex_config_patch.as_ref().expect("model should trigger codex patch");
    assert!(patch.contains("model = \"o3\""), "codex TOML should contain model field: {}", patch);
}

// ── Codex sandbox mode ───────────────────────────────────────────────────────

#[test]
fn codex_sandbox_emitted_in_config_patch() {
    let r = ResolvedConfig {
        codex_sandbox: Some("network-only".to_string()),
        ..resolved(vec![])
    };
    let out = compile(&r, "codex").unwrap();
    let patch = out.codex_config_patch.as_ref().expect("sandbox should trigger codex patch");
    assert!(patch.contains("sandbox = \"network-only\""), "codex TOML should contain sandbox field: {}", patch);
}

#[test]
fn codex_sandbox_none_omitted() {
    let r = resolved(vec![]);
    let out = compile(&r, "codex").unwrap();
    let patch = out.codex_config_patch.as_ref().expect("ship MCP always triggers patch");
    assert!(!patch.contains("sandbox"), "codex TOML should not contain sandbox when None: {}", patch);
}

#[test]
fn codex_sandbox_not_emitted_for_other_providers() {
    let r = ResolvedConfig {
        codex_sandbox: Some("full".to_string()),
        ..resolved(vec![])
    };
    for provider in &["claude", "gemini", "cursor"] {
        let out = compile(&r, provider).unwrap();
        assert!(out.codex_config_patch.is_none(), "{provider} must not get codex config patch");
    }
}
