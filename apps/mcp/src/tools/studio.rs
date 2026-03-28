use compiler::{ListAgentsResponse, PullAgent, PullMcpServer, PullProfile, PullRule, PullSkill};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub use super::studio_push::push_bundle;

// ── Helpers ─────────────────────────────────────────────────────────────

fn library_dir() -> Option<PathBuf> {
    runtime::project::get_global_dir().ok()
}

fn collect_agent_ids(agents_dir: &Path) -> Vec<String> {
    let mut ids = Vec::new();
    if let Ok(entries) = std::fs::read_dir(agents_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if let Some(id) = name
                .strip_suffix(".jsonc")
                .or_else(|| name.strip_suffix(".toml"))
            {
                ids.push(id.to_string());
            }
        }
    }
    ids
}

// ── List ────────────────────────────────────────────────────────────────

pub fn list_local_agents(project_dir: &Path) -> String {
    let mut ids = collect_agent_ids(&project_dir.join(".ship").join("agents"));
    if let Some(lib) = library_dir() {
        for id in collect_agent_ids(&lib.join("agents")) {
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }
    ids.sort();
    serde_json::to_string(&ListAgentsResponse { agents: ids }).unwrap_or_default()
}

// ── Pull: CLI → Studio ──────────────────────────────────────────────────

/// Full JSONC agent file shape — every field the schema defines.
#[derive(Debug, serde::Deserialize)]
struct AgentJsonc {
    agent: AgentJsoncProfile,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    env: Option<HashMap<String, String>>,
    #[serde(default)]
    available_models: Option<Vec<String>>,
    #[serde(default)]
    agent_limits: Option<serde_json::Value>,
    #[serde(default)]
    skills: Option<AgentJsoncRefs>,
    #[serde(default)]
    mcp: Option<AgentJsoncMcp>,
    #[serde(default)]
    plugins: Option<serde_json::Value>,
    #[serde(default)]
    permissions: Option<serde_json::Value>,
    #[serde(default)]
    rules: Option<AgentJsoncRules>,
    #[serde(default)]
    provider_settings: Option<serde_json::Value>,
    #[serde(default)]
    hooks: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, serde::Deserialize)]
