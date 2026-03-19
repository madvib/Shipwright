use crate::compile::compile;
use crate::resolve::ResolvedConfig;
use crate::types::HookTrigger;

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

#[test]
fn gemini_model_emitted_in_settings_patch() {
    let r = ResolvedConfig {
        model: Some("gemini-2.5-pro".to_string()),
        ..resolved(vec![])
    };
    let out = compile(&r, "gemini").unwrap();
    let patch = out.gemini_settings_patch.expect("model should trigger gemini patch");
    assert_eq!(patch["model"], "gemini-2.5-pro");
}

#[test]
fn gemini_no_model_no_hooks_no_patch() {
    let r = resolved(vec![]);
    let out = compile(&r, "gemini").unwrap();
    assert!(out.gemini_settings_patch.is_none());
}
