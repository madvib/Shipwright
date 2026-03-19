//! Load a [`ProjectLibrary`] from the `.ship/agents/` directory tree.
//! No compilation or resolution occurs here — pure filesystem loading.

use anyhow::Result;
use compiler::{AgentProfile, HookConfig, HookTrigger, McpServerConfig, McpServerType, Permissions, ProjectLibrary, Rule, Skill, SkillSource};
use serde::Deserialize;
use std::path::Path;

// ── Top-level entry point ─────────────────────────────────────────────────────

/// Load a [`ProjectLibrary`] from an `agents/` directory.
/// Missing files and dirs are silently skipped — an empty library is valid.
pub fn load_library(agents_dir: &Path) -> Result<ProjectLibrary> {
    Ok(ProjectLibrary {
        mcp_servers: load_mcp_servers(agents_dir)?,
        permissions: load_permissions(agents_dir)?,
        hooks: load_hooks(agents_dir)?,
        rules: load_rules(agents_dir)?,
        skills: load_skills(agents_dir)?,
        agent_profiles: load_agent_profiles(agents_dir)?,
        ..Default::default()
    })
}

// ── MCP servers ───────────────────────────────────────────────────────────────

fn load_mcp_servers(agents_dir: &Path) -> Result<Vec<McpServerConfig>> {
    let path = agents_dir.join("mcp.toml");
    let file = crate::mcp::McpFile::load(&path)?;
    Ok(file.servers.into_iter().map(|e| {
        let server_type = match e.server_type.as_deref() {
            Some("http") => McpServerType::Http,
            Some("sse")  => McpServerType::Sse,
            _ => if e.url.is_some() && e.command.is_none() { McpServerType::Http }
                 else { McpServerType::Stdio },
        };
        McpServerConfig {
            id: e.id.clone(),
            name: e.name.unwrap_or_else(|| e.id.clone()),
            command: e.command.unwrap_or_default(),
            args: e.args,
            env: e.env,
            scope: e.scope,
            server_type,
            url: e.url,
            disabled: e.disabled,
            timeout_secs: None,
        }
    }).collect())
}

// ── Permissions ───────────────────────────────────────────────────────────────

/// A named permission preset section from `agents/permissions.toml`.
/// Matches the `[ship-standard]`, `[ship-guarded]`, etc. blocks.
#[derive(Deserialize, Default, Clone)]
pub struct PermissionPreset {
    #[serde(default)]
    pub default_mode: Option<String>,
    #[serde(default)]
    pub tools_allow: Vec<String>,
    #[serde(default)]
    pub tools_deny: Vec<String>,
    #[serde(default)]
    pub tools_ask: Vec<String>,
}


fn load_permissions(agents_dir: &Path) -> Result<Permissions> {
    let path = agents_dir.join("permissions.toml");
    if !path.exists() { return Ok(Permissions::default()); }
    let s = std::fs::read_to_string(&path)?;
    // Try flat Permissions first, fall back to default on parse error.
    // The named-preset sections ([ship-standard] etc.) are ignored here —
    // they are resolved via load_permission_preset() when a profile activates.
    match toml::from_str::<Permissions>(&s) {
        Ok(p) => Ok(p),
        Err(_) => Ok(Permissions::default()),
    }
}

/// Load a named permission preset section (e.g. `[ship-standard]`) from
/// `agents/permissions.toml`. Returns `None` if the file or section is absent.
pub fn load_permission_preset(agents_dir: &Path, preset_name: &str) -> Option<PermissionPreset> {
    let path = agents_dir.join("permissions.toml");
    if !path.exists() { return None; }
    let s = std::fs::read_to_string(&path).ok()?;
    let val: toml::Value = toml::from_str(&s).ok()?;
    let section = val.get(preset_name)?.as_table()?;

    let get_str_list = |key: &str| -> Vec<String> {
        section.get(key)
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
            .unwrap_or_default()
    };
    let default_mode = section.get("default_mode")
        .and_then(|v| v.as_str())
        .map(str::to_string);

    Some(PermissionPreset {
        default_mode,
        tools_allow: get_str_list("tools_allow"),
        tools_deny: get_str_list("tools_deny"),
        tools_ask: get_str_list("tools_ask"),
    })
}

