//! Realistic agent fixture tests — multi-feature configs across all providers.

use super::fixtures::{make_hook, make_rule, make_server, make_skill, resolved};
use crate::compile::compile;
use crate::resolve::ResolvedConfig;
use crate::types::{AgentLimits, HookTrigger, Permissions, ToolPermissions};
use std::collections::HashMap;

// ── Fixtures ─────────────────────────────────────────────────────────────────

fn full_agent() -> ResolvedConfig {
    let mut github = make_server("github");
    github
        .env
        .insert("GITHUB_TOKEN".into(), "${GITHUB_TOKEN}".into());
    github.timeout_secs = Some(30);

    ResolvedConfig {
        providers: vec![
            "claude".into(),
            "gemini".into(),
            "codex".into(),
            "cursor".into(),
        ],
        model: Some("claude-opus-4-6".into()),
        max_cost_per_session: Some(10.0),
        max_turns: Some(50),
        mcp_servers: vec![github, make_server("postgres")],
        skills: vec![make_skill("code-review"), make_skill("test-generator")],
        rules: vec![
            make_rule(
                "01-style.md",
                "# Code Style\n\n- Use explicit type annotations on public APIs.",
            ),
            make_rule(
                "02-safety.md",
                "# Safety\n\n- Never commit secrets or API keys.",
            ),
        ],
        permissions: Permissions {
            tools: ToolPermissions {
                allow: vec!["*".into()],
                ask: vec!["Bash(rm -rf*)".into(), "Bash(*--force*)".into()],
                deny: vec![
                    "Bash(git push --force*)".into(),
                    "Bash(*npm publish*)".into(),
                    "mcp__*__delete*".into(),
                ],
            },
            agent: AgentLimits {
                max_cost_per_session: Some(10.0),
                max_turns: Some(50),
                require_confirmation: vec![],
            },
            default_mode: Some("acceptEdits".into()),
            ..Default::default()
        },
        hooks: vec![
            make_hook(HookTrigger::Stop, "ship end-session --summary", None),
            make_hook(
                HookTrigger::SubagentStop,
                "ship log 'subagent completed'",
                None,
            ),
        ],
        ..resolved(vec![])
    }
}

fn empty_agent() -> ResolvedConfig {
    resolved(vec![])
}

// ── Test 1: Full agent compiles correctly for Claude ─────────────────────────

#[test]
fn realistic_agent_context_contains_both_rules() {
    let out = compile(&full_agent(), "claude").unwrap();
    let ctx = out
        .context_content
        .expect("rules must produce a context file");
    assert!(
        ctx.contains("Use explicit type annotations"),
        "missing style rule"
    );
    assert!(ctx.contains("Never commit secrets"), "missing safety rule");
}

#[test]
fn realistic_agent_emits_both_skill_files() {
    let out = compile(&full_agent(), "claude").unwrap();
    assert!(
        out.skill_files
            .contains_key(".claude/skills/code-review/SKILL.md")
    );
    assert!(
        out.skill_files
            .contains_key(".claude/skills/test-generator/SKILL.md")
    );
    let review = &out.skill_files[".claude/skills/code-review/SKILL.md"];
    assert!(
        review.contains("code-review"),
        "skill content must include id"
    );
}

#[test]
fn realistic_agent_settings_has_hooks() {
    let patch = compile(&full_agent(), "claude")
        .unwrap()
        .claude_settings_patch
        .expect("agent with hooks must emit settings");
    let stop = patch["hooks"]["Stop"].as_array().unwrap();
    assert_eq!(stop.len(), 1);
    let stop_inner = stop[0]["hooks"].as_array().unwrap();
    assert_eq!(stop_inner[0]["command"], "ship end-session --summary");
    let sub = patch["hooks"]["SubagentStop"].as_array().unwrap();
    assert_eq!(sub.len(), 1);
    let sub_inner = sub[0]["hooks"].as_array().unwrap();
    assert_eq!(sub_inner[0]["command"], "ship log 'subagent completed'");
}

#[test]
fn realistic_agent_settings_has_deny_and_default_mode() {
    let patch = compile(&full_agent(), "claude")
        .unwrap()
        .claude_settings_patch
        .unwrap();
    let perms = &patch["permissions"];
    let deny = perms["deny"].as_array().unwrap();
    assert!(deny.iter().any(|v| v == "Bash(git push --force*)"));
    assert!(deny.iter().any(|v| v == "mcp__*__delete*"));
    assert_eq!(perms["defaultMode"], "acceptEdits");
}

