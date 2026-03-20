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
            codex_enabled_tools: vec![],
            codex_disabled_tools: vec![],
            gemini_trust: None,
            gemini_include_tools: vec![],
            gemini_exclude_tools: vec![],
            gemini_timeout_ms: None,
            cursor_env_file: None,
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
        Some(HookConfig { id: e.id, trigger, command: e.command, matcher: e.matcher, cursor_event: None, gemini_event: None })
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

/// Parse a SKILL.md file per the agentskills.io spec.
///
/// Frontmatter keys parsed: `name`, `description`, `license`, `compatibility`,
/// `allowed-tools` (space-delimited), `metadata` (key-value block).
/// Legacy top-level `version` and `author` keys are folded into `metadata`.
/// Emits a warning if the frontmatter `name` does not match the directory `id`.
fn parse_skill(id: &str, raw: &str) -> Skill {
    use std::collections::HashMap;

    let mut name = id.to_string();
    let mut description = None;
    let mut license = None;
    let mut compatibility = None;
    let mut allowed_tools = vec![];
    let mut metadata: HashMap<String, String> = HashMap::new();
    let mut content_start = 0usize;

    if let Some(rest) = raw.strip_prefix("---\n")
        && let Some(end) = rest.find("\n---\n")
    {
        let fm = &rest[..end];
        let mut in_metadata = false;
        for line in fm.lines() {
            if in_metadata {
                // Indented key: value pairs under `metadata:`
                if line.starts_with("  ") || line.starts_with('\t') {
                    let trimmed = line.trim();
                    if let Some((k, v)) = trimmed.split_once(':') {
                        metadata.insert(k.trim().to_string(), v.trim().to_string());
                    }
                    continue;
                } else {
                    in_metadata = false;
                }
            }
            if let Some(v) = line.strip_prefix("name:") {
                name = v.trim().to_string();
            } else if let Some(v) = line.strip_prefix("description:") {
                description = Some(v.trim().to_string());
            } else if let Some(v) = line.strip_prefix("license:") {
                license = Some(v.trim().to_string());
            } else if let Some(v) = line.strip_prefix("compatibility:") {
                compatibility = Some(v.trim().to_string());
            } else if let Some(v) = line.strip_prefix("allowed-tools:") {
                allowed_tools = v.trim().split_whitespace().map(str::to_string).collect();
            } else if line.trim_end() == "metadata:" {
                in_metadata = true;
            } else if let Some(v) = line.strip_prefix("version:") {
                // Legacy: fold into metadata
                metadata.insert("version".to_string(), v.trim().to_string());
            } else if let Some(v) = line.strip_prefix("author:") {
                // Legacy: fold into metadata
                metadata.insert("author".to_string(), v.trim().to_string());
            }
        }
        content_start = 4 + end + 5; // "---\n" + fm + "\n---\n"
    }

    // Warn if frontmatter name does not match directory id (spec requirement).
    // Allow human-readable display names like "Write ADR" for id "write-adr".
    let normalized_name = name.to_lowercase().replace(' ', "-");
    if normalized_name != id && name != id {
        eprintln!(
            "warning: skill '{}': frontmatter name '{}' does not match directory name",
            id, name
        );
    }

    let content = raw[content_start..].trim().to_string();
    Skill {
        id: id.to_string(),
        name,
        description,
        license,
        compatibility,
        allowed_tools,
        metadata,
        content,
        source: SkillSource::default(),
    }
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
#[path = "loader_tests.rs"]
mod tests;

