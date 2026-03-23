use crate::compile::compile;
use crate::resolve::ResolvedConfig;

use super::fixtures::*;

/// Source: https://developers.openai.com/codex/mcp
/// Codex config is TOML — codex_config_patch must be a valid TOML string.
#[test]
fn codex_produces_toml_patch_not_json() {
    let r = resolved(vec![make_server("github")]);
    let out = compile(&r, "codex").unwrap();
    let toml_str = out
        .codex_config_patch
        .expect("codex must emit a TOML config patch");
    let parsed: toml::Table = toml::from_str(&toml_str).expect("must be valid TOML");
    assert!(
        parsed.contains_key("mcp_servers"),
        "must have mcp_servers table"
    );
}

#[test]
fn codex_toml_ship_server_first() {
    let r = resolved(vec![make_server("github")]);
    let out = compile(&r, "codex").unwrap();
    let toml_str = out.codex_config_patch.unwrap();
    let parsed: toml::Table = toml::from_str(&toml_str).unwrap();
    let mcp = parsed["mcp_servers"].as_table().unwrap();
    let keys: Vec<&str> = mcp.keys().map(|k| k.as_str()).collect();
    assert_eq!(
        keys[0], "ship",
        "ship server must be first in Codex TOML output"
    );
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
    assert_eq!(
        parsed["mcp_servers"]["slow"]["startup_timeout_sec"]
            .as_integer()
            .unwrap(),
        30
    );
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
    let patch = out
        .codex_config_patch
        .as_ref()
        .expect("model should trigger codex patch");
    assert!(
        patch.contains("model = \"o3\""),
        "codex TOML should contain model field: {}",
        patch
    );
}

// ── Codex sandbox mode (Phase 1B) ────────────────────────────────────────────

/// Phase 1B: key must be `sandbox_mode` (not `sandbox`), values translated.
#[test]
fn codex_sandbox_key_is_sandbox_mode() {
    let r = ResolvedConfig {
        codex_sandbox: Some("network-only".to_string()),
        ..resolved(vec![])
    };
    let out = compile(&r, "codex").unwrap();
    let patch = out
        .codex_config_patch
        .as_ref()
        .expect("sandbox should trigger codex patch");
    assert!(
        !patch.contains("sandbox ="),
        "old key 'sandbox' must NOT be emitted: {}",
        patch
    );
    assert!(
        patch.contains("sandbox_mode"),
        "new key 'sandbox_mode' must be emitted: {}",
        patch
    );
}

#[test]
fn codex_sandbox_values_translated_correctly() {
    let cases = [
        ("full", "danger-full-internet"),
        ("network-only", "network-disabled"),
        ("off", "disabled"),
    ];
    for (input, expected) in cases {
        let r = ResolvedConfig {
            codex_sandbox: Some(input.to_string()),
            ..resolved(vec![])
        };
        let out = compile(&r, "codex").unwrap();
        let patch = out.codex_config_patch.as_ref().unwrap();
        assert!(
            patch.contains(&format!("sandbox_mode = \"{expected}\"")),
            "input '{}' should produce sandbox_mode = '{}': {}",
            input,
            expected,
            patch
        );
    }
}

#[test]
fn codex_sandbox_none_omitted() {
    let r = resolved(vec![]);
    let out = compile(&r, "codex").unwrap();
    let patch = out
        .codex_config_patch
        .as_ref()
        .expect("ship MCP always triggers patch");
    assert!(
        !patch.contains("sandbox_mode"),
        "codex TOML should not contain sandbox_mode when None: {}",
        patch
    );
    assert!(
        !patch.contains("sandbox ="),
        "codex TOML should not contain sandbox when None: {}",
        patch
    );
}

#[test]
fn codex_sandbox_not_emitted_for_other_providers() {
    let r = ResolvedConfig {
        codex_sandbox: Some("full".to_string()),
        ..resolved(vec![])
    };
    for provider in &["claude", "gemini", "cursor"] {
        let out = compile(&r, provider).unwrap();
        assert!(
            out.codex_config_patch.is_none(),
            "{provider} must not get codex config patch"
        );
    }
}

// ── Phase 4: new Codex fields ─────────────────────────────────────────────────