struct AgentJsoncProfile {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    providers: Option<Vec<String>>,
    #[serde(default)]
    icon: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct AgentJsoncRefs {
    #[serde(default)]
    refs: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct AgentJsoncMcp {
    #[serde(default)]
    servers: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct AgentJsoncRules {
    #[serde(default)]
    refs: Vec<String>,
    #[serde(default)]
    inline: Option<String>,
}

pub fn pull_agents(project_dir: &Path) -> String {
    let mut agents = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    // Project agents (.ship/agents/)
    let ship_dir = project_dir.join(".ship");
    pull_agents_from_dir(&ship_dir, "project", &mut agents, &mut seen_ids);

    // Library agents (~/.ship/agents/) — skip IDs already found in project
    if let Some(lib) = library_dir() {
        pull_agents_from_dir(&lib, "library", &mut agents, &mut seen_ids);
    }

    agents.sort_by(|a, b| a.profile.id.cmp(&b.profile.id));
    serde_json::to_string(&compiler::PullResponse { agents }).unwrap_or_default()
}

/// Derive a default icon from agent tags when no explicit icon is set.
fn icon_from_tags(tags: &[String]) -> Option<String> {
    for tag in tags {
        let t = tag.to_lowercase();
        match t.as_str() {
            "testing" | "tdd" => return Some("\u{1f9ea}".into()),
            "review" | "security" => return Some("\u{1f50d}".into()),
            "design" | "ui" => return Some("\u{1f3a8}".into()),
            "deploy" | "release" => return Some("\u{1f680}".into()),
            "coordination" | "commander" => return Some("\u{1f3af}".into()),
            _ => {}
        }
    }
    None
}

fn pull_agents_from_dir(
    ship_dir: &Path,
    source: &str,
    agents: &mut Vec<PullAgent>,
    seen_ids: &mut std::collections::HashSet<String>,
) {
    let agents_dir = ship_dir.join("agents");
    let entries = match std::fs::read_dir(&agents_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if !name_str.ends_with(".jsonc") {
            continue;
        }
        let raw = match std::fs::read_to_string(entry.path()) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let parsed: AgentJsonc = match compiler::jsonc::from_jsonc_str(&raw) {
            Ok(p) => p,
            Err(_) => continue,
        };

        if !seen_ids.insert(parsed.agent.id.clone()) {
            continue; // project shadows library
        }

        let skill_refs = parsed.skills.map(|s| s.refs).unwrap_or_default();
        let rules_section = parsed.rules;
        let rule_refs = rules_section
            .as_ref()
            .map(|r| r.refs.clone())
            .unwrap_or_default();
        let rules_inline = rules_section.and_then(|r| r.inline);
        let mcp_names = parsed.mcp.map(|m| m.servers).unwrap_or_default();

        let skills = resolve_skills(ship_dir, &skill_refs);
        let rules = resolve_rules(ship_dir, &rule_refs);
        let mcp_servers = mcp_names
            .into_iter()
            .map(|n| PullMcpServer {
                name: n,
                command: String::new(),
                url: None,
            })
            .collect();

        let icon = parsed
            .agent
            .icon
            .or_else(|| icon_from_tags(&parsed.agent.tags));

        agents.push(PullAgent {
            profile: PullProfile {
                id: parsed.agent.id.clone(),
                name: parsed.agent.name.unwrap_or_else(|| parsed.agent.id.clone()),
                description: parsed.agent.description.unwrap_or_default(),
                providers: parsed
                    .agent
                    .providers
                    .unwrap_or_else(|| vec!["claude".into()]),
                version: parsed.agent.version.unwrap_or_else(|| "0.1.0".into()),
                icon,
            },
            skills,
            mcp_servers,
            rules,
            rules_inline,
            hooks: parsed.hooks.unwrap_or_default(),
            permissions: parsed.permissions,
            model: parsed.model,
            env: parsed.env,
            available_models: parsed.available_models,
            agent_limits: parsed.agent_limits,
            plugins: parsed.plugins,
            provider_settings: parsed.provider_settings,
            source: source.into(),
        });
    }
}

fn resolve_skills(ship_dir: &Path, refs: &[String]) -> Vec<PullSkill> {
    let skill_dirs = runtime::read_skill_paths(ship_dir);
    refs.iter()
        .filter_map(|r| {
            let id = r.rsplit('/').next().unwrap_or(r);
            // Search all configured skill paths; first match wins.
            let skill_dir = skill_dirs
                .iter()
                .map(|d| d.join(id))
                .find(|d| d.join("SKILL.md").exists())?;
            let skill_md = skill_dir.join("SKILL.md");
            let content = std::fs::read_to_string(&skill_md).ok()?;
            let fm = parse_skill_frontmatter(&content);

            let files = collect_skill_files(&skill_dir);

            let vars_schema = std::fs::read_to_string(skill_dir.join("assets/vars.json"))
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok());

            let evals = std::fs::read_to_string(skill_dir.join("evals/evals.json"))
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok());

            let reference_docs = collect_reference_docs(&skill_dir);

            Some(PullSkill {
                id: id.to_string(),
                name: fm.name.unwrap_or_else(|| id.to_string()),
                description: fm.description,
                content,
                source: "imported".into(),
                stable_id: fm.stable_id,
                tags: fm.tags,
                authors: fm.authors,
                vars_schema,
                files,
                reference_docs,
                evals,
            })
        })
        .collect()
}

/// Collect all file paths in a skill directory, relative to the skill root.
pub(crate) fn collect_skill_files(skill_dir: &Path) -> Vec<String> {
    let mut files = Vec::new();
    if !skill_dir.is_dir() {
        return files;
    }
    collect_files_recursive(skill_dir, skill_dir, &mut files);
    files.sort();
    files
}

pub(crate) fn collect_files_recursive(root: &Path, dir: &Path, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursive(root, &path, out);
        } else if path.is_file()
            && let Ok(rel) = path.strip_prefix(root)
        {
            let rel_str = rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/");
            out.push(rel_str);
        }
    }
}

/// Read markdown files from references/docs/ into a map.
pub(crate) fn collect_reference_docs(skill_dir: &Path) -> HashMap<String, String> {
    let docs_dir = skill_dir.join("references").join("docs");
    let mut docs = HashMap::new();
    if !docs_dir.is_dir() {
        return docs;
    }
    let mut doc_files = Vec::new();
    collect_files_recursive(skill_dir, &docs_dir, &mut doc_files);
    for rel_path in doc_files {
        let full = skill_dir.join(&rel_path);
        if let Ok(content) = std::fs::read_to_string(&full) {
            docs.insert(rel_path, content);
        }
    }
    docs
}

