use std::path::Path;

use runtime::{get_skill_vars, list_effective_skills, list_skill_vars, set_skill_var};

use crate::requests::{GetSkillVarsRequest, ListSkillVarsRequest, ListSkillsRequest, SetSkillVarRequest};

pub fn list_skills(project_dir: &Path, req: ListSkillsRequest) -> String {
    let skills = match list_effective_skills(project_dir) {
        Ok(s) => s,
        Err(e) => return format!("Error listing skills: {}", e),
    };
    let filtered: Vec<_> = if let Some(ref query) = req.query {
        let q = query.to_ascii_lowercase();
        skills
            .into_iter()
            .filter(|s| {
                s.id.to_ascii_lowercase().contains(&q)
                    || s.name.to_ascii_lowercase().contains(&q)
                    || s.description
                        .as_deref()
                        .unwrap_or("")
                        .to_ascii_lowercase()
                        .contains(&q)
            })
            .collect()
    } else {
        skills
    };
    if filtered.is_empty() {
        return "No skills found.".to_string();
    }
    let mut out = String::from("Skills:\n");
    for s in &filtered {
        let desc = s.description.as_deref().unwrap_or("(no description)");
        out.push_str(&format!("- {} — {} — {}\n", s.id, s.name, desc));
    }
    out
}

/// `get_skill_vars` MCP tool — return merged variable state for a skill.
pub fn get_skill_vars_tool(ship_dir: &Path, req: GetSkillVarsRequest) -> String {
    match get_skill_vars(ship_dir, &req.skill_id) {
        Ok(Some(vars)) => {
            match serde_json::to_string_pretty(&vars) {
                Ok(json) => json,
                Err(e) => format!("Error serializing vars: {e}"),
            }
        }
        Ok(None) => format!(
            "Skill '{}' has no vars.json — no variables configured.",
            req.skill_id
        ),
        Err(e) => format!("Error reading vars for '{}': {e}", req.skill_id),
    }
}

/// `set_skill_var` MCP tool — write a single variable value.
pub fn set_skill_var_tool(ship_dir: &Path, req: SetSkillVarRequest) -> String {
    let value: serde_json::Value = match serde_json::from_str(&req.value_json) {
        Ok(v) => v,
        Err(e) => {
            return format!(
                "Invalid JSON value '{}': {e}\nPass values as JSON — e.g. '\"gitmoji\"' for strings, 'true' for bools.",
                req.value_json
            );
        }
    };

    match set_skill_var(ship_dir, &req.skill_id, &req.key, value) {
        Ok(()) => format!("set {}.{} = {}", req.skill_id, req.key, req.value_json),
        Err(e) => format!("Error: {e}"),
    }
}

/// `list_skill_vars` MCP tool — list all skills with configured variables.
pub fn list_skill_vars_tool(ship_dir: &Path, req: ListSkillVarsRequest) -> String {
    let all = match list_skill_vars(ship_dir) {
        Ok(v) => v,
        Err(e) => return format!("Error listing skill vars: {e}"),
    };

    let filtered: Vec<_> = if let Some(ref id) = req.skill_id {
        all.into_iter().filter(|(k, _)| k == id).collect()
    } else {
        all
    };

    if filtered.is_empty() {
        return "No skills with vars found.".to_string();
    }

    let mut out = String::new();
    for (skill_id, vars) in &filtered {
        out.push_str(&format!("{}:\n", skill_id));
        let mut keys: Vec<&String> = vars.keys().collect();
        keys.sort();
        for k in keys {
            let v = serde_json::to_string(&vars[k]).unwrap_or_else(|_| "null".to_string());
            out.push_str(&format!("  {} = {}\n", k, v));
        }
    }
    out
}
