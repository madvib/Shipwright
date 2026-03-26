use serde_json::Value as Json;

use crate::compile::compile;
use crate::resolve::ResolvedConfig;
use crate::types::{McpServerConfig, McpServerType, Permissions, ToolPermissions};

use super::fixtures::*;

fn opencode_patch(r: &ResolvedConfig) -> Json {
    compile(r, "opencode")
        .unwrap()
        .opencode_config_patch
        .expect("opencode must emit a config patch")
}

// ── Basic output ────────────────────────────────────────────────────────────

#[test]
fn opencode_produces_json_patch() {
    let r = resolved(vec![make_server("github")]);
    let patch = opencode_patch(&r);
    assert!(patch.is_object(), "opencode patch must be a JSON object");
}

#[test]
fn opencode_patch_only_for_opencode_provider() {
    let r = resolved(vec![make_server("x")]);
    assert!(compile(&r, "opencode").unwrap().opencode_config_patch.is_some());
    assert!(compile(&r, "claude").unwrap().opencode_config_patch.is_none());
    assert!(compile(&r, "codex").unwrap().opencode_config_patch.is_none());
    assert!(compile(&r, "gemini").unwrap().opencode_config_patch.is_none());
    assert!(compile(&r, "cursor").unwrap().opencode_config_patch.is_none());
}

// ── Model ───────────────────────────────────────────────────────────────────

#[test]
fn opencode_model_emitted() {
    let r = ResolvedConfig {
        model: Some("anthropic/claude-sonnet-4-5".to_string()),
        ..resolved(vec![])
    };
    let patch = opencode_patch(&r);
    assert_eq!(patch["model"].as_str().unwrap(), "anthropic/claude-sonnet-4-5");
}

// ── MCP servers ─────────────────────────────────────────────────────────────

#[test]
fn opencode_mcp_uses_mcp_key_not_mcpservers() {
    let r = resolved(vec![make_server("github")]);
    let patch = opencode_patch(&r);
    assert!(patch.get("mcp").is_some(), "must use 'mcp' key");
    assert!(patch.get("mcpServers").is_none(), "must NOT use 'mcpServers' key");
}

#[test]
fn opencode_mcp_ship_server_first() {
    let r = resolved(vec![make_server("github")]);
    let patch = opencode_patch(&r);
    let mcp = patch["mcp"].as_object().unwrap();
    let keys: Vec<&String> = mcp.keys().collect();
    assert_eq!(keys[0], "ship", "ship server must be first");
}

#[test]
fn opencode_mcp_ship_server_command_includes_serve() {
    // Regression: ship use was generating ["ship", "mcp"] — missing "serve".
    // The correct command is `ship mcp serve`.
    let r = resolved(vec![]);
    let patch = opencode_patch(&r);
    let cmd = patch["mcp"]["ship"]["command"]
        .as_array()
        .expect("ship server must have command array");
    let cmd_strs: Vec<&str> = cmd.iter().map(|v| v.as_str().unwrap()).collect();
    assert_eq!(
        cmd_strs,
        vec!["ship", "mcp", "serve"],
        "opencode ship server command must be [\"ship\", \"mcp\", \"serve\"]"
    );
}

#[test]
fn opencode_mcp_local_format() {
    let mut s = make_server("context7");
    s.command = "npx".to_string();
    s.args = vec!["-y".into(), "@upstash/context7-mcp".into()];
    let r = resolved(vec![s]);
    let patch = opencode_patch(&r);
    let entry = &patch["mcp"]["context7"];
    assert_eq!(entry["type"].as_str().unwrap(), "local", "stdio → type: local");
    // command must be an array [cmd, arg1, arg2, ...]
    let cmd = entry["command"].as_array().unwrap();
    assert_eq!(cmd[0].as_str().unwrap(), "npx");
    assert_eq!(cmd[1].as_str().unwrap(), "-y");
    assert_eq!(cmd[2].as_str().unwrap(), "@upstash/context7-mcp");
}

#[test]
fn opencode_mcp_remote_format() {
    let s = McpServerConfig {
        id: "remote-api".to_string(),
        name: "remote-api".to_string(),
        command: String::new(),
        args: vec![],
        env: Default::default(),
        scope: "project".to_string(),
        server_type: McpServerType::Sse,
        url: Some("https://api.example.com/mcp".to_string()),
        disabled: false,
        timeout_secs: None,
        codex_enabled_tools: vec![],
        codex_disabled_tools: vec![],
        gemini_trust: None,
        gemini_include_tools: vec![],
        gemini_exclude_tools: vec![],
        gemini_timeout_ms: None,
        cursor_env_file: None,
    };
    let r = resolved(vec![s]);
    let patch = opencode_patch(&r);
    let entry = &patch["mcp"]["remote-api"];
    assert_eq!(entry["type"].as_str().unwrap(), "remote");
    assert_eq!(entry["url"].as_str().unwrap(), "https://api.example.com/mcp");
    assert!(entry.get("command").is_none(), "remote must not have command");
}

