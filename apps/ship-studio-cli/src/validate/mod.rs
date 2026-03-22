//! `ship validate` — check .ship/ config for errors before compile.

use anyhow::Result;
use serde::Serialize;
use std::path::Path;

use crate::loader::load_permission_preset;
use crate::mcp::{McpEntry, McpFile};
use crate::agent_config::AgentConfig;
use crate::profile::find_agent_file;

#[cfg(test)]
mod tests;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ValidationError {
    pub agent: String,
    pub file: String,
    pub error: String,
}

/// Per-agent result: name + collected errors.
#[derive(Debug)]
pub struct AgentReport {
    pub agent_id: String,
    pub errors: Vec<ValidationError>,
}

// ── Public entry points ───────────────────────────────────────────────────────

/// Run validation for one or all agents. Print results. Return Err if any error found.
pub fn run_validate(agent_id: Option<&str>, json: bool, project_root: &Path) -> Result<()> {
    let ship_dir = project_root.join(".ship");
    if !ship_dir.exists() {
        anyhow::bail!(".ship/ not found. Run: ship init");
    }
    let agents_dir = ship_dir.join("agents");

    let reports = if let Some(id) = agent_id {
        let agent_path = find_agent_file(id, project_root)
            .ok_or_else(|| anyhow::anyhow!("Agent '{}' not found", id))?;
        vec![validate_agent(id, &agent_path, &agents_dir)]
    } else {
        validate_all(&agents_dir, project_root)
    };

    let all_errors: Vec<&ValidationError> = reports.iter()
        .flat_map(|r| r.errors.iter())
        .collect();

    if json {
        println!("{}", serde_json::to_string_pretty(&all_errors)?);
        if !all_errors.is_empty() {
            anyhow::bail!("");
        }
        return Ok(());
    }

    let mut any_errors = false;
    for report in &reports {
        if report.errors.is_empty() {
            println!("✓ agent {} — valid", report.agent_id);
        } else {
            any_errors = true;
            println!("✗ agent {} — {} error{}", report.agent_id, report.errors.len(),
                if report.errors.len() == 1 { "" } else { "s" });
            for e in &report.errors {
                println!("  {} — {}", e.file, e.error);
            }
        }
    }

    if any_errors {
        anyhow::bail!("validation failed");
    }
    Ok(())
}

// ── Core validation ───────────────────────────────────────────────────────────

/// Validate all agents found in agents/ (primary) and agents/profiles/ (compat).
fn validate_all(agents_dir: &Path, project_root: &Path) -> Vec<AgentReport> {
    let mut reports = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    // Primary: agents/*.{jsonc,toml} (flat)
    if let Ok(entries) = std::fs::read_dir(agents_dir) {
        let mut paths: Vec<_> = entries.flatten()
            .filter(|e| crate::paths::is_config_ext(&e.path()) && e.path().is_file())
            // Exclude known non-agent config files
            .filter(|e| crate::paths::is_agent_config(&e.path()))
            .collect();
        paths.sort_by_key(|e| e.file_name());
        for entry in paths {
            let path = entry.path();
            let id = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
            seen_ids.insert(id.clone());
            reports.push(validate_agent(&id, &path, agents_dir));
        }
    }

    // Compat: agents/profiles/*.{jsonc,toml}
    let profiles_dir = agents_dir.join("profiles");
    if profiles_dir.exists()
        && let Ok(entries) = std::fs::read_dir(&profiles_dir)
    {
        let mut paths: Vec<_> = entries.flatten()
            .filter(|e| crate::paths::is_config_ext(&e.path()))
            .collect();
        paths.sort_by_key(|e| e.file_name());
        for entry in paths {
            let path = entry.path();
            let id = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
            // Skip if already found in agents/
            if seen_ids.contains(&id) { continue; }
            seen_ids.insert(id.clone());
            reports.push(validate_agent(&id, &path, agents_dir));
        }
    }

    // Also check any compat presets dir
    let presets_dir = agents_dir.join("presets");
    if presets_dir.exists()
        && let Ok(entries) = std::fs::read_dir(&presets_dir)
    {
        let mut paths: Vec<_> = entries.flatten()
            .filter(|e| crate::paths::is_config_ext(&e.path()))
            .collect();
        paths.sort_by_key(|e| e.file_name());
        for entry in paths {
            let path = entry.path();
            let id = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
            // Skip if already found in agents/ or agents/profiles/
            if seen_ids.contains(&id) { continue; }
            reports.push(validate_agent(&id, &path, agents_dir));
        }
    }
    reports
}

