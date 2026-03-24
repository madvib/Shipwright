use serde::Deserialize;
use std::path::Path;

/// Transfer bundle sent from Studio to the local CLI via MCP.
#[derive(Debug, Deserialize)]
pub struct TransferBundle {
    pub agent: AgentBundle,
    #[serde(default)]
    pub dependencies: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub skills: std::collections::HashMap<String, SkillBundle>,
}

#[derive(Debug, Deserialize)]
pub struct AgentBundle {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub rules: Vec<String>,
    #[serde(default)]
    pub mcp_servers: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct SkillBundle {
    pub files: std::collections::HashMap<String, String>,
}

pub fn push_bundle(project_dir: &Path, bundle_json: &str) -> String {
    let bundle: TransferBundle = match serde_json::from_str(bundle_json) {
        Ok(b) => b,
        Err(e) => return format!("Error: invalid bundle JSON: {e}"),
    };

    let ship_dir = project_dir.join(".ship");
    if !ship_dir.exists() {
        return "Error: .ship/ not found. Run `ship init` first.".into();
    }

    if let Err(e) = scan_bundle(&bundle) {
        return format!("Error: {e}");
    }

    let agent_id = bundle.agent.id.clone();
    let skill_count = bundle.skills.len();
    let dep_count = bundle.dependencies.len();

    if let Err(e) = write_bundle(&ship_dir, &bundle) {
        return format!("Error writing bundle: {e}");
    }

    format!(
        "Imported agent '{}': {} skill(s), {} dep(s)",
        agent_id, skill_count, dep_count
    )
}

fn scan_bundle(bundle: &TransferBundle) -> Result<(), String> {
    let mut findings = Vec::new();
    for (skill_id, skill) in &bundle.skills {
        for (path, content) in &skill.files {
            let filename = format!("skills/{skill_id}/{path}");
            findings.extend(runtime::security::scan_text(content, &filename));
        }
    }
    for (i, rule) in bundle.agent.rules.iter().enumerate() {
        findings.extend(runtime::security::scan_text(rule, &format!("rule[{i}]")));
    }
    if runtime::security::has_critical(&findings) {
        let critical: Vec<String> = findings
            .iter()
            .filter(|f| f.severity == runtime::security::Severity::Critical)
            .map(|f| f.to_string())
            .collect();
        return Err(format!(
            "security scan blocked import: {} critical finding(s): {}",
            critical.len(),
            critical.join("; ")
        ));
    }
    Ok(())
}

fn write_bundle(ship_dir: &Path, bundle: &TransferBundle) -> Result<(), String> {
    write_agent(ship_dir, &bundle.agent)?;
    for (skill_id, skill) in &bundle.skills {
        write_skill(ship_dir, skill_id, skill)?;
    }
    Ok(())
}

fn write_agent(ship_dir: &Path, agent: &AgentBundle) -> Result<(), String> {
    let agents_dir = ship_dir.join("agents");
    std::fs::create_dir_all(&agents_dir).map_err(|e| e.to_string())?;

    let mut obj = serde_json::Map::new();
    obj.insert("id".into(), serde_json::json!(agent.id));
    if let Some(ref name) = agent.name {
        obj.insert("name".into(), serde_json::json!(name));
    }
    if let Some(ref desc) = agent.description {
        obj.insert("description".into(), serde_json::json!(desc));
    }
    if let Some(ref model) = agent.model {
        obj.insert("model".into(), serde_json::json!(model));
    }
    if !agent.skills.is_empty() {
        obj.insert("skills".into(), serde_json::json!(agent.skills));
    }
    if !agent.rules.is_empty() {
        obj.insert("rules".into(), serde_json::json!(agent.rules));
    }
    if !agent.mcp_servers.is_empty() {
        obj.insert("mcp_servers".into(), serde_json::json!(agent.mcp_servers));
    }

    let content = serde_json::to_string_pretty(&obj).unwrap_or_else(|_| "{}".into());
    let dest = agents_dir.join(format!("{}.jsonc", agent.id));
    std::fs::write(&dest, content).map_err(|e| format!("writing {}: {e}", dest.display()))?;
    Ok(())
}

fn write_skill(ship_dir: &Path, skill_id: &str, skill: &SkillBundle) -> Result<(), String> {
    let skill_dir = ship_dir.join("skills").join(skill_id);
    std::fs::create_dir_all(&skill_dir).map_err(|e| e.to_string())?;
    for (rel_path, content) in &skill.files {
        let dest = skill_dir.join(rel_path);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(&dest, content)
            .map_err(|e| format!("writing {}: {e}", dest.display()))?;
    }
    Ok(())
}