#[test]
fn opencode_mcp_environment_not_env() {
    let mut s = make_server("with-env");
    s.env.insert("API_KEY".to_string(), "secret".to_string());
    let r = resolved(vec![s]);
    let patch = opencode_patch(&r);
    let entry = &patch["mcp"]["with-env"];
    assert!(entry.get("environment").is_some(), "must use 'environment' key");
    assert!(entry.get("env").is_none(), "must NOT use 'env' key");
    assert_eq!(entry["environment"]["API_KEY"].as_str().unwrap(), "secret");
}

#[test]
fn opencode_mcp_timeout_in_milliseconds() {
    let mut s = make_server("slow");
    s.timeout_secs = Some(30);
    let r = resolved(vec![s]);
    let patch = opencode_patch(&r);
    assert_eq!(
        patch["mcp"]["slow"]["timeout"].as_u64().unwrap(),
        30000,
        "timeout must be in milliseconds"
    );
}

#[test]
fn opencode_mcp_disabled_server_excluded() {
    let mut s = make_server("disabled");
    s.disabled = true;
    let r = resolved(vec![s]);
    let patch = opencode_patch(&r);
    let mcp = patch["mcp"].as_object().unwrap();
    assert!(!mcp.contains_key("disabled"), "disabled servers must be excluded");
}

// ── Permissions ─────────────────────────────────────────────────────────────

#[test]
fn opencode_permissions_simple_deny() {
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions {
                deny: vec!["Bash".to_string()],
                ..Default::default()
            },
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let patch = opencode_patch(&r);
    assert_eq!(
        patch["permission"]["bash"].as_str().unwrap(),
        "deny",
        "Bash deny → bash: deny"
    );
}

#[test]
fn opencode_permissions_granular_bash() {
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions {
                allow: vec!["Bash(git *)".to_string()],
                deny: vec!["Bash(rm *)".to_string()],
                ..Default::default()
            },
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let patch = opencode_patch(&r);
    let bash = &patch["permission"]["bash"];
    assert!(bash.is_object(), "multiple bash patterns → granular object");
    assert_eq!(bash["git *"].as_str().unwrap(), "allow");
    assert_eq!(bash["rm *"].as_str().unwrap(), "deny");
}

#[test]
fn opencode_permissions_read_write_grep() {
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions {
                ask: vec!["Write".to_string(), "Grep".to_string()],
                deny: vec!["Read".to_string()],
                ..Default::default()
            },
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let patch = opencode_patch(&r);
    assert_eq!(patch["permission"]["edit"].as_str().unwrap(), "ask");
    assert_eq!(patch["permission"]["grep"].as_str().unwrap(), "ask");
    assert_eq!(patch["permission"]["read"].as_str().unwrap(), "deny");
}

#[test]
fn opencode_permissions_omitted_when_default() {
    // With servers present (so patch is emitted), default perms should not add permission key
    let r = resolved(vec![make_server("test")]);
    let patch = opencode_patch(&r);
    assert!(
        patch.get("permission").is_none(),
        "default permissions should not emit permission key"
    );
}

// ── settings_extra passthrough ──────────────────────────────────────────────

#[test]
fn opencode_settings_extra_merged() {
    let r = ResolvedConfig {
        opencode_settings_extra: Some(serde_json::json!({
            "theme": "dark",
            "autoupdate": false
        })),
        ..resolved(vec![])
    };
    let patch = opencode_patch(&r);
    assert_eq!(patch["theme"].as_str().unwrap(), "dark");
    assert_eq!(patch["autoupdate"].as_bool().unwrap(), false);
}

#[test]
fn opencode_settings_extra_merged_last() {
    // settings_extra should overwrite Ship-managed keys (escape hatch)
    let r = ResolvedConfig {
        model: Some("anthropic/claude-sonnet-4-5".to_string()),
        opencode_settings_extra: Some(serde_json::json!({ "model": "openai/o3" })),
        ..resolved(vec![])
    };
    let patch = opencode_patch(&r);
    assert_eq!(
        patch["model"].as_str().unwrap(),
        "openai/o3",
        "settings_extra must win over Ship-managed model"
    );
}

// ── Skills dir ──────────────────────────────────────────────────────────────

#[test]
fn opencode_skills_dir_is_opencode() {
    let desc = crate::compile::get_provider("opencode").unwrap();
    assert_eq!(
        desc.skills_dir.base_path().unwrap(),
        ".opencode/skills",
        "OpenCode skills must go to .opencode/skills/"
    );
}

// ── Provider descriptor ─────────────────────────────────────────────────────

#[test]
fn opencode_mcp_key_is_mcp() {
    let desc = crate::compile::get_provider("opencode").unwrap();
    assert_eq!(desc.mcp_key.as_str(), "mcp", "OpenCode uses 'mcp' not 'mcpServers'");
}

#[test]
fn opencode_supports_tool_permissions() {
    let desc = crate::compile::get_provider("opencode").unwrap();
    let flags = desc.feature_flags();
    assert!(flags.supports_tool_permissions, "OpenCode supports tool permissions");
    assert!(flags.supports_mcp, "OpenCode supports MCP");
    assert!(flags.supports_memory, "OpenCode supports memory/context");
}