// ── Hooks ─────────────────────────────────────────────────────────────────────

#[derive(Deserialize, Default)]
struct HooksFile {
    #[serde(default)]
    hooks: Vec<HookEntry>,
}

#[derive(Deserialize)]
struct HookEntry {
    id: String,
    trigger: String,
    command: String,
    #[serde(default)]
    matcher: Option<String>,
}

fn load_hooks(agents_dir: &Path) -> Result<Vec<HookConfig>> {
    let path = agents_dir.join("hooks.toml");
    if !path.exists() { return Ok(vec![]); }
    let file: HooksFile = toml::from_str(&std::fs::read_to_string(&path)?)?;
    Ok(file.hooks.into_iter().filter_map(|e| {
        let trigger = match e.trigger.as_str() {
            "pre_tool_use"  | "PreToolUse"  => HookTrigger::PreToolUse,
            "post_tool_use" | "PostToolUse" => HookTrigger::PostToolUse,
            "notification"  | "Notification"=> HookTrigger::Notification,
            "stop"          | "Stop"        => HookTrigger::Stop,
            "subagent_stop" | "SubagentStop"=> HookTrigger::SubagentStop,
            "pre_compact"   | "PreCompact"  => HookTrigger::PreCompact,
            _ => return None,
        };
        Some(HookConfig { id: e.id, trigger, command: e.command, matcher: e.matcher })
    }).collect())
}

// ── Rules ─────────────────────────────────────────────────────────────────────

fn load_rules(agents_dir: &Path) -> Result<Vec<Rule>> {
    let rules_dir = agents_dir.join("rules");
    if !rules_dir.exists() { return Ok(vec![]); }
    let mut rules = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(&rules_dir)?
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "md"))
        .collect();
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let file_name = e.file_name().to_string_lossy().to_string();
        let raw = std::fs::read_to_string(e.path())?;
        rules.push(parse_rule(&file_name, &raw));
    }
    Ok(rules)
}

/// Parse a rule `.md` file, stripping YAML frontmatter if present.
/// Frontmatter fields: `description`, `globs` (list), `alwaysApply` (bool).
fn parse_rule(file_name: &str, raw: &str) -> Rule {
    if let Some(rest) = raw.strip_prefix("---\n")
        && let Some(end) = rest.find("\n---\n")
    {
        let fm = &rest[..end];
        let body = &rest[end + 5..];
        let mut always_apply = true;
        let mut globs = vec![];
        let mut description = None;
        for line in fm.lines() {
            if let Some(v) = line.strip_prefix("alwaysApply:") {
                always_apply = v.trim() != "false";
            } else if let Some(v) = line.strip_prefix("description:") {
                description = Some(v.trim().trim_matches('"').to_string());
            } else if line.trim_start().starts_with("- ") {
                globs.push(line.trim().trim_start_matches("- ").to_string());
            }
        }
        return Rule { file_name: file_name.to_string(), content: body.trim().to_string(),
                      always_apply, globs, description };
    }
    Rule { file_name: file_name.to_string(), content: raw.trim().to_string(),
           always_apply: true, globs: vec![], description: None }
}

// ── Skills ────────────────────────────────────────────────────────────────────