#[test]
fn realistic_agent_settings_has_ask_and_limits() {
    let patch = compile(&full_agent(), "claude")
        .unwrap()
        .claude_settings_patch
        .unwrap();
    let ask = patch["permissions"]["ask"].as_array().unwrap();
    assert!(ask.iter().any(|v| v == "Bash(rm -rf*)"));
    assert_eq!(patch["maxCostPerSession"], 10.0);
    assert_eq!(patch["maxTurns"], 50);
}

#[test]
fn realistic_agent_mcp_has_both_servers_plus_ship() {
    let mcp = compile(&full_agent(), "claude").unwrap().mcp_servers;
    let obj = mcp.as_object().unwrap();
    assert!(obj.contains_key("ship"), "ship server always injected");
    assert!(obj.contains_key("github"));
    assert!(obj.contains_key("postgres"));
    assert_eq!(obj["github"]["command"], "npx");
    assert!(obj["github"]["env"]["GITHUB_TOKEN"].is_string());
}

// ── Test 2: All providers consistent ─────────────────────────────────────────

#[test]
fn realistic_agent_all_providers_have_context_with_rules() {
    let r = full_agent();
    for provider in &["claude", "gemini", "codex"] {
        let ctx = compile(&r, provider)
            .unwrap()
            .context_content
            .unwrap_or_else(|| panic!("{provider} must have context_content"));
        assert!(ctx.contains("Use explicit type annotations"), "{provider}");
        assert!(ctx.contains("Never commit secrets"), "{provider}");
    }
    // Cursor uses rule_files instead
    let cursor = compile(&r, "cursor").unwrap();
    assert!(cursor.context_content.is_none());
    assert!(
        cursor
            .rule_files
            .values()
            .any(|c| c.contains("Use explicit type annotations")),
        "cursor rule_files must contain style rule"
    );
}

#[test]
fn realistic_agent_all_providers_have_skill_files() {
    let r = full_agent();
    let paths: HashMap<&str, &str> = [
        ("claude", ".claude/skills"),
        ("gemini", ".agents/skills"),
        ("codex", ".agents/skills"),
        ("cursor", ".cursor/skills"),
    ]
    .into();
    for (provider, base) in &paths {
        let out = compile(&r, provider).unwrap();
        for id in &["code-review", "test-generator"] {
            let path = format!("{base}/{id}/SKILL.md");
            assert!(out.skill_files.contains_key(&path), "{provider}: {path}");
        }
    }
}

#[test]
fn realistic_agent_all_providers_have_mcp_servers() {
    let r = full_agent();
    for provider in &["claude", "gemini", "codex", "cursor"] {
        let obj = compile(&r, provider).unwrap().mcp_servers;
        let mcp = obj.as_object().unwrap();
        assert!(mcp.contains_key("ship"), "{provider}: ship");
        assert!(mcp.contains_key("github"), "{provider}: github");
        assert!(mcp.contains_key("postgres"), "{provider}: postgres");
    }
}

#[test]
fn realistic_agent_provider_patches_are_exclusive() {
    let r = full_agent();
    assert!(
        compile(&r, "claude")
            .unwrap()
            .claude_settings_patch
            .is_some()
    );
    assert!(
        compile(&r, "gemini")
            .unwrap()
            .claude_settings_patch
            .is_none()
    );
    assert!(
        compile(&r, "codex")
            .unwrap()
            .claude_settings_patch
            .is_none()
    );
    assert!(
        compile(&r, "cursor")
            .unwrap()
            .claude_settings_patch
            .is_none()
    );
    assert!(compile(&r, "codex").unwrap().codex_config_patch.is_some());
    assert!(compile(&r, "claude").unwrap().codex_config_patch.is_none());
    assert!(compile(&r, "gemini").unwrap().gemini_policy_patch.is_some());
    assert!(compile(&r, "claude").unwrap().gemini_policy_patch.is_none());
}

// ── Test 3: Deny-only must not leak allow ────────────────────────────────────

fn deny_only_config() -> ResolvedConfig {
    ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions {
                allow: vec![],
                ask: vec![],
                deny: vec!["Bash(rm -rf *)".into(), "mcp__*__delete*".into()],
            },
            ..Default::default()
        },
        ..empty_agent()
    }
}

