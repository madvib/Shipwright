use crate::compile::agents::{compile_agent_profiles, profile_targets_provider};
use crate::types::AgentProfile;
use crate::types::agent_profile::*;

fn make_profile(id: &str, name: &str, providers: &[&str]) -> AgentProfile {
    AgentProfile {
        profile: ProfileMeta {
            id: id.to_string(),
            name: name.to_string(),
            version: None,
            description: Some(format!("{name} agent")),
            providers: providers.iter().map(|s| s.to_string()).collect(),
            icon: None,
        },
        skills: SkillRefs {
            refs: vec!["skill-a".to_string()],
        },
        mcp: McpRefs {
            servers: vec!["ship".to_string()],
        },
        plugins: PluginRefs::default(),
        permissions: ProfilePermissions {
            preset: Some("ship-standard".to_string()),
            tools_deny: vec!["Bash(rm -rf *)".to_string()],
            default_mode: Some("acceptEdits".to_string()),
            ..Default::default()
        },
        rules: ProfileRules {
            inline: Some("You are a specialist.".to_string()),
        },
        apps: Default::default(),
        provider_settings: Default::default(),
    }
}

#[test]
fn provider_filter_targets_listed() {
    let p = make_profile("test", "Test", &["claude", "gemini"]);
    assert!(profile_targets_provider(&p, "claude"));
    assert!(profile_targets_provider(&p, "gemini"));
    assert!(!profile_targets_provider(&p, "cursor"));
}

#[test]
fn provider_filter_empty_targets_all() {
    let p = make_profile("test", "Test", &[]);
    assert!(profile_targets_provider(&p, "claude"));
    assert!(profile_targets_provider(&p, "codex"));
}

#[test]
fn claude_agent_output() {
    let p = make_profile("reviewer", "Code Reviewer", &["claude"]);
    let files = compile_agent_profiles(&[p], "claude");
    assert_eq!(files.len(), 1);
    let content = &files[".claude/agents/reviewer.md"];
    assert!(content.starts_with("---\n"));
    assert!(content.contains("name: reviewer"));
    assert!(content.contains("description: Code Reviewer agent"));
    assert!(content.contains("permissionMode: acceptEdits"));
    assert!(content.contains("mcpServers:"));
    assert!(content.contains("  - ship"));
    assert!(content.contains("skills:"));
    assert!(content.contains("  - skill-a"));
    assert!(content.contains("disallowedTools:"));
    assert!(content.contains("You are a specialist."));
}

#[test]
fn gemini_agent_output() {
    let p = make_profile("reviewer", "Code Reviewer", &["gemini"]);
    let files = compile_agent_profiles(&[p], "gemini");
    assert_eq!(files.len(), 1);
    let content = &files[".gemini/agents/reviewer.md"];
    assert!(content.starts_with("---\n"));
    assert!(content.contains("name: reviewer"));
    assert!(content.contains("kind: local"));
    assert!(content.contains("tools:"));
    assert!(content.contains("You are a specialist."));
}

#[test]
fn cursor_agent_output() {
    let p = make_profile("reviewer", "Code Reviewer", &["cursor"]);
    let files = compile_agent_profiles(&[p], "cursor");
    assert_eq!(files.len(), 1);
    let content = &files[".cursor/agents/reviewer.md"];
    assert!(content.starts_with("---\n"));
    assert!(content.contains("name: reviewer"));
    assert!(content.contains("description: Code Reviewer agent"));
    assert!(content.contains("You are a specialist."));
}

#[test]
fn codex_agent_output() {
    let p = make_profile("reviewer", "Code Reviewer", &["codex"]);
    let files = compile_agent_profiles(&[p], "codex");
    assert_eq!(files.len(), 1);
    let content = &files[".codex/agents/reviewer.toml"];
    assert!(content.contains("name = \"Code Reviewer\""));
    assert!(content.contains("description = \"Code Reviewer agent\""));
    assert!(content.contains("[mcp_servers.ship]"));
}

#[test]
fn skips_unmatched_provider() {
    let p = make_profile("reviewer", "Code Reviewer", &["claude"]);
    let files = compile_agent_profiles(&[p], "gemini");
    assert!(files.is_empty());
}

#[test]
fn multiple_profiles() {
    let profiles = vec![
        make_profile("alpha", "Alpha", &["claude"]),
        make_profile("beta", "Beta", &["claude"]),
    ];
    let files = compile_agent_profiles(&profiles, "claude");
    assert_eq!(files.len(), 2);
    assert!(files.contains_key(".claude/agents/alpha.md"));
    assert!(files.contains_key(".claude/agents/beta.md"));
}

#[test]
fn yaml_quoting() {
    use crate::compile::agents::yaml_quote;
    assert_eq!(yaml_quote("simple"), "simple");
    assert_eq!(yaml_quote("has: colon"), "\"has: colon\"");
    assert_eq!(yaml_quote("Bash(rm -rf *)"), "\"Bash(rm -rf *)\"");
}