fn load_skills(agents_dir: &Path) -> Result<Vec<Skill>> {
    let skills_dir = agents_dir.join("skills");
    if !skills_dir.exists() { return Ok(vec![]); }
    let mut skills = Vec::new();
    for entry in std::fs::read_dir(&skills_dir)?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Subdirectory format: <skill-id>/SKILL.md
            let skill_md = path.join("SKILL.md");
            if skill_md.exists() {
                let id = entry.file_name().to_string_lossy().to_string();
                let raw = std::fs::read_to_string(&skill_md)?;
                skills.push(parse_skill(&id, &raw));
            }
        } else if path.extension().is_some_and(|x| x == "md") {
            // Flat format: <skill-id>.md
            let id = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
            let raw = std::fs::read_to_string(&path)?;
            skills.push(parse_skill(&id, &raw));
        }
    }
    Ok(skills)
}

/// Parse a SKILL.md file, extracting frontmatter name/description fields.
fn parse_skill(id: &str, raw: &str) -> Skill {
    let mut name = id.to_string();
    let mut description = None;
    let mut content_start = 0;

    if let Some(rest) = raw.strip_prefix("---\n")
        && let Some(end) = rest.find("\n---\n")
    {
        let fm = &rest[..end];
        for line in fm.lines() {
            if let Some(v) = line.strip_prefix("name:") {
                name = v.trim().to_string();
            } else if let Some(v) = line.strip_prefix("description:") {
                description = Some(v.trim().to_string());
            }
        }
        content_start = 4 + end + 5; // "---\n" + fm + "\n---\n"
    }
    let content = raw[content_start..].trim().to_string();
    Skill { id: id.to_string(), name, description, version: None, author: None,
            content, source: SkillSource::default() }
}

// ── Agent profiles ────────────────────────────────────────────────────────────

fn load_agent_profiles(agents_dir: &Path) -> Result<Vec<AgentProfile>> {
    let profiles_dir = agents_dir.join("profiles");
    if !profiles_dir.exists() { return Ok(vec![]); }
    let mut profiles = Vec::new();
    for entry in std::fs::read_dir(&profiles_dir)?.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|x| x == "toml") {
            let content = std::fs::read_to_string(&path)?;
            match toml::from_str::<AgentProfile>(&content) {
                Ok(profile) => profiles.push(profile),
                Err(e) => {
                    eprintln!(
                        "warning: skipping {}: {}",
                        path.file_name().unwrap_or_default().to_string_lossy(),
                        e,
                    );
                }
            }
        }
    }
    profiles.sort_by(|a, b| a.profile.id.cmp(&b.profile.id));
    Ok(profiles)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
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

[ship-open]
default_mode = "bypassPermissions"
"#);
        let preset = load_permission_preset(tmp.path(), "ship-standard").unwrap();
        assert_eq!(preset.default_mode.as_deref(), Some("acceptEdits"));
        assert!(preset.tools_ask.contains(&"Bash(rm -rf*)".to_string()));
        assert!(preset.tools_deny.contains(&"Bash(git push --force*)".to_string()));
    }

    #[test]
    fn load_permission_preset_missing_section_returns_none() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), "permissions.toml", "[ship-open]\ndefault_mode = \"bypassPermissions\"\n");
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
        // Sorted by id
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
        // Named-section-only permissions.toml (not a flat Permissions struct)
        write(tmp.path(), "permissions.toml", r#"
[ship-standard]
default_mode = "acceptEdits"
"#);
        // load_library uses flat-parse path which falls back to Permissions::default().
        // Permissions::default() (Rust derive) has tools.allow = [] (not ["*"]).
        // Named preset sections must be accessed via load_permission_preset(), not load_library().
        let lib = load_library(tmp.path()).unwrap();
        // The library loads without error — permissions fall back to default.
        // Named sections are resolved separately via load_permission_preset.
        assert!(lib.permissions.tools.allow.is_empty() || !lib.permissions.tools.allow.contains(&"[ship-standard]".to_string()),
            "named preset sections must not appear as allow patterns");
        // load_permission_preset can still read the named section
        let preset = load_permission_preset(tmp.path(), "ship-standard").unwrap();
        assert_eq!(preset.default_mode.as_deref(), Some("acceptEdits"));
    }
}
