use std::fs;
use std::path::PathBuf;

// ── Global (~/.ship/) ─────────────────────────────────────────────────────────

pub fn global_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".ship")
}

pub fn global_modes_dir() -> PathBuf { global_dir().join("modes") }
pub fn global_agents_dir() -> PathBuf { global_dir().join("agents") }
pub fn global_skills_dir() -> PathBuf { global_dir().join("skills") }
pub fn global_mcp_dir() -> PathBuf { global_dir().join("mcp") }
pub fn global_cache_dir() -> PathBuf { global_dir().join("cache") }

pub fn ensure_global_dirs() -> anyhow::Result<()> {
    for dir in [global_dir(), global_modes_dir(), global_skills_dir(),
                global_mcp_dir(), global_cache_dir()] {
        fs::create_dir_all(dir)?;
    }
    Ok(())
}

// ── Project (.ship/) ──────────────────────────────────────────────────────────

pub fn project_dir() -> PathBuf { PathBuf::from(".ship") }
pub fn project_modes_dir() -> PathBuf { project_dir().join("modes") }
pub fn agents_dir() -> PathBuf { project_dir().join("agents") }
pub fn rules_dir() -> PathBuf { project_dir().join("rules") }
pub fn skills_dir() -> PathBuf { project_dir().join("skills") }
pub fn mcp_path() -> PathBuf { project_dir().join("mcp.jsonc") }
pub fn project_ship_toml() -> PathBuf { project_dir().join("ship.toml") }
pub fn project_ship_jsonc() -> PathBuf { project_dir().join("ship.jsonc") }

/// Returns true if the path has a .jsonc or .json extension.
pub fn is_config_ext(path: &std::path::Path) -> bool {
    path.extension().is_some_and(|x| x == "toml" || x == "jsonc" || x == "json")
}

/// Returns true if the path has a .jsonc or .json extension (not .toml).
pub fn is_jsonc_ext(path: &std::path::Path) -> bool {
    path.extension().is_some_and(|x| x == "jsonc" || x == "json")
}

/// Returns true if the path is an agent config file (not mcp.* or permissions.*).
pub fn is_agent_config(path: &std::path::Path) -> bool {
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    stem != "mcp" && stem != "permissions"
}

/// Returns the absolute path to `.ship/` for the current project, or errors.
/// Uses git-worktree-aware traversal so this works from subdirs and worktrees.
pub fn project_ship_dir_required() -> anyhow::Result<std::path::PathBuf> {
    runtime::project::get_project_dir(None)
}

pub fn ensure_project_dirs() -> anyhow::Result<()> {
    for dir in [project_dir(), project_modes_dir(), agents_dir(),
                rules_dir(), skills_dir()] {
        fs::create_dir_all(dir)?;
    }
    Ok(())
}


/// Return (agent_id, scope) pairs from project + global dirs.
/// Scans both `.jsonc` and `.toml` files; `.jsonc` takes precedence.
pub fn list_agent_ids(local_only: bool, project_only: bool) -> Vec<(String, &'static str)> {
    let mut agents = Vec::new();
    // Project agents: .ship/agents/*.{jsonc,toml} (flat)
    if !local_only {
        if let Ok(entries) = fs::read_dir(agents_dir()) {
            for e in entries.flatten() {
                let path = e.path();
                if is_config_ext(&path) && path.is_file() && is_agent_config(&path) {
                    let name = path.file_stem().unwrap().to_string_lossy().to_string();
                    if !agents.iter().any(|(id, _)| id == &name) {
                        agents.push((name, "project"));
                    }
                }
            }
        }
        // Also check legacy modes dir
        if let Ok(entries) = fs::read_dir(project_modes_dir()) {
            for e in entries.flatten() {
                let path = e.path();
                if is_config_ext(&path) {
                    let name = path.file_stem().unwrap().to_string_lossy().to_string();
                    if !agents.iter().any(|(id, _)| id == &name) {
                        agents.push((name, "project"));
                    }
                }
            }
        }
    }
    // Global agents
    if !project_only
        && let Ok(entries) = fs::read_dir(global_modes_dir()) {
            for e in entries.flatten() {
                let path = e.path();
                if is_config_ext(&path) {
                    let name = path.file_stem().unwrap().to_string_lossy().to_string();
                    if !agents.iter().any(|(id, _)| id == &name) {
                        agents.push((name, "global"));
                    }
                }
            }
        }
    agents
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_paths_under_home_ship() {
        let g = global_dir();
        assert!(g.to_string_lossy().ends_with(".ship"));
        assert_eq!(global_modes_dir(), g.join("modes"));
        assert_eq!(global_skills_dir(), g.join("skills"));
    }

    #[test]
    fn project_paths_are_relative() {
        assert_eq!(project_dir(), PathBuf::from(".ship"));
        assert_eq!(agents_dir(), PathBuf::from(".ship/agents"));
        assert_eq!(mcp_path(), PathBuf::from(".ship/mcp.jsonc"));
    }
}
