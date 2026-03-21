use super::*;
use tempfile::TempDir;

fn write(dir: &Path, rel: &str, content: &str) {
    let p = dir.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

#[test]
fn empty_agents_dir_gives_default_library() {
    let tmp = TempDir::new().unwrap();
    let lib = load_library(tmp.path()).unwrap();
    assert!(lib.mcp_servers.is_empty());
    assert!(lib.skills.is_empty());
    assert!(lib.rules.is_empty());
}

#[test]
fn loads_stdio_mcp_server() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), "mcp.toml", r#"
[[servers]]
id = "github"
name = "GitHub"
command = "npx"
args = ["-y", "@mcp/github"]
"#);
    let lib = load_library(tmp.path()).unwrap();
    assert_eq!(lib.mcp_servers.len(), 1);
    assert_eq!(lib.mcp_servers[0].id, "github");
    assert_eq!(lib.mcp_servers[0].command, "npx");
    assert!(matches!(lib.mcp_servers[0].server_type, McpServerType::Stdio));
}

#[test]
fn loads_http_mcp_server_by_url() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), "mcp.toml", r#"
[[servers]]
id = "remote"
url = "https://api.example.com/mcp"
"#);
    let lib = load_library(tmp.path()).unwrap();
    assert!(matches!(lib.mcp_servers[0].server_type, McpServerType::Http));
}

#[test]
fn loads_rules_sorted_alphabetically() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), "rules/b-workflow.md", "Run tests first.");
    write(tmp.path(), "rules/a-style.md", "Use explicit types.");
    let lib = load_library(tmp.path()).unwrap();
    assert_eq!(lib.rules.len(), 2);
    assert_eq!(lib.rules[0].file_name, "a-style.md");
    assert_eq!(lib.rules[1].file_name, "b-workflow.md");
}

#[test]
fn rule_frontmatter_parsed() {
    let raw = "---\ndescription: \"Style guide\"\nalwaysApply: false\n---\n\nKeep it clean.";
    let rule = parse_rule("style.md", raw);
    assert_eq!(rule.description.as_deref(), Some("Style guide"));
    assert!(!rule.always_apply);
    assert_eq!(rule.content, "Keep it clean.");
}

#[test]
fn rule_without_frontmatter_uses_full_content() {
    let rule = parse_rule("plain.md", "Just plain content.");
    assert_eq!(rule.content, "Just plain content.");
    assert!(rule.always_apply);
    assert!(rule.description.is_none());
}

#[test]
fn loads_skill_from_skill_md() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), "skills/my-skill/SKILL.md",
        "---\nname: My Skill\ndescription: Does stuff\n---\n\nInstructions here.");
    let lib = load_library(tmp.path()).unwrap();
    assert_eq!(lib.skills.len(), 1);
    assert_eq!(lib.skills[0].id, "my-skill");
    assert_eq!(lib.skills[0].name, "My Skill");
    assert_eq!(lib.skills[0].description.as_deref(), Some("Does stuff"));
    assert_eq!(lib.skills[0].content, "Instructions here.");
}

#[test]
fn loads_skill_from_flat_md() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), "skills/ship-coordination.md",
        "---\nname: Ship Coordination\ndescription: Coordination skill\n---\n\nContent here.");
    let lib = load_library(tmp.path()).unwrap();
    assert_eq!(lib.skills.len(), 1);
    assert_eq!(lib.skills[0].id, "ship-coordination");
    assert_eq!(lib.skills[0].name, "Ship Coordination");
}

#[test]
fn loads_skills_mixed_formats() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), "skills/flat-skill.md",
        "---\nname: Flat\n---\n\nFlat content.");
    write(tmp.path(), "skills/dir-skill/SKILL.md",
        "---\nname: Dir\n---\n\nDir content.");
    let lib = load_library(tmp.path()).unwrap();
    assert_eq!(lib.skills.len(), 2);
}

#[test]
fn loads_permissions() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), "permissions.toml", r#"
[tools]
deny = ["Bash(rm -rf *)"]
"#);
    let lib = load_library(tmp.path()).unwrap();
    assert!(lib.permissions.tools.deny.contains(&"Bash(rm -rf *)".to_string()));
}

#[test]
fn loads_hooks() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), "hooks.toml", r#"
[[hooks]]
id = "check"
trigger = "pre_tool_use"
command = "ship hooks run"
matcher = "Bash"
"#);
    let lib = load_library(tmp.path()).unwrap();
    assert_eq!(lib.hooks.len(), 1);
    assert_eq!(lib.hooks[0].command, "ship hooks run");
    assert!(matches!(lib.hooks[0].trigger, HookTrigger::PreToolUse));
}

#[test]
fn unknown_hook_trigger_skipped() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), "hooks.toml", r#"
[[hooks]]
id = "bad"
trigger = "unknown_event"
command = "echo hi"
"#);
    let lib = load_library(tmp.path()).unwrap();
    assert!(lib.hooks.is_empty(), "unknown trigger must be silently dropped");
}

#[test]
fn load_permission_preset_reads_named_section() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), "permissions.toml", r#"
[ship-standard]
default_mode = "acceptEdits"
tools_ask = ["Bash(rm -rf*)"]
tools_deny = ["Bash(git push --force*)"]

