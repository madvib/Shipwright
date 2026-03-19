use crate::compile::compile;
use crate::resolve::ResolvedConfig;
use crate::types::{HookTrigger, Permissions, Rule, ToolPermissions};

use super::fixtures::*;

// ── Idempotency ───────────────────────────────────────────────────────────────

/// Compiling the same input twice must produce identical output.
#[test]
fn compile_is_idempotent() {
    use crate::types::AgentLimits;
    let r = ResolvedConfig {
        mcp_servers: vec![make_server("github"), make_server("linear")],
        skills: vec![make_skill("rust-expert")],
        rules: vec![Rule { file_name: "style.md".into(), content: "Use explicit types.".into(), always_apply: true, globs: vec![], description: None }],
        permissions: Permissions {
            tools: ToolPermissions {
                allow: vec!["*".to_string()],
                deny: vec!["Bash(rm -rf *)".to_string()],
                ..Default::default()
            },
            agent: AgentLimits { max_cost_per_session: Some(2.50), ..Default::default() },
            ..Default::default()
        },
        hooks: vec![make_hook(HookTrigger::PostToolUse, "ship log", Some("*"))],
        ..resolved(vec![])
    };

    let first = compile(&r, "claude").unwrap();
    let second = compile(&r, "claude").unwrap();

    assert_eq!(
        serde_json::to_string(&first.mcp_servers).unwrap(),
        serde_json::to_string(&second.mcp_servers).unwrap(),
        "MCP output must be identical across compilations"
    );
    assert_eq!(first.context_content, second.context_content);
    assert_eq!(first.skill_files, second.skill_files);
    assert_eq!(
        serde_json::to_string(&first.claude_settings_patch).unwrap(),
        serde_json::to_string(&second.claude_settings_patch).unwrap(),
        "Settings patch must be identical across compilations"
    );
}

/// Every provider must produce identical output when compiled twice.
#[test]
fn all_providers_are_idempotent() {
    let r = ResolvedConfig {
        mcp_servers: vec![make_server("github"), make_server("linear")],
        skills: vec![{ let mut s = make_skill("workflow"); s.content = "Do the thing.".into(); s }],
        rules: vec![Rule { file_name: "style.md".into(), content: "Keep it clean.".into(), always_apply: true, globs: vec![], description: None }],
        hooks: vec![make_hook(HookTrigger::PreToolUse, "ship check", Some("Bash"))],
        ..resolved(vec![])
    };
    for provider in &["claude", "gemini", "codex", "cursor"] {
        let a = compile(&r, provider).unwrap();
        let b = compile(&r, provider).unwrap();
        assert_eq!(serde_json::to_string(&a.mcp_servers).unwrap(), serde_json::to_string(&b.mcp_servers).unwrap(), "{provider}: mcp_servers not idempotent");
        assert_eq!(a.context_content, b.context_content, "{provider}: context not idempotent");
        assert_eq!(a.skill_files, b.skill_files, "{provider}: skill_files not idempotent");
        assert_eq!(serde_json::to_string(&a.claude_settings_patch).unwrap(), serde_json::to_string(&b.claude_settings_patch).unwrap(), "{provider}: claude_settings_patch not idempotent");
        assert_eq!(a.codex_config_patch, b.codex_config_patch, "{provider}: codex_config_patch not idempotent");
        assert_eq!(serde_json::to_string(&a.gemini_settings_patch).unwrap(), serde_json::to_string(&b.gemini_settings_patch).unwrap(), "{provider}: gemini_settings_patch not idempotent");
        assert_eq!(serde_json::to_string(&a.cursor_hooks_patch).unwrap(), serde_json::to_string(&b.cursor_hooks_patch).unwrap(), "{provider}: cursor_hooks_patch not idempotent");
        assert_eq!(serde_json::to_string(&a.cursor_cli_permissions).unwrap(), serde_json::to_string(&b.cursor_cli_permissions).unwrap(), "{provider}: cursor_cli_permissions not idempotent");
        assert_eq!(a.rule_files, b.rule_files, "{provider}: rule_files not idempotent");
        assert_eq!(a.gemini_policy_patch, b.gemini_policy_patch, "{provider}: gemini_policy_patch not idempotent");
    }
}

/// All four providers compile the same skill set correctly.
#[test]
fn all_providers_emit_skills() {
    let mut skill = make_skill("refactor");
    skill.content = "Refactor carefully.".into();
    let r = ResolvedConfig { skills: vec![skill], ..resolved(vec![]) };
    assert!(compile(&r, "claude").unwrap().skill_files.contains_key(".claude/skills/refactor/SKILL.md"));
    assert!(compile(&r, "gemini").unwrap().skill_files.contains_key(".agents/skills/refactor/SKILL.md"));
    assert!(compile(&r, "codex").unwrap().skill_files.contains_key(".agents/skills/refactor/SKILL.md"));
    assert!(compile(&r, "cursor").unwrap().skill_files.contains_key(".cursor/skills/refactor/SKILL.md"));
}

