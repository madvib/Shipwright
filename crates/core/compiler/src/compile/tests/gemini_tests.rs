use crate::compile::compile;
use crate::resolve::ResolvedConfig;
use crate::types::{HookConfig, HookTrigger};

use super::fixtures::*;

// ── Gemini hooks ──────────────────────────────────────────────────────────────

#[test]
fn gemini_hooks_pre_tool_maps_to_before_tool() {
    let r = ResolvedConfig {
        hooks: vec![make_hook(HookTrigger::PreToolUse, "ship check", Some("Bash"))],
        ..resolved(vec![])
    };
    let out = compile(&r, "gemini").unwrap();
    let patch = out.gemini_settings_patch.expect("gemini must emit hooks patch");
    assert!(patch["hooks"]["BeforeTool"].is_array());
    assert_eq!(patch["hooks"]["BeforeTool"][0]["matcher"], "Bash");
    assert_eq!(patch["hooks"]["BeforeTool"][0]["hooks"][0]["command"], "ship check");
}

#[test]
fn gemini_hooks_trigger_mapping() {
    let hooks = vec![
        make_hook(HookTrigger::PreToolUse,  "cmd-pre",    None),
        make_hook(HookTrigger::PostToolUse, "cmd-post",   None),
        make_hook(HookTrigger::Stop,        "cmd-stop",   None),
        make_hook(HookTrigger::PreCompact,  "cmd-compact",None),
        make_hook(HookTrigger::Notification,"cmd-notify", None),
    ];
    let r = ResolvedConfig { hooks, ..resolved(vec![]) };
    let out = compile(&r, "gemini").unwrap();
    let patch = out.gemini_settings_patch.unwrap();
    let h = &patch["hooks"];
    assert!(h["BeforeTool"].is_array(),   "PreToolUse → BeforeTool");
    assert!(h["AfterTool"].is_array(),    "PostToolUse → AfterTool");
    assert!(h["SessionEnd"].is_array(),   "Stop → SessionEnd");
    assert!(h["PreCompress"].is_array(),  "PreCompact → PreCompress");
    assert!(h["Notification"].is_array(), "Notification → Notification");
    assert!(h.get("SubagentStop").is_none());
}

#[test]
fn gemini_hooks_subagent_stop_dropped() {
    let r = ResolvedConfig {
        hooks: vec![make_hook(HookTrigger::SubagentStop, "cmd", None)],
        ..resolved(vec![])
    };
    let out = compile(&r, "gemini").unwrap();
    assert!(out.gemini_settings_patch.is_none());
}

#[test]
fn gemini_no_hooks_no_patch() {
    let out = compile(&resolved(vec![]), "gemini").unwrap();
    assert!(out.gemini_settings_patch.is_none());
}

#[test]
fn gemini_hooks_not_emitted_for_other_providers() {
    let r = ResolvedConfig {
        hooks: vec![make_hook(HookTrigger::PreToolUse, "cmd", None)],
        ..resolved(vec![])
    };
    assert!(compile(&r, "claude").unwrap().gemini_settings_patch.is_none());
    assert!(compile(&r, "codex").unwrap().gemini_settings_patch.is_none());
    assert!(compile(&r, "cursor").unwrap().gemini_settings_patch.is_none());
}

// ── Model selection for Gemini ────────────────────────────────────────────────

/// Gemini emits model as `{ "model": { "name": "..." } }` per its settings schema.
#[test]
fn gemini_model_emitted_in_settings_patch() {
    let r = ResolvedConfig {
        model: Some("gemini-2.5-pro".to_string()),
        ..resolved(vec![])
    };
    let out = compile(&r, "gemini").unwrap();
    let patch = out.gemini_settings_patch.expect("model should trigger gemini patch");
    assert_eq!(patch["model"]["name"], "gemini-2.5-pro");
}

#[test]
fn gemini_no_model_no_hooks_no_patch() {
    let r = resolved(vec![]);
    let out = compile(&r, "gemini").unwrap();
    assert!(out.gemini_settings_patch.is_none());
}

// ── Phase 4: new Gemini fields ────────────────────────────────────────────────

/// Model from library flows through to gemini patch as `model.name`.
#[test]
fn library_model_flows_to_gemini_patch() {
    use crate::resolve::{ProjectLibrary, resolve_library};
    let library = ProjectLibrary {
        model: Some("gemini-2.5-flash".to_string()),
        ..Default::default()
    };
    let resolved = resolve_library(&library, None, None);
    let out = compile(&resolved, "gemini").unwrap();
    let patch = out.gemini_settings_patch.expect("model must trigger gemini patch");
    assert_eq!(patch["model"]["name"], "gemini-2.5-flash");
}