[ship-elevated]
default_mode = "dontAsk"
"#);
    let preset = load_permission_preset(tmp.path(), "ship-standard").unwrap();
    assert_eq!(preset.default_mode.as_deref(), Some("acceptEdits"));
    assert!(preset.tools_ask.contains(&"Bash(rm -rf*)".to_string()));
    assert!(preset.tools_deny.contains(&"Bash(git push --force*)".to_string()));
}

#[test]
fn load_permission_preset_missing_section_returns_none() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), "permissions.toml", "[ship-elevated]\ndefault_mode = \"dontAsk\"\n");
    let result = load_permission_preset(tmp.path(), "nonexistent");
    assert!(result.is_none());
}

#[test]
fn load_permission_preset_missing_file_returns_none() {
    let tmp = TempDir::new().unwrap();
    let result = load_permission_preset(tmp.path(), "ship-standard");
    assert!(result.is_none());
}

#[test]
fn loads_agent_profiles_from_profiles_dir() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), "profiles/web-lane.toml", r#"
[profile]
id = "web-lane"
name = "Web Lane"
providers = ["claude"]

[skills]
refs = ["tanstack-start"]

[permissions]
preset = "ship-standard"
"#);
    write(tmp.path(), "profiles/server-lane.toml", r#"
[profile]
id = "server-lane"
name = "Server Lane"
providers = ["claude"]
"#);
    let lib = load_library(tmp.path()).unwrap();
    assert_eq!(lib.agent_profiles.len(), 2);
    assert_eq!(lib.agent_profiles[0].profile.id, "server-lane");
    assert_eq!(lib.agent_profiles[1].profile.id, "web-lane");
    assert_eq!(lib.agent_profiles[1].skills.refs, vec!["tanstack-start"]);
}

#[test]
fn agent_profiles_empty_when_dir_missing() {
    let tmp = TempDir::new().unwrap();
    let lib = load_library(tmp.path()).unwrap();
    assert!(lib.agent_profiles.is_empty());
}

#[test]
fn agent_profiles_skips_invalid_toml() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), "profiles/good.toml", r#"
[profile]
id = "good"
name = "Good"
"#);
    write(tmp.path(), "profiles/bad.toml", "this is not valid toml { { {");
    let lib = load_library(tmp.path()).unwrap();
    assert_eq!(lib.agent_profiles.len(), 1);
    assert_eq!(lib.agent_profiles[0].profile.id, "good");
}

#[test]
fn permissions_file_with_named_sections_falls_back_to_default() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), "permissions.toml", r#"
[ship-standard]
default_mode = "acceptEdits"
"#);
    let lib = load_library(tmp.path()).unwrap();
    assert!(lib.permissions.tools.allow.is_empty()
        || !lib.permissions.tools.allow.contains(&"[ship-standard]".to_string()));
    let preset = load_permission_preset(tmp.path(), "ship-standard").unwrap();
    assert_eq!(preset.default_mode.as_deref(), Some("acceptEdits"));
}

// ── Skill spec fields ─────────────────────────────────────────────────────────

#[test]
fn skill_parses_license_and_compatibility() {
    let raw = "---\nname: my-skill\ndescription: Does stuff\nlicense: MIT\ncompatibility: claude >= 3\n---\n\nInstructions.";
    let skill = parse_skill("my-skill", raw);
    assert_eq!(skill.license.as_deref(), Some("MIT"));
    assert_eq!(skill.compatibility.as_deref(), Some("claude >= 3"));
}

#[test]
fn skill_parses_allowed_tools_hyphenated_key() {
    let raw = "---\nname: my-skill\ndescription: x\nallowed-tools: Read Edit Grep\n---\n\nContent.";
    let skill = parse_skill("my-skill", raw);
    assert_eq!(skill.allowed_tools, vec!["Read", "Edit", "Grep"]);
}

#[test]
fn skill_parses_metadata_block() {
    let raw = "---\nname: my-skill\ndescription: x\nmetadata:\n  team: infra\n  priority: high\n---\n\nContent.";
    let skill = parse_skill("my-skill", raw);
    assert_eq!(skill.metadata.get("team").map(String::as_str), Some("infra"));
    assert_eq!(skill.metadata.get("priority").map(String::as_str), Some("high"));
}

#[test]
fn skill_folds_legacy_version_author_into_metadata() {
    let raw = "---\nname: my-skill\ndescription: x\nversion: 1.2.3\nauthor: alice\n---\n\nContent.";
    let skill = parse_skill("my-skill", raw);
    assert_eq!(skill.metadata.get("version").map(String::as_str), Some("1.2.3"));
    assert_eq!(skill.metadata.get("author").map(String::as_str), Some("alice"));
}

#[test]
fn skill_allowed_tools_empty_when_absent() {
    let raw = "---\nname: my-skill\ndescription: x\n---\n\nContent.";
    let skill = parse_skill("my-skill", raw);
    assert!(skill.allowed_tools.is_empty());
}

#[test]
fn skill_metadata_empty_when_absent() {
    let raw = "---\nname: my-skill\ndescription: x\n---\n\nContent.";
    let skill = parse_skill("my-skill", raw);
    assert!(skill.metadata.is_empty());
}