/// Claude, Gemini, Codex emit a context file when rules are present.
/// Cursor uses per-file .mdc rules instead.
#[test]
fn all_providers_emit_context_file() {
    let r = ResolvedConfig {
        rules: vec![Rule { file_name: "style.md".into(), content: "Write clean code.".into(), always_apply: true, globs: vec![], description: None }],
        ..resolved(vec![])
    };
    assert!(compile(&r, "claude").unwrap().context_content.is_some(), "claude needs CLAUDE.md");
    assert!(compile(&r, "gemini").unwrap().context_content.is_some(), "gemini needs GEMINI.md");
    assert!(compile(&r, "codex").unwrap().context_content.is_some(), "codex needs AGENTS.md");
    assert!(compile(&r, "cursor").unwrap().context_content.is_none(), "cursor uses rule_files");
    assert!(!compile(&r, "cursor").unwrap().rule_files.is_empty(), "cursor must populate rule_files");
}

/// No provider emits a patch output it doesn't own.
#[test]
fn patch_outputs_are_provider_exclusive() {
    let r = ResolvedConfig {
        rules: vec![Rule { file_name: "r.md".into(), content: "x".into(), always_apply: true, globs: vec![], description: None }],
        hooks: vec![make_hook(HookTrigger::PreToolUse, "cmd", None)],
        permissions: Permissions {
            tools: ToolPermissions { deny: vec!["Bash(rm -rf *)".into()], ..Default::default() },
            ..Default::default()
        },
        ..resolved(vec![])
    };
    // claude_settings_patch: claude only
    assert!(compile(&r, "claude").unwrap().claude_settings_patch.is_some());
    for p in &["gemini", "codex", "cursor"] {
        assert!(compile(&r, p).unwrap().claude_settings_patch.is_none(), "{p} must not get claude patch");
    }
    // codex_config_patch: codex only
    assert!(compile(&r, "codex").unwrap().codex_config_patch.is_some());
    for p in &["claude", "gemini", "cursor"] {
        assert!(compile(&r, p).unwrap().codex_config_patch.is_none(), "{p} must not get codex patch");
    }
    // gemini_settings_patch: gemini only
    assert!(compile(&r, "gemini").unwrap().gemini_settings_patch.is_some());
    for p in &["claude", "codex", "cursor"] {
        assert!(compile(&r, p).unwrap().gemini_settings_patch.is_none(), "{p} must not get gemini patch");
    }
    // cursor_hooks_patch: cursor only
    assert!(compile(&r, "cursor").unwrap().cursor_hooks_patch.is_some());
    for p in &["claude", "gemini", "codex"] {
        assert!(compile(&r, p).unwrap().cursor_hooks_patch.is_none(), "{p} must not get cursor patch");
    }
    // cursor_cli_permissions: cursor only
    assert!(compile(&r, "cursor").unwrap().cursor_cli_permissions.is_some());
    for p in &["claude", "gemini", "codex"] {
        assert!(compile(&r, p).unwrap().cursor_cli_permissions.is_none(), "{p} must not get cursor cli permissions");
    }
    // rule_files: cursor only
    assert!(!compile(&r, "cursor").unwrap().rule_files.is_empty(), "cursor must have rule_files");
    for p in &["claude", "gemini", "codex"] {
        assert!(compile(&r, p).unwrap().rule_files.is_empty(), "{p} must not get rule_files");
    }
}

/// compile_library_all via ProjectLibrary round-trip.
#[test]
fn library_round_trip_via_resolve() {
    use crate::resolve::{ProjectLibrary, resolve_library};
    use crate::types::AgentLimits;

    let library = ProjectLibrary {
        mcp_servers: vec![make_server("github")],
        skills: vec![make_skill("workflow")],
        rules: vec![Rule { file_name: "style.md".into(), content: "Keep it clean.".into(), always_apply: true, globs: vec![], description: None }],
        permissions: Permissions {
            tools: ToolPermissions {
                allow: vec!["*".to_string()],
                deny: vec!["Bash(sudo *)".to_string()],
                ..Default::default()
            },
            ..Default::default()
        },
        hooks: vec![make_hook(HookTrigger::PreToolUse, "ship check", Some("Bash"))],
        ..Default::default()
    };

    let resolved = resolve_library(&library, None, None);
    assert_eq!(resolved.hooks.len(), 1, "hooks must survive resolve_library");

    let out = compile(&resolved, "claude").unwrap();

    assert!(out.mcp_servers["github"].is_object());
    assert!(out.context_content.unwrap().contains("Keep it clean."));
    assert!(out.skill_files.contains_key(".claude/skills/workflow/SKILL.md"));

    let patch = out.claude_settings_patch.unwrap();
    assert_eq!(patch["permissions"]["deny"][0], "Bash(sudo *)");
    assert!(patch["hooks"]["PreToolUse"].is_array());
    assert_eq!(patch["hooks"]["PreToolUse"][0]["matcher"], "Bash");
    assert_eq!(out.mcp_config_path.as_deref(), Some(".mcp.json"));
}