pub(crate) struct SkillFrontmatter {
    pub(crate) name: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) stable_id: Option<String>,
    pub(crate) tags: Vec<String>,
    pub(crate) authors: Vec<String>,
}

pub(crate) fn parse_skill_frontmatter(content: &str) -> SkillFrontmatter {
    let mut fm = SkillFrontmatter {
        name: None,
        description: None,
        stable_id: None,
        tags: Vec::new(),
        authors: Vec::new(),
    };

    if !content.starts_with("---") {
        return fm;
    }
    let rest = &content[3..];
    let end = match rest.find("\n---") {
        Some(i) => i,
        None => return fm,
    };
    let block = &rest[..end];
    for line in block.lines() {
        if let Some(v) = line.strip_prefix("name:") {
            fm.name = Some(v.trim().to_string());
        } else if let Some(v) = line.strip_prefix("description:") {
            fm.description = Some(v.trim().to_string());
        } else if let Some(v) = line.strip_prefix("stable-id:") {
            fm.stable_id = Some(v.trim().to_string());
        } else if let Some(v) = line.strip_prefix("tags:") {
            fm.tags = parse_inline_array(v.trim());
        } else if let Some(v) = line.strip_prefix("authors:") {
            fm.authors = parse_inline_array(v.trim());
        }
    }
    fm
}

/// Parse `[a, b, c]` into `vec!["a", "b", "c"]`.
fn parse_inline_array(s: &str) -> Vec<String> {
    let trimmed = s.trim();
    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return Vec::new();
    }
    trimmed[1..trimmed.len() - 1]
        .split(',')
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect()
}