#[test]
fn gemini_default_approval_mode_translate() {
    let cases = [
        ("default", "suggest"),
        ("auto_edit", "auto-edit"),
        ("plan", "yolo"),
    ];
    for (input, expected) in cases {
        let r = ResolvedConfig {
            gemini_default_approval_mode: Some(input.to_string()),
            ..resolved(vec![])
        };
        let out = compile(&r, "gemini").unwrap();
        let patch = out.gemini_settings_patch.expect("approval mode must trigger patch");
        assert_eq!(patch["general"]["defaultApprovalMode"], expected, "input '{input}'");
    }
}

#[test]
fn gemini_max_session_turns_emitted() {
    let r = ResolvedConfig {
        gemini_max_session_turns: Some(50),
        ..resolved(vec![])
    };
    let out = compile(&r, "gemini").unwrap();
    let patch = out.gemini_settings_patch.expect("maxSessionTurns must trigger patch");
    assert_eq!(patch["general"]["maxSessionTurns"], 50);
}

#[test]
fn gemini_security_flags_emitted() {
    let r = ResolvedConfig {
        gemini_disable_yolo_mode: Some(true),
        gemini_disable_always_allow: Some(false),
        ..resolved(vec![])
    };
    let out = compile(&r, "gemini").unwrap();
    let patch = out.gemini_settings_patch.expect("security flags must trigger patch");
    assert_eq!(patch["security"]["disableYoloMode"], true);
    assert_eq!(patch["security"]["disableAlwaysAllow"], false);
}

#[test]
fn gemini_tools_sandbox_emitted() {
    let r = ResolvedConfig {
        gemini_tools_sandbox: Some("docker".to_string()),
        ..resolved(vec![])
    };
    let out = compile(&r, "gemini").unwrap();
    let patch = out.gemini_settings_patch.expect("tools.sandbox must trigger patch");
    assert_eq!(patch["tools"]["sandbox"], "docker");
}

#[test]
fn gemini_settings_extra_merged_last() {
    let r = ResolvedConfig {
        gemini_settings_extra: Some(serde_json::json!({ "telemetry": { "enabled": false } })),
        ..resolved(vec![])
    };
    let out = compile(&r, "gemini").unwrap();
    let patch = out.gemini_settings_patch.expect("settings_extra must trigger patch");
    assert_eq!(patch["telemetry"]["enabled"], false);
}

#[test]
fn gemini_raw_event_bypasses_trigger_mapping() {
    let hook = HookConfig {
        id: "raw-hook".to_string(),
        trigger: HookTrigger::Stop,
        matcher: None,
        command: "ship log".to_string(),
        cursor_event: None,
        gemini_event: Some("SessionStart".to_string()),
    };
    let r = ResolvedConfig {
        hooks: vec![hook],
        ..resolved(vec![])
    };
    let out = compile(&r, "gemini").unwrap();
    let patch = out.gemini_settings_patch.expect("raw event must trigger patch");
    // SessionStart not normally mapped, but raw event bypasses that.
    assert!(patch["hooks"]["SessionStart"].is_array(), "raw gemini_event must emit under SessionStart");
    // SessionEnd must NOT have an entry (trigger mapping bypassed).
    assert!(patch["hooks"].get("SessionEnd").is_none() || patch["hooks"]["SessionEnd"].as_array().map_or(true, |a| a.is_empty()));
}

#[test]
fn gemini_mcp_server_trust_emitted() {
    let mut s = make_server("trusted-server");
    s.gemini_trust = Some(true);
    let r = resolved(vec![s]);
    let out = compile(&r, "gemini").unwrap();
    assert_eq!(out.mcp_servers["trusted-server"]["trust"], true);
}

#[test]
fn gemini_mcp_server_include_exclude_tools_emitted() {
    let mut s = make_server("selective-server");
    s.gemini_include_tools = vec!["read".to_string(), "write".to_string()];
    s.gemini_exclude_tools = vec!["delete".to_string()];
    let r = resolved(vec![s]);
    let out = compile(&r, "gemini").unwrap();
    let entry = &out.mcp_servers["selective-server"];
    let include = entry["includeTools"].as_array().unwrap();
    assert_eq!(include.len(), 2);
    let exclude = entry["excludeTools"].as_array().unwrap();
    assert_eq!(exclude[0], "delete");
}

#[test]
fn gemini_mcp_server_timeout_emitted() {
    let mut s = make_server("slow-server");
    s.gemini_timeout_ms = Some(5000);
    let r = resolved(vec![s]);
    let out = compile(&r, "gemini").unwrap();
    assert_eq!(out.mcp_servers["slow-server"]["timeout"], 5000);
}
