use std::fs;
use std::path::PathBuf;

// ── Global (~/.ship/) ─────────────────────────────────────────────────────────

pub fn global_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".ship")
}

pub fn global_modes_dir() -> PathBuf { global_dir().join("modes") }
pub fn global_skills_dir() -> PathBuf { global_dir().join("skills") }
pub fn global_mcp_dir() -> PathBuf { global_dir().join("mcp") }
pub fn global_cache_dir() -> PathBuf { global_dir().join("cache") }
pub fn global_mcp_registry() -> PathBuf { global_mcp_dir().join("registry.toml") }

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
pub fn project_profiles_dir() -> PathBuf { agents_dir().join("profiles") }
/// Compat: also check agents/presets/ for projects not yet migrated.
pub fn project_presets_dir() -> PathBuf { agents_dir().join("presets") }
pub fn agents_dir() -> PathBuf { project_dir().join("agents") }
pub fn agents_rules_dir() -> PathBuf { agents_dir().join("rules") }
pub fn agents_skills_dir() -> PathBuf { agents_dir().join("skills") }
pub fn agents_mcp_path() -> PathBuf { agents_dir().join("mcp.toml") }
pub fn agents_permissions_path() -> PathBuf { agents_dir().join("permissions.toml") }
pub fn agents_hooks_path() -> PathBuf { agents_dir().join("hooks.toml") }
pub fn project_ship_toml() -> PathBuf { project_dir().join("ship.toml") }

/// Returns the absolute path to `.ship/` in the current directory, or errors.
pub fn project_ship_dir_required() -> anyhow::Result<std::path::PathBuf> {
    let cwd = std::env::current_dir()?;
    let ship_dir = cwd.join(".ship");
    if !ship_dir.exists() {
        anyhow::bail!(".ship/ not found in {}. Run: ship init", cwd.display());
    }
    Ok(ship_dir)
}

pub fn ensure_project_dirs() -> anyhow::Result<()> {
    for dir in [project_dir(), project_modes_dir(), agents_dir(),
                agents_rules_dir(), agents_skills_dir(),
                project_profiles_dir(), project_presets_dir()] {
        fs::create_dir_all(dir)?;
    }
    Ok(())
}

/// Find a mode file by ID: project first, then global.
pub fn find_mode_file(id: &str) -> Option<PathBuf> {
    let p = project_modes_dir().join(format!("{}.toml", id));
    if p.exists() { return Some(p); }
    let g = global_modes_dir().join(format!("{}.toml", id));
    if g.exists() { return Some(g); }
    None
}

/// Return (mode_id, scope) pairs from project + global dirs.
pub fn list_mode_ids(local_only: bool, project_only: bool) -> Vec<(String, &'static str)> {
    let mut modes = Vec::new();
    if !local_only {
        if let Ok(entries) = fs::read_dir(project_modes_dir()) {
            for e in entries.flatten() {
                if e.path().extension().map_or(false, |x| x == "toml") {
                    modes.push((e.path().file_stem().unwrap().to_string_lossy().to_string(), "project"));
                }
            }
        }
    }
    if !project_only {
        if let Ok(entries) = fs::read_dir(global_modes_dir()) {
            for e in entries.flatten() {
                if e.path().extension().map_or(false, |x| x == "toml") {
                    modes.push((e.path().file_stem().unwrap().to_string_lossy().to_string(), "global"));
                }
            }
        }
    }
    modes
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
        assert_eq!(agents_mcp_path(), PathBuf::from(".ship/agents/mcp.toml"));
    }
}