fn resolve_rules(ship_dir: &Path, refs: &[String]) -> Vec<PullRule> {
    let rules_dir = ship_dir.join("rules");
    refs.iter()
        .filter_map(|r| {
            let file_name = if r.ends_with(".md") {
                r.clone()
            } else {
                format!("{r}.md")
            };
            let path = rules_dir.join(&file_name);
            let content = std::fs::read_to_string(&path).ok()?;
            Some(PullRule { file_name, content })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_agent_file(dir: &Path, id: &str, extra: &str) {
        let agents = dir.join("agents");
        std::fs::create_dir_all(&agents).unwrap();
        let content = format!(
            r#"{{ "agent": {{ "id": "{id}", "name": "{id}", "providers": ["claude"] }}{extra} }}"#,
        );
        std::fs::write(agents.join(format!("{id}.jsonc")), content).unwrap();
    }

    #[test]
    fn pull_agents_tags_source() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path().join("project");
        std::fs::create_dir_all(&project).unwrap();
        write_agent_file(&project.join(".ship"), "agent-a", "");

        let raw = pull_agents(&project);
        let resp: compiler::PullResponse = serde_json::from_str(&raw).unwrap();
        assert_eq!(resp.agents.len(), 1);
        assert_eq!(resp.agents[0].source, "project");
    }

    #[test]
    fn pull_agents_project_shadows_library() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path().join("project");
        let library = tmp.path().join("library");
        std::fs::create_dir_all(&project).unwrap();

        write_agent_file(&project.join(".ship"), "shared-agent", "");
        write_agent_file(&library, "shared-agent", "");
        write_agent_file(&library, "lib-only", "");

        let mut agents = Vec::new();
        let mut seen = std::collections::HashSet::new();
        pull_agents_from_dir(&project.join(".ship"), "project", &mut agents, &mut seen);
        pull_agents_from_dir(&library, "library", &mut agents, &mut seen);

        agents.sort_by(|a, b| a.profile.id.cmp(&b.profile.id));
        assert_eq!(agents.len(), 2);
        let shared = agents
            .iter()
            .find(|a| a.profile.id == "shared-agent")
            .unwrap();
        assert_eq!(shared.source, "project", "project should shadow library");
        let lib = agents.iter().find(|a| a.profile.id == "lib-only").unwrap();
        assert_eq!(lib.source, "library");
    }

    #[test]
    fn pull_preserves_all_fields() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path().join("project");
        std::fs::create_dir_all(&project).unwrap();
        write_agent_file(
            &project.join(".ship"),
            "full-agent",
            r#",
            "model": "claude-sonnet-4-20250514",
            "env": { "API_KEY": "test" },
            "available_models": ["claude-sonnet-4-20250514"],
            "agent_limits": { "max_turns": 50 },
            "permissions": { "preset": "ship-standard", "default_mode": "plan" },
            "provider_settings": { "claude": { "contextWindowTokens": 100000 } },
            "rules": { "inline": "Be concise." }
            "#,
        );

        let raw = pull_agents(&project);
        let resp: compiler::PullResponse = serde_json::from_str(&raw).unwrap();
        let a = &resp.agents[0];
        assert_eq!(a.model.as_deref(), Some("claude-sonnet-4-20250514"));
        assert_eq!(a.env.as_ref().unwrap().get("API_KEY").unwrap(), "test");
        assert_eq!(a.available_models.as_ref().unwrap().len(), 1);
        assert!(a.agent_limits.is_some());
        assert!(a.permissions.is_some());
        assert!(a.provider_settings.is_some());
        assert_eq!(a.rules_inline.as_deref(), Some("Be concise."));
    }

    #[test]
    fn collect_agent_ids_handles_missing_dir() {
        let ids = collect_agent_ids(Path::new("/nonexistent/path"));
        assert!(ids.is_empty());
    }

    fn write_skill_at(ship_dir: &Path, rel_dir: &str, id: &str, body: &str) {
        let skill_dir = ship_dir.join(rel_dir).join(id);
        std::fs::create_dir_all(&skill_dir).unwrap();
        let content = format!("---\nname: {id}\n---\n{body}");
        std::fs::write(skill_dir.join("SKILL.md"), content).unwrap();
    }

    fn write_manifest_skill_paths(ship_dir: &Path, paths: &[&str]) {
        let arr: Vec<String> = paths.iter().map(|p| format!("\"{}\"", p)).collect();
        let content = format!(
            r#"{{"id": "test", "project": {{"skill_paths": [{}]}}}}"#,
            arr.join(", ")
        );
        std::fs::write(ship_dir.join(runtime::config::PRIMARY_CONFIG_FILE), content).unwrap();
    }

    #[test]
    fn resolve_skills_finds_across_paths() {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        write_manifest_skill_paths(&ship_dir, &["skills/", "docs/"]);

        write_skill_at(&ship_dir, "skills", "tdd", "Write tests first.");
        write_skill_at(&ship_dir, "docs", "tutorial", "A tutorial skill.");

        let skills = resolve_skills(&ship_dir, &["tdd".to_string(), "tutorial".to_string()]);
        assert_eq!(skills.len(), 2);
        let ids: Vec<&str> = skills.iter().map(|s| s.id.as_str()).collect();
        assert!(ids.contains(&"tdd"));
        assert!(ids.contains(&"tutorial"));
    }

    #[test]
    fn resolve_skills_first_path_wins() {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        write_manifest_skill_paths(&ship_dir, &["first/", "second/"]);

        write_skill_at(&ship_dir, "first", "dup", "First version.");
        write_skill_at(&ship_dir, "second", "dup", "Second version.");

        let skills = resolve_skills(&ship_dir, &["dup".to_string()]);
        assert_eq!(skills.len(), 1);
        assert!(skills[0].content.contains("First version"));
    }

    #[test]
    fn list_local_agents_project_only() {
        let tmp = tempfile::tempdir().unwrap();
        write_agent_file(&tmp.path().join(".ship"), "my-agent", "");
        let raw = list_local_agents(tmp.path());
        let resp: ListAgentsResponse = serde_json::from_str(&raw).unwrap();
        assert!(resp.agents.contains(&"my-agent".to_string()));
    }

    #[test]
    fn push_then_pull_round_trips_all_fields() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();
        std::fs::create_dir_all(project.join(".ship")).unwrap();

        // Push a fully-configured agent
        let bundle = serde_json::json!({
            "agent": {
                "id": "round-trip",
                "name": "Round Trip Agent",
                "description": "Tests lossless transfer",
                "version": "1.2.3",
                "providers": ["claude", "cursor"],
                "model": "claude-sonnet-4-20250514",
                "env": { "SECRET": "abc" },
                "available_models": ["claude-sonnet-4-20250514", "claude-haiku-4-5-20251001"],
                "agent_limits": { "max_turns": 100, "max_cost_per_session": 5.0 },
                "skill_refs": ["tdd", "code-review"],
                "rule_refs": ["style-guide"],
                "rules_inline": "Always write tests first.",
                "mcp_servers": ["ship", "github"],
                "plugins": { "install": ["superpowers"], "scope": "project" },
                "permissions": {
                    "preset": "ship-standard",
                    "default_mode": "plan",
                    "tools_allow": ["Read", "Glob"],
                    "tools_deny": ["Bash(rm -rf *)"]
                },
                "provider_settings": {
                    "claude": { "contextWindowTokens": 100000 },
                    "cursor": { "tabAutocomplete": true }
                },
                "hooks": [{ "event": "onSave", "command": "cargo fmt" }]
            },
            "skills": {
                "tdd": { "files": { "SKILL.md": "---\nname: TDD\n---\nWrite tests first." } },
                "code-review": { "files": { "SKILL.md": "---\nname: Code Review\n---\nReview code." } }
            },
            "rules": {
                "style-guide": "Use snake_case for all functions."
            },
            "dependencies": {}
        });

        let result = push_bundle(project, &serde_json::to_string(&bundle).unwrap());
        assert!(!result.starts_with("Error"), "push failed: {result}");

        // Now pull it back
        let raw = pull_agents(project);
        let resp: compiler::PullResponse = serde_json::from_str(&raw).unwrap();
        assert_eq!(resp.agents.len(), 1);
        let a = &resp.agents[0];

        // Profile fields
        assert_eq!(a.profile.id, "round-trip");
        assert_eq!(a.profile.name, "Round Trip Agent");
        assert_eq!(a.profile.description, "Tests lossless transfer");
        assert_eq!(a.profile.version, "1.2.3");
        assert_eq!(a.profile.providers, vec!["claude", "cursor"]);

        // Top-level fields
        assert_eq!(a.model.as_deref(), Some("claude-sonnet-4-20250514"));
        assert_eq!(a.env.as_ref().unwrap().get("SECRET").unwrap(), "abc");
        assert_eq!(a.available_models.as_ref().unwrap().len(), 2);
        let limits = a.agent_limits.as_ref().unwrap();
        assert_eq!(limits["max_turns"], 100);

        // Permissions
        let perms = a.permissions.as_ref().unwrap();
        assert_eq!(perms["preset"], "ship-standard");
        assert_eq!(perms["default_mode"], "plan");

        // Skills resolved with content
        assert_eq!(a.skills.len(), 2);
        let tdd = a.skills.iter().find(|s| s.id == "tdd").unwrap();
        assert!(tdd.content.contains("Write tests first"));

        // Rules resolved with content
        assert_eq!(a.rules.len(), 1);
        assert!(a.rules[0].content.contains("snake_case"));
        assert_eq!(a.rules_inline.as_deref(), Some("Always write tests first."));

        // MCP servers
        assert_eq!(a.mcp_servers.len(), 2);

        // Plugins
        let plugins = a.plugins.as_ref().unwrap();
        assert_eq!(plugins["install"][0], "superpowers");

        // Provider settings
        let ps = a.provider_settings.as_ref().unwrap();
        assert_eq!(ps["claude"]["contextWindowTokens"], 100000);
        assert_eq!(ps["cursor"]["tabAutocomplete"], true);

        // Hooks survive round-trip
        assert_eq!(a.hooks.len(), 1);
        assert_eq!(a.hooks[0]["event"], "onSave");
        assert_eq!(a.hooks[0]["command"], "cargo fmt");
    }

    #[test]
    fn icon_from_tags_returns_expected_defaults() {
        let tags = |s: &[&str]| -> Vec<String> { s.iter().map(|t| t.to_string()).collect() };
        assert_eq!(icon_from_tags(&tags(&["testing"])), Some("\u{1f9ea}".into()));
        assert_eq!(icon_from_tags(&tags(&["tdd"])), Some("\u{1f9ea}".into()));
        assert_eq!(icon_from_tags(&tags(&["review"])), Some("\u{1f50d}".into()));
        assert_eq!(icon_from_tags(&tags(&["security"])), Some("\u{1f50d}".into()));
        assert_eq!(icon_from_tags(&tags(&["design"])), Some("\u{1f3a8}".into()));
        assert_eq!(icon_from_tags(&tags(&["ui"])), Some("\u{1f3a8}".into()));
        assert_eq!(icon_from_tags(&tags(&["deploy"])), Some("\u{1f680}".into()));
        assert_eq!(icon_from_tags(&tags(&["release"])), Some("\u{1f680}".into()));
        assert_eq!(icon_from_tags(&tags(&["coordination"])), Some("\u{1f3af}".into()));
        assert_eq!(icon_from_tags(&tags(&["commander"])), Some("\u{1f3af}".into()));
        assert_eq!(icon_from_tags(&tags(&["general"])), None);
        assert_eq!(icon_from_tags(&[]), None);
    }

    #[test]
    fn icon_from_tags_first_match_wins() {
        let tags: Vec<String> = vec!["deploy".into(), "testing".into()];
        // "deploy" appears first, so rocket wins
        assert_eq!(icon_from_tags(&tags), Some("\u{1f680}".into()));
    }

    #[test]
    fn pull_explicit_icon_from_agent() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path().join("project");
        let ship_dir = project.join(".ship");
        let agents_dir = ship_dir.join("agents");
        std::fs::create_dir_all(&agents_dir).unwrap();
        let content = format!(
            r#"{{ "agent": {{ "id": "icon-agent", "name": "Icon Agent", "icon": "{}" }} }}"#,
            "\u{1f525}"
        );
        std::fs::write(agents_dir.join("icon-agent.jsonc"), content).unwrap();

        let raw = pull_agents(&project);
        let resp: compiler::PullResponse = serde_json::from_str(&raw).unwrap();
        assert_eq!(resp.agents[0].profile.icon.as_deref(), Some("\u{1f525}"));
    }

    #[test]
    fn pull_derives_icon_from_tags_when_absent() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path().join("project");
        let ship_dir = project.join(".ship");
        let agents_dir = ship_dir.join("agents");
        std::fs::create_dir_all(&agents_dir).unwrap();
        std::fs::write(
            agents_dir.join("tdd-agent.jsonc"),
            r#"{ "agent": { "id": "tdd-agent", "name": "TDD Agent", "tags": ["testing"] } }"#,
        )
        .unwrap();

        let raw = pull_agents(&project);
        let resp: compiler::PullResponse = serde_json::from_str(&raw).unwrap();
        assert_eq!(resp.agents[0].profile.icon.as_deref(), Some("\u{1f9ea}"));
    }

    #[test]
    fn pull_explicit_icon_overrides_tag_default() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path().join("project");
        let ship_dir = project.join(".ship");
        let agents_dir = ship_dir.join("agents");
        std::fs::create_dir_all(&agents_dir).unwrap();
        let content = format!(
            r#"{{ "agent": {{ "id": "custom-agent", "name": "Custom", "tags": ["testing"], "icon": "{}" }} }}"#,
            "\u{2728}"
        );
        std::fs::write(agents_dir.join("custom-agent.jsonc"), content).unwrap();

        let raw = pull_agents(&project);
        let resp: compiler::PullResponse = serde_json::from_str(&raw).unwrap();
        // Explicit icon wins over tag-derived default
        assert_eq!(resp.agents[0].profile.icon.as_deref(), Some("\u{2728}"));
    }

    #[test]
    fn push_then_pull_round_trips_icon() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();
        std::fs::create_dir_all(project.join(".ship")).unwrap();

        let icon = "\u{1f9ea}";
        let bundle = serde_json::json!({
            "agent": {
                "id": "icon-rt",
                "name": "Icon RT",
                "icon": icon,
                "skill_refs": [],
                "rule_refs": [],
                "mcp_servers": []
            },
            "skills": {},
            "rules": {},
            "dependencies": {}
        });

        let result = push_bundle(project, &serde_json::to_string(&bundle).unwrap());
        assert!(!result.starts_with("Error"), "push failed: {result}");

        let raw = pull_agents(project);
        let resp: compiler::PullResponse = serde_json::from_str(&raw).unwrap();
        assert_eq!(resp.agents[0].profile.icon.as_deref(), Some(icon));
    }
}