#[test]
fn permission_deny_only_no_allow_leak_claude() {
    let patch = compile(&deny_only_config(), "claude")
        .unwrap()
        .claude_settings_patch
        .unwrap();
    let p = &patch["permissions"];
    assert!(
        p.get("allow").is_none() || p["allow"].as_array().map(|a| a.is_empty()).unwrap_or(true),
        "deny-only must not leak allow in claude"
    );
    assert_eq!(p["deny"].as_array().unwrap().len(), 2);
}

#[test]
fn permission_deny_only_no_allow_leak_cursor() {
    let patch = compile(&deny_only_config(), "cursor")
        .unwrap()
        .cursor_cli_permissions
        .unwrap();
    assert!(
        patch["permissions"].get("allow").is_none(),
        "deny-only must not leak allow in cursor"
    );
    assert!(!patch["permissions"]["deny"].as_array().unwrap().is_empty());
}

#[test]
fn permission_deny_only_no_allow_leak_gemini() {
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions {
                allow: vec![],
                ask: vec![],
                deny: vec!["Bash(rm -rf *)".into(), "Write".into()],
            },
            ..Default::default()
        },
        ..empty_agent()
    };
    let toml_str = compile(&r, "gemini").unwrap().gemini_policy_patch.unwrap();
    assert!(
        !toml_str.contains(r#"decision = "allow""#),
        "deny-only must not leak allow in gemini"
    );
    assert!(toml_str.contains(r#"decision = "deny""#));
}

// ── Test 4: Empty agent compiles without errors ──────────────────────────────

#[test]
fn empty_agent_compiles_for_all_providers() {
    let r = empty_agent();
    for provider in &["claude", "gemini", "codex", "cursor"] {
        let out = compile(&r, provider).unwrap();
        assert!(
            out.mcp_servers.as_object().unwrap().contains_key("ship"),
            "{provider}: ship always"
        );
        assert!(out.skill_files.is_empty(), "{provider}: no skill files");
    }
}

// ── Test 5: Hook emission across providers ───────────────────────────────────

#[test]
fn both_stop_hooks_emit_to_claude_settings() {
    let r = ResolvedConfig {
        hooks: vec![
            make_hook(HookTrigger::Stop, "ship end-session --summary", None),
            make_hook(HookTrigger::SubagentStop, "ship log 'subagent done'", None),
        ],
        ..empty_agent()
    };
    let patch = compile(&r, "claude")
        .unwrap()
        .claude_settings_patch
        .unwrap();
    let stop = patch["hooks"]["Stop"].as_array().unwrap();
    assert_eq!(stop.len(), 1);
    let stop_inner = stop[0]["hooks"].as_array().unwrap();
    assert_eq!(stop_inner[0]["command"], "ship end-session --summary");
    let sub = patch["hooks"]["SubagentStop"].as_array().unwrap();
    assert_eq!(sub.len(), 1);
    let sub_inner = sub[0]["hooks"].as_array().unwrap();
    assert_eq!(sub_inner[0]["command"], "ship log 'subagent done'");
}

#[test]
fn stop_hook_maps_to_gemini_session_end() {
    let r = ResolvedConfig {
        hooks: vec![make_hook(HookTrigger::Stop, "ship end-session", None)],
        ..empty_agent()
    };
    let patch = compile(&r, "gemini")
        .unwrap()
        .gemini_settings_patch
        .unwrap();
    assert!(patch["hooks"]["SessionEnd"].is_array());
}

#[test]
fn stop_hook_maps_to_cursor_session_end() {
    let r = ResolvedConfig {
        hooks: vec![make_hook(HookTrigger::Stop, "ship end-session", None)],
        ..empty_agent()
    };
    let patch = compile(&r, "cursor").unwrap().cursor_hooks_patch.unwrap();
    assert!(patch["sessionEnd"].is_array());
}

#[test]
fn subagent_stop_dropped_for_non_claude_providers() {
    let r = ResolvedConfig {
        hooks: vec![make_hook(
            HookTrigger::SubagentStop,
            "ship log subagent",
            None,
        )],
        ..empty_agent()
    };
    assert!(
        compile(&r, "gemini")
            .unwrap()
            .gemini_settings_patch
            .is_none()
    );
    assert!(compile(&r, "cursor").unwrap().cursor_hooks_patch.is_none());
}