/// Validate a single agent at `agent_path` against `agents_dir`.
pub fn validate_agent(agent_id: &str, agent_path: &Path, agents_dir: &Path) -> AgentReport {
    let mut errors = Vec::new();

    // 1. Parse config (JSONC or TOML based on extension)
    let agent = match AgentConfig::load(agent_path) {
        Ok(p) => p,
        Err(e) => {
            errors.push(ValidationError {
                agent: agent_id.to_string(),
                file: agent_path.display().to_string(),
                error: format!("config parse error: {}", e),
            });
            return AgentReport { agent_id: agent_id.to_string(), errors };
        }
    };

    let agent_file = agent_path.display().to_string();

    // 2. Skill refs exist in agents/skills/
    let skills_dir = agents_dir.join("skills");
    for skill_id in &agent.skills.refs {
        if !skill_ref_exists(&skills_dir, skill_id) {
            errors.push(ValidationError {
                agent: agent_id.to_string(),
                file: agent_file.clone(),
                error: format!("skill '{}' not found in agents/skills/", skill_id),
            });
        }
    }

    // 3. MCP server refs exist in mcp config AND have required fields
    let mcp_jsonc = agents_dir.join("mcp.jsonc");
    let mcp_path = if mcp_jsonc.exists() { mcp_jsonc } else { agents_dir.join("mcp.toml") };
    let mcp_file = load_mcp_file(&mcp_path);
    for server_id in &agent.mcp.servers {
        match mcp_file.servers.iter().find(|s| &s.id == server_id) {
            None => errors.push(ValidationError {
                agent: agent_id.to_string(),
                file: "agents/mcp.toml".to_string(),
                error: format!("mcp server '{}' not found in mcp.toml", server_id),
            }),
            Some(s) => {
                if let Some(e) = check_mcp_entry(s) {
                    errors.push(ValidationError {
                        agent: agent_id.to_string(),
                        file: "agents/mcp.toml".to_string(),
                        error: format!("mcp.servers.{} — {}", server_id, e),
                    });
                }
            }
        }
    }

    // 4. Validate all mcp.toml entries (regardless of agent refs)
    for server in &mcp_file.servers {
        if let Some(e) = check_mcp_entry(server) {
            errors.push(ValidationError {
                agent: agent_id.to_string(),
                file: "agents/mcp.toml".to_string(),
                error: format!("mcp.servers.{} — {}", server.id, e),
            });
        }
    }
    // Deduplicate errors from mcp entry checks (a server might be caught twice)
    errors.dedup_by(|a, b| a.file == b.file && a.error == b.error);

    // 5. permissions.preset references a known preset
    if let Some(preset_name) = &agent.permissions.preset {
        if preset_name.trim().is_empty() {
            errors.push(ValidationError {
                agent: agent_id.to_string(),
                file: agent_file.clone(),
                error: "permissions.preset must be a non-empty string".to_string(),
            });
        } else {
            let perm_jsonc = agents_dir.join("permissions.jsonc");
            let perm_path = if perm_jsonc.exists() { perm_jsonc } else { agents_dir.join("permissions.toml") };
            if perm_path.exists() && load_permission_preset(agents_dir, preset_name).is_none() {
                errors.push(ValidationError {
                    agent: agent_id.to_string(),
                    file: "agents/permissions.toml".to_string(),
                    error: format!("preset '{}' not found in permissions.toml", preset_name),
                });
            }
        }
    }

    AgentReport { agent_id: agent_id.to_string(), errors }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn skill_ref_exists(skills_dir: &Path, skill_id: &str) -> bool {
    if !skills_dir.exists() { return false; }
    // Flat: skills/<id>.md
    if skills_dir.join(format!("{}.md", skill_id)).exists() { return true; }
    // Dir: skills/<id>/SKILL.md
    if skills_dir.join(skill_id).join("SKILL.md").exists() { return true; }
    false
}

fn load_mcp_file(path: &Path) -> McpFile {
    McpFile::load(path).unwrap_or_default()
}

/// Returns an error message if the entry is missing required fields; None if valid.
fn check_mcp_entry(s: &McpEntry) -> Option<String> {
    let is_http = s.url.is_some() && s.command.is_none()
        || s.server_type.as_deref() == Some("http")
        || s.server_type.as_deref() == Some("sse");
    if is_http {
        if s.url.as_deref().is_none_or(|u| u.trim().is_empty()) {
            return Some("HTTP/SSE server missing 'url' field".to_string());
        }
    } else {
        // stdio
        if s.command.as_deref().is_none_or(|c| c.trim().is_empty()) {
            return Some("stdio server missing 'command' field".to_string());
        }
    }
    None
}
