use compiler::{ListAgentsResponse, PullAgent, PullMcpServer, PullProfile, PullRule, PullSkill};
use std::path::Path;

pub use super::studio_push::push_bundle;

// ── List ────────────────────────────────────────────────────────────────

pub fn list_local_agents(project_dir: &Path) -> String {
    let agents_dir = project_dir.join(".ship").join("agents");
    if !agents_dir.exists() {
        return serde_json::to_string(&ListAgentsResponse { agents: vec![] })
            .unwrap_or_default();
    }
    let mut ids = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&agents_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if let Some(id) = name.strip_suffix(".jsonc").or_else(|| name.strip_suffix(".toml")) {
                ids.push(id.to_string());
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
    let ship_dir = project_dir.join(".ship");
    if !ship_dir.exists() {
        return serde_json::to_string(&compiler::PullResponse { agents: vec![] })
            .unwrap_or_default();
    }

    let agents_dir = ship_dir.join("agents");
    if !agents_dir.exists() {
        return serde_json::to_string(&compiler::PullResponse { agents: vec![] })
            .unwrap_or_default();
    }

    let mut agents = Vec::new();
    let entries = match std::fs::read_dir(&agents_dir) {
        Ok(e) => e,
        Err(_) => {
            return serde_json::to_string(&compiler::PullResponse { agents: vec![] })
                .unwrap_or_default()
        }
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

        let skill_refs = parsed.skills.map(|s| s.refs).unwrap_or_default();
        let rule_refs = parsed.rules.map(|r| r.refs).unwrap_or_default();
        let mcp_names = parsed.mcp.map(|m| m.servers).unwrap_or_default();

        let skills = resolve_skills(&ship_dir, &skill_refs);
        let rules = resolve_rules(&ship_dir, &rule_refs);
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
        });
    }

    agents.sort_by(|a, b| a.profile.id.cmp(&b.profile.id));
    serde_json::to_string(&compiler::PullResponse { agents }).unwrap_or_default()
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
