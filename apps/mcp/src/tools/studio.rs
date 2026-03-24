use compiler::{ListAgentsResponse, PullAgent, PullMcpServer, PullProfile, PullRule, PullSkill};
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
            if let Some(id) = name.strip_suffix(".jsonc").or_else(|| name.strip_suffix(".toml")) {
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

/// JSONC agent file shape (nested, with comments stripped by compiler).
#[derive(Debug, serde::Deserialize)]
struct AgentJsonc {
    agent: AgentJsoncProfile,
    #[serde(default)]
    skills: Option<AgentJsoncRefs>,
    #[serde(default)]
    mcp: Option<AgentJsoncMcp>,
    #[serde(default)]
    permissions: Option<serde_json::Value>,
    #[serde(default)]
    rules: Option<AgentJsoncRefs>,
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
        let rule_refs = parsed.rules.map(|r| r.refs).unwrap_or_default();
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

        agents.push(PullAgent {
            profile: PullProfile {
                id: parsed.agent.id.clone(),
                name: parsed.agent.name.unwrap_or_else(|| parsed.agent.id.clone()),
                description: parsed.agent.description.unwrap_or_default(),
                providers: parsed.agent.providers.unwrap_or_else(|| vec!["claude".into()]),
                version: parsed.agent.version.unwrap_or_else(|| "0.1.0".into()),
            },
            skills,
            mcp_servers,
            rules,
            hooks: vec![],
            permissions: parsed.permissions,
            source: source.into(),
        });
    }
}

fn resolve_skills(ship_dir: &Path, refs: &[String]) -> Vec<PullSkill> {
    let skills_dir = ship_dir.join("skills");
    refs.iter()
        .filter_map(|r| {
            let id = r.rsplit('/').next().unwrap_or(r);
            let skill_md = skills_dir.join(id).join("SKILL.md");
            let content = std::fs::read_to_string(&skill_md).ok()?;
            let (name, description) = parse_skill_frontmatter(&content);
            Some(PullSkill {
                id: id.to_string(),
                name: name.unwrap_or_else(|| id.to_string()),
                description,
                content,
                source: "imported".into(),
            })
        })
        .collect()
}

fn parse_skill_frontmatter(content: &str) -> (Option<String>, Option<String>) {
    if !content.starts_with("---") {
        return (None, None);
    }
    let rest = &content[3..];
    let end = match rest.find("\n---") {
        Some(i) => i,
        None => return (None, None),
    };
    let fm = &rest[..end];
    let mut name = None;
    let mut desc = None;
    for line in fm.lines() {
        if let Some(v) = line.strip_prefix("name:") {
            name = Some(v.trim().to_string());
        } else if let Some(v) = line.strip_prefix("description:") {
            desc = Some(v.trim().to_string());
        }
    }
    (name, desc)
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

    fn write_agent(dir: &Path, id: &str) {
        let agents = dir.join("agents");
        std::fs::create_dir_all(&agents).unwrap();
        let content = format!(
            r#"{{ "agent": {{ "id": "{id}", "name": "{id}", "providers": ["claude"] }} }}"#,
        );
        std::fs::write(agents.join(format!("{id}.jsonc")), content).unwrap();
    }

    #[test]
    fn pull_agents_tags_source() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path().join("project");
        std::fs::create_dir_all(&project).unwrap();
        write_agent(&project.join(".ship"), "agent-a");

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

        write_agent(&project.join(".ship"), "shared-agent");
        write_agent(&library, "shared-agent");
        write_agent(&library, "lib-only");

        // Test with explicit dirs (bypassing library_dir() which reads global)
        let mut agents = Vec::new();
        let mut seen = std::collections::HashSet::new();
        pull_agents_from_dir(&project.join(".ship"), "project", &mut agents, &mut seen);
        pull_agents_from_dir(&library, "library", &mut agents, &mut seen);

        agents.sort_by(|a, b| a.profile.id.cmp(&b.profile.id));
        assert_eq!(agents.len(), 2);
        let shared = agents.iter().find(|a| a.profile.id == "shared-agent").unwrap();
        assert_eq!(shared.source, "project", "project should shadow library");
        let lib = agents.iter().find(|a| a.profile.id == "lib-only").unwrap();
        assert_eq!(lib.source, "library");
    }

    #[test]
    fn collect_agent_ids_handles_missing_dir() {
        let ids = collect_agent_ids(Path::new("/nonexistent/path"));
        assert!(ids.is_empty());
    }

    #[test]
    fn list_local_agents_project_only() {
        let tmp = tempfile::tempdir().unwrap();
        write_agent(&tmp.path().join(".ship"), "my-agent");
        let raw = list_local_agents(tmp.path());
        let resp: ListAgentsResponse = serde_json::from_str(&raw).unwrap();
        assert!(resp.agents.contains(&"my-agent".to_string()));
    }
}