/// Model from library flows through to codex config as `model = "..."`.
#[test]
fn library_model_flows_to_codex_config() {
    use crate::resolve::{ProjectLibrary, resolve_library};
    let library = ProjectLibrary {
        model: Some("o4-mini".to_string()),
        ..Default::default()
    };
    let resolved = resolve_library(&library, None, None);
    let out = compile(&resolved, "codex").unwrap();
    let patch = out
        .codex_config_patch
        .as_ref()
        .expect("model must trigger codex patch");
    assert!(
        patch.contains("model = \"o4-mini\""),
        "model must be in codex config: {}",
        patch
    );
}

#[test]
fn codex_approval_policy_translated() {
    let cases = [
        ("default", "suggest"),
        ("auto_edit", "auto-edit"),
        ("plan", "full-auto"),
    ];
    for (input, expected) in cases {
        let r = ResolvedConfig {
            codex_approval_policy: Some(input.to_string()),
            ..resolved(vec![])
        };
        let out = compile(&r, "codex").unwrap();
        let patch = out
            .codex_config_patch
            .as_ref()
            .expect("approval policy must trigger patch");
        assert!(
            patch.contains(&format!("approval_policy = \"{expected}\"")),
            "input '{}' should produce approval_policy = '{}': {}",
            input,
            expected,
            patch
        );
    }
}

#[test]
fn codex_reasoning_effort_emitted() {
    let r = ResolvedConfig {
        codex_reasoning_effort: Some("high".to_string()),
        ..resolved(vec![])
    };
    let out = compile(&r, "codex").unwrap();
    let patch = out.codex_config_patch.as_ref().unwrap();
    assert!(
        patch.contains("model_reasoning_effort = \"high\""),
        "{}",
        patch
    );
}

#[test]
fn codex_agents_table_emitted() {
    let r = ResolvedConfig {
        codex_max_threads: Some(4),
        codex_max_depth: Some(3),
        codex_job_max_runtime_seconds: Some(300),
        ..resolved(vec![])
    };
    let out = compile(&r, "codex").unwrap();
    let patch = out.codex_config_patch.as_ref().unwrap();
    let parsed: toml::Table = toml::from_str(patch).expect("must be valid TOML");
    let agents = parsed.get("agents").expect("agents table must be present");
    assert_eq!(agents["max_threads"].as_integer().unwrap(), 4);
    assert_eq!(agents["max_depth"].as_integer().unwrap(), 3);
    assert_eq!(agents["job_max_runtime_seconds"].as_integer().unwrap(), 300);
}

#[test]
fn codex_shell_environment_policy_emitted() {
    let r = ResolvedConfig {
        codex_shell_env_policy: Some("inherit".to_string()),
        ..resolved(vec![])
    };
    let out = compile(&r, "codex").unwrap();
    let patch = out.codex_config_patch.as_ref().unwrap();
    assert!(
        patch.contains("shell_environment_policy = \"inherit\""),
        "{}",
        patch
    );
}

#[test]
fn codex_notify_emitted() {
    let r = ResolvedConfig {
        codex_notify: Some(serde_json::json!(true)),
        ..resolved(vec![])
    };
    let out = compile(&r, "codex").unwrap();
    let patch = out.codex_config_patch.as_ref().unwrap();
    assert!(patch.contains("notify = true"), "{}", patch);
}

#[test]
fn codex_mcp_server_tool_filters_emitted() {
    let mut s = make_server("filtered-server");
    s.codex_enabled_tools = vec!["read".to_string()];
    s.codex_disabled_tools = vec!["delete".to_string()];
    let r = resolved(vec![s]);
    let out = compile(&r, "codex").unwrap();
    let patch = out.codex_config_patch.as_ref().unwrap();
    let parsed: toml::Table = toml::from_str(patch).expect("must be valid TOML");
    let server = &parsed["mcp_servers"]["filtered-server"];
    let enabled = server["enabled_tools"].as_array().unwrap();
    assert_eq!(enabled[0].as_str().unwrap(), "read");
    let disabled = server["disabled_tools"].as_array().unwrap();
    assert_eq!(disabled[0].as_str().unwrap(), "delete");
}

#[test]
fn codex_settings_extra_merged_verbatim() {
    let r = ResolvedConfig {
        codex_settings_extra: Some(serde_json::json!({ "custom_key": "custom_value" })),
        ..resolved(vec![])
    };
    let out = compile(&r, "codex").unwrap();
    let patch = out.codex_config_patch.as_ref().unwrap();
    assert!(patch.contains("custom_key"), "{}", patch);
}
