use std::path::Path;

pub use compiler::{AgentBundle, SkillBundle, TransferBundle};

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
    let rule_count = bundle.rules.len();

    if let Err(e) = write_bundle(&ship_dir, &bundle) {
        return format!("Error writing bundle: {e}");
    }

    format!(
        "Imported agent '{}': {} skill(s), {} rule(s)",
        agent_id, skill_count, rule_count
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
    for (rule_name, content) in &bundle.rules {
        findings.extend(runtime::security::scan_text(content, &format!("rules/{rule_name}")));
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
    for (rule_name, content) in &bundle.rules {
        write_rule(ship_dir, rule_name, content)?;
    }
    Ok(())
}

/// Write agent JSONC matching the schema structure:
/// { agent: { id, name, ... }, model, skills: { refs }, mcp: { servers }, ... }
fn write_agent(ship_dir: &Path, agent: &AgentBundle) -> Result<(), String> {
    let agents_dir = ship_dir.join("agents");
    std::fs::create_dir_all(&agents_dir).map_err(|e| e.to_string())?;

    let mut root = serde_json::Map::new();

    // agent section
    let mut agent_obj = serde_json::Map::new();
    agent_obj.insert("id".into(), serde_json::json!(agent.id));
    if let Some(ref name) = agent.name {
        agent_obj.insert("name".into(), serde_json::json!(name));
    }
    if let Some(ref desc) = agent.description {
        agent_obj.insert("description".into(), serde_json::json!(desc));
    }
    if let Some(ref version) = agent.version {
        agent_obj.insert("version".into(), serde_json::json!(version));
    }
    if let Some(ref providers) = agent.providers {
        agent_obj.insert("providers".into(), serde_json::json!(providers));
    }
    root.insert("agent".into(), serde_json::Value::Object(agent_obj));

    // top-level optional fields
    if let Some(ref model) = agent.model {
        root.insert("model".into(), serde_json::json!(model));
    }
    if let Some(ref env) = agent.env {
        if !env.is_empty() {
            root.insert("env".into(), serde_json::json!(env));
        }
    }
    if let Some(ref models) = agent.available_models {
        if !models.is_empty() {
            root.insert("available_models".into(), serde_json::json!(models));
        }
    }
    if let Some(ref limits) = agent.agent_limits {
        root.insert("agent_limits".into(), limits.clone());
    }

    // skills section
    if !agent.skill_refs.is_empty() {
        root.insert("skills".into(), serde_json::json!({ "refs": agent.skill_refs }));
    }

    // mcp section
    if !agent.mcp_servers.is_empty() {
        root.insert("mcp".into(), serde_json::json!({ "servers": agent.mcp_servers }));
    }

    // plugins section
    if let Some(ref plugins) = agent.plugins {
        root.insert("plugins".into(), plugins.clone());
    }

    // permissions section
    if let Some(ref perms) = agent.permissions {
        root.insert("permissions".into(), perms.clone());
    }

    // rules section
    let has_refs = !agent.rule_refs.is_empty();
    let has_inline = agent.rules_inline.as_ref().is_some_and(|s| !s.is_empty());
    if has_refs || has_inline {
        let mut rules_obj = serde_json::Map::new();
        if has_refs {
            rules_obj.insert("refs".into(), serde_json::json!(agent.rule_refs));
        }
        if let Some(ref inline) = agent.rules_inline {
            if !inline.is_empty() {
                rules_obj.insert("inline".into(), serde_json::json!(inline));
            }
        }
        root.insert("rules".into(), serde_json::Value::Object(rules_obj));
    }

    // provider_settings section
    if let Some(ref ps) = agent.provider_settings {
        root.insert("provider_settings".into(), ps.clone());
    }

    // hooks section
    if let Some(ref hooks) = agent.hooks {
        if !hooks.is_empty() {
            root.insert("hooks".into(), serde_json::json!(hooks));
        }
    }

    let content = serde_json::to_string_pretty(&root).unwrap_or_else(|_| "{}".into());
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

fn write_rule(ship_dir: &Path, rule_name: &str, content: &str) -> Result<(), String> {
    let rules_dir = ship_dir.join("rules");
    std::fs::create_dir_all(&rules_dir).map_err(|e| e.to_string())?;
    let file_name = if rule_name.ends_with(".md") {
        rule_name.to_string()
    } else {
        format!("{rule_name}.md")
    };
    let dest = rules_dir.join(&file_name);
    std::fs::write(&dest, content)
        .map_err(|e| format!("writing {}: {e}", dest.display()))?;
    Ok(())
}
