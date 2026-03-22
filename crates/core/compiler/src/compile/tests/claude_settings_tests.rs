use crate::compile::compile;
use crate::resolve::ResolvedConfig;
use crate::types::Permissions;

use super::fixtures::*;

/// Ship always emits autoMemoryEnabled: false. When no other settings are
/// configured, the patch contains only that field.
fn assert_memory_only(patch: &Option<serde_json::Value>) {
    let p = patch
        .as_ref()
        .expect("patch must exist (autoMemoryEnabled)");
    assert_eq!(p["autoMemoryEnabled"], false);
    assert_eq!(
        p.as_object().unwrap().len(),
        1,
        "expected only autoMemoryEnabled"
    );
}

// ── defaultMode ───────────────────────────────────────────────────────────────

#[test]
fn default_mode_compiles_to_permissions_default_mode() {
    let r = ResolvedConfig {
        permissions: Permissions {
            default_mode: Some("plan".to_string()),
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "claude").unwrap();
    let patch = out
        .claude_settings_patch
        .expect("default_mode must emit a patch");
    assert_eq!(patch["permissions"]["defaultMode"], "plan");
}

#[test]
fn default_mode_none_omitted_from_patch() {
    let out = compile(&resolved(vec![]), "claude").unwrap();
    assert_memory_only(&out.claude_settings_patch);
}

// ── model ─────────────────────────────────────────────────────────────────────

#[test]
fn model_compiles_to_settings_patch() {
    let r = ResolvedConfig {
        model: Some("claude-opus-4-6".to_string()),
        ..resolved(vec![])
    };
    let out = compile(&r, "claude").unwrap();
    let patch = out.claude_settings_patch.expect("model must emit a patch");
    assert_eq!(patch["model"], "claude-opus-4-6");
}

#[test]
fn model_none_omits_field() {
    let r = ResolvedConfig {
        model: None,
        ..resolved(vec![])
    };
    let out = compile(&r, "claude").unwrap();
    assert_memory_only(&out.claude_settings_patch);
}

#[test]
fn model_only_for_claude_not_other_providers() {
    let r = ResolvedConfig {
        model: Some("claude-opus-4-6".to_string()),
        ..resolved(vec![])
    };
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
}

// ── additionalDirectories ─────────────────────────────────────────────────────

#[test]
fn additional_directories_emitted_when_set() {
    let r = ResolvedConfig {
        permissions: Permissions {
            additional_directories: vec!["/tmp/project".to_string()],
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "claude").unwrap();
    let patch = out
        .claude_settings_patch
        .expect("additionalDirectories must emit a patch");
    let dirs = patch["permissions"]["additionalDirectories"]
        .as_array()
        .unwrap();
    assert_eq!(dirs.len(), 1);
    assert_eq!(dirs[0], "/tmp/project");
}

#[test]
fn additional_directories_empty_omitted() {
    let r = ResolvedConfig {
        permissions: Permissions {
            additional_directories: vec![],
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "claude").unwrap();
    assert_memory_only(&out.claude_settings_patch);
}

// ── Env vars ──────────────────────────────────────────────────────────────────

#[test]
fn env_vars_emitted_in_claude_settings_patch() {
    let mut env = std::collections::HashMap::new();
    env.insert(
        "ANTHROPIC_MODEL".to_string(),
        "claude-sonnet-4-20250514".to_string(),
    );
    env.insert("DEBUG".to_string(), "true".to_string());
    let r = ResolvedConfig {
        env,
        ..resolved(vec![])
    };
    let out = compile(&r, "claude").unwrap();
    let patch = out
        .claude_settings_patch
        .expect("env should trigger settings patch");
    let env_obj = patch.get("env").expect("patch should have env key");
    assert_eq!(env_obj["ANTHROPIC_MODEL"], "claude-sonnet-4-20250514");
    assert_eq!(env_obj["DEBUG"], "true");
}

#[test]
fn empty_env_no_settings_patch() {
    let out = compile(&resolved(vec![]), "claude").unwrap();
    assert_memory_only(&out.claude_settings_patch);
}

#[test]
fn env_only_for_claude_not_other_providers() {
    let mut env = std::collections::HashMap::new();
    env.insert("FOO".to_string(), "bar".to_string());
    let r = ResolvedConfig {
        env,
        ..resolved(vec![])
    };
    for provider in &["gemini", "codex", "cursor"] {
        let out = compile(&r, provider).unwrap();
        assert!(
            out.claude_settings_patch.is_none(),
            "provider {} should not have claude_settings_patch",
            provider
        );
    }
}

// ── Available models ──────────────────────────────────────────────────────────

#[test]
fn available_models_emitted_in_claude_settings_patch() {
    let r = ResolvedConfig {
        available_models: vec![
            "claude-sonnet-4-20250514".into(),
            "claude-haiku-3-5-20241022".into(),
        ],
        ..resolved(vec![])
    };
    let out = compile(&r, "claude").unwrap();
    let patch = out
        .claude_settings_patch
        .expect("availableModels should trigger patch");
    let models = patch
        .get("availableModels")
        .expect("patch should have availableModels");
    let arr = models.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0], "claude-sonnet-4-20250514");
}

#[test]
fn empty_available_models_no_patch() {
    let out = compile(&resolved(vec![]), "claude").unwrap();
    assert_memory_only(&out.claude_settings_patch);
}

// ── Agent profiles & team agents ──────────────────────────────────────────────

#[test]
fn agent_profiles_compiled_into_agent_files() {
    use crate::types::agent_profile::*;

    let profile = AgentProfile {
        profile: ProfileMeta {
            id: "reviewer".to_string(),
            name: "Code Reviewer".to_string(),
            version: None,
            description: Some("Reviews code".to_string()),
            providers: vec!["claude".to_string()],
        },
        skills: SkillRefs::default(),
        mcp: McpRefs::default(),
        plugins: PluginRefs::default(),
        permissions: ProfilePermissions::default(),
        rules: ProfileRules {
            inline: Some("Review carefully.".to_string()),
        },
        provider_settings: Default::default(),
    };

    let r = ResolvedConfig {
        agent_profiles: vec![profile],
        ..resolved(vec![])
    };
    let out = compile(&r, "claude").unwrap();
    assert!(out.agent_files.contains_key(".claude/agents/reviewer.md"));
    let content = &out.agent_files[".claude/agents/reviewer.md"];
    assert!(
        content.contains("name: reviewer"),
        "name field must use profile id; got:\n{content}"
    );
    assert!(
        content.contains("Reviews code"),
        "description must appear; got:\n{content}"
    );
    assert!(
        content.contains("Review carefully."),
        "inline rules must appear; got:\n{content}"
    );
}

#[test]
fn claude_team_agents_passed_through_to_agent_files() {
    let r = ResolvedConfig {
        claude_team_agents: vec![(
            "lead.md".to_string(),
            "# Team Lead\nYou lead the team.".to_string(),
        )],
        ..resolved(vec![])
    };
    let out = compile(&r, "claude").unwrap();
    assert!(out.agent_files.contains_key(".claude/agents/lead.md"));
    assert_eq!(
        out.agent_files[".claude/agents/lead.md"],
        "# Team Lead\nYou lead the team."
    );
}

#[test]
fn team_agents_only_for_claude() {
    let r = ResolvedConfig {
        claude_team_agents: vec![("lead.md".to_string(), "content".to_string())],
        ..resolved(vec![])
    };
    for provider in &["gemini", "codex", "cursor"] {
        let out = compile(&r, provider).unwrap();
        assert!(
            !out.agent_files.contains_key(".claude/agents/lead.md"),
            "{provider} must not get claude team agents"
        );
    }
}

// ── Phase 4: Claude provider settings ────────────────────────────────────────

#[test]
fn claude_theme_emitted_in_settings_patch() {
    let r = ResolvedConfig {
        claude_theme: Some("dark".to_string()),
        ..resolved(vec![])
    };
    let out = compile(&r, "claude").unwrap();
    let patch = out.claude_settings_patch.expect("theme must emit a patch");
    assert_eq!(patch["theme"], "dark");
}

#[test]
fn claude_auto_updates_emitted_in_settings_patch() {
    let r = ResolvedConfig {
        claude_auto_updates: Some(false),
        ..resolved(vec![])
    };
    let out = compile(&r, "claude").unwrap();
    let patch = out
        .claude_settings_patch
        .expect("autoUpdates must emit a patch");
    assert_eq!(patch["autoUpdates"], false);
}

#[test]
fn claude_include_co_authored_by_emitted() {
    let r = ResolvedConfig {
        claude_include_co_authored_by: Some(true),
        ..resolved(vec![])
    };
    let out = compile(&r, "claude").unwrap();
    let patch = out
        .claude_settings_patch
        .expect("includeCoAuthoredBy must emit a patch");
    assert_eq!(patch["includeCoAuthoredBy"], true);
}

/// claude_settings_extra must remain last — it can override typed fields.
#[test]
fn claude_settings_extra_is_last_and_can_override() {
    let r = ResolvedConfig {
        claude_theme: Some("light".to_string()),
        claude_settings_extra: Some(serde_json::json!({ "theme": "dark" })),
        ..resolved(vec![])
    };
    let out = compile(&r, "claude").unwrap();
    let patch = out.claude_settings_patch.expect("must emit a patch");
    // extra overrides the typed field
    assert_eq!(patch["theme"], "dark");
}

/// Verifies the correct Claude Code hook format: wrapped in { hooks: [...] }.
#[test]
fn claude_hooks_wrapped_in_hooks_array() {
    use crate::types::HookTrigger;
    let r = ResolvedConfig {
        hooks: vec![make_hook(HookTrigger::Stop, "ship notify", None)],
        ..resolved(vec![])
    };
    let out = compile(&r, "claude").unwrap();
    let patch = out.claude_settings_patch.unwrap();
    // Structure: { "Stop": [{ "hooks": [{ "type": "command", "command": "..." }] }] }
    let stop_arr = patch["hooks"]["Stop"].as_array().unwrap();
    assert_eq!(stop_arr.len(), 1);
    let entry = &stop_arr[0];
    assert!(
        entry.get("hooks").is_some(),
        "hook entry must have 'hooks' array"
    );
    let inner = entry["hooks"].as_array().unwrap();
    assert_eq!(inner[0]["command"], "ship notify");
    assert_eq!(inner[0]["type"], "command");
    // Must NOT have "command" at the top level
    assert!(
        entry.get("command").is_none(),
        "command must be nested inside hooks array"
    );
}
