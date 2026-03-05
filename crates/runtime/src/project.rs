use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub const SHIP_DIR_NAME: &str = ".ship";

// ── Namespace path helpers ────────────────────────────────────────────────────
// All document paths are derived from these. Never construct paths with raw
// string joins outside of these helpers.

/// `.ship/project/` — vision, notes, ADRs
pub fn project_ns(ship_dir: &Path) -> PathBuf {
    ship_dir.join("project")
}

/// `.ship/workflow/` — features, specs, issues
pub fn workflow_ns(ship_dir: &Path) -> PathBuf {
    ship_dir.join("workflow")
}

/// `.ship/agents/` — modes, skills, prompts
pub fn agents_ns(ship_dir: &Path) -> PathBuf {
    ship_dir.join("agents")
}

/// `.ship/generated/` — runtime-generated/transient artifacts
pub fn generated_ns(ship_dir: &Path) -> PathBuf {
    ship_dir.join("generated")
}

pub fn adrs_dir(ship_dir: &Path) -> PathBuf {
    project_ns(ship_dir).join("adrs")
}

pub fn releases_dir(ship_dir: &Path) -> PathBuf {
    project_ns(ship_dir).join("releases")
}

/// `.ship/project/releases/upcoming/` — planned/active release plans.
pub fn upcoming_releases_dir(ship_dir: &Path) -> PathBuf {
    releases_dir(ship_dir).join("upcoming")
}

pub fn notes_dir(ship_dir: &Path) -> PathBuf {
    project_ns(ship_dir).join("notes")
}

pub fn specs_dir(ship_dir: &Path) -> PathBuf {
    workflow_ns(ship_dir).join("specs")
}

pub fn features_dir(ship_dir: &Path) -> PathBuf {
    project_ns(ship_dir).join("features")
}

pub fn issues_dir(ship_dir: &Path) -> PathBuf {
    workflow_ns(ship_dir).join("issues")
}

pub fn modes_dir(ship_dir: &Path) -> PathBuf {
    agents_ns(ship_dir).join("modes")
}

pub fn skills_dir(ship_dir: &Path) -> PathBuf {
    agents_ns(ship_dir).join("skills")
}

pub fn prompts_dir(ship_dir: &Path) -> PathBuf {
    agents_ns(ship_dir).join("prompts")
}

pub fn rules_dir(ship_dir: &Path) -> PathBuf {
    agents_ns(ship_dir).join("rules")
}

pub fn mcp_config_path(ship_dir: &Path) -> PathBuf {
    agents_ns(ship_dir).join("mcp.toml")
}

pub fn permissions_config_path(ship_dir: &Path) -> PathBuf {
    agents_ns(ship_dir).join("permissions.toml")
}

/// Resolve the enclosing `.ship` directory from any descendant path.
pub fn ship_dir_from_path(path: &Path) -> Option<PathBuf> {
    path.ancestors()
        .find(|ancestor| {
            ancestor
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name == SHIP_DIR_NAME)
        })
        .map(Path::to_path_buf)
}

fn canonicalize_lossy(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn parse_gitdir_pointer(dot_git_file: &Path) -> Option<PathBuf> {
    let content = fs::read_to_string(dot_git_file).ok()?;
    let line = content.lines().next()?.trim();
    let raw = line.strip_prefix("gitdir:")?.trim();
    if raw.is_empty() {
        return None;
    }

    let parsed = PathBuf::from(raw);
    if parsed.is_absolute() {
        Some(parsed)
    } else {
        let base = dot_git_file.parent()?;
        Some(base.join(parsed))
    }
}

fn git_common_dir_from(dot_git_path: &Path) -> Option<PathBuf> {
    if dot_git_path.is_dir() {
        return Some(canonicalize_lossy(dot_git_path));
    }

    let gitdir = canonicalize_lossy(&parse_gitdir_pointer(dot_git_path)?);
    // Worktree pointers usually target: <main>/.git/worktrees/<name>
    // In that case, the common git dir is <main>/.git.
    let worktrees = gitdir.parent()?;
    let marker = worktrees.file_name()?.to_str()?;
    if marker == "worktrees" {
        return worktrees.parent().map(canonicalize_lossy);
    }
    Some(gitdir)
}

fn ship_dir_from_git_worktree(start_dir: &Path) -> Option<PathBuf> {
    for ancestor in start_dir.ancestors() {
        let dot_git = ancestor.join(".git");
        if !dot_git.exists() {
            continue;
        }

        let common_git = git_common_dir_from(&dot_git)?;
        let main_root = common_git.parent()?;
        let ship_candidate = main_root.join(SHIP_DIR_NAME);
        if ship_candidate.exists() && ship_candidate.is_dir() {
            return Some(canonicalize_lossy(&ship_candidate));
        }
    }
    None
}

fn ship_dir_from_git_worktree_pointer(start_dir: &Path) -> Option<PathBuf> {
    for ancestor in start_dir.ancestors() {
        let dot_git = ancestor.join(".git");
        if !dot_git.exists() {
            continue;
        }

        if dot_git.is_dir() {
            return None;
        }

        let gitdir = canonicalize_lossy(&parse_gitdir_pointer(&dot_git)?);
        let worktrees = gitdir.parent()?;
        if worktrees.file_name()?.to_str()? != "worktrees" {
            return None;
        }
        let common_git = canonicalize_lossy(worktrees.parent()?);
        let main_root = common_git.parent()?;
        let ship_candidate = main_root.join(SHIP_DIR_NAME);
        if ship_candidate.exists() && ship_candidate.is_dir() {
            return Some(canonicalize_lossy(&ship_candidate));
        }
        return None;
    }
    None
}

fn resolve_project_dir_from_start(
    start_dir: &Path,
    migrate_legacy: bool,
) -> Result<Option<PathBuf>> {
    if let Some(ship_path) = ship_dir_from_git_worktree_pointer(start_dir) {
        return Ok(Some(ship_path));
    }

    let mut current_dir = start_dir.to_path_buf();
    loop {
        let ship_path = current_dir.join(SHIP_DIR_NAME);
        if ship_path.exists() && ship_path.is_dir() {
            return Ok(Some(canonicalize_lossy(&ship_path)));
        }

        if migrate_legacy {
            let legacy_path = current_dir.join(".project");
            if legacy_path.exists() && legacy_path.is_dir() {
                fs::rename(&legacy_path, &ship_path)
                    .context("Failed to migrate .project to .ship")?;
                return Ok(Some(canonicalize_lossy(&ship_path)));
            }
        }

        if let Some(parent) = current_dir.parent() {
            current_dir = parent.to_path_buf();
        } else {
            break;
        }
    }

    Ok(ship_dir_from_git_worktree(start_dir))
}

/// Resolve the canonical `.ship` directory for a given path without using env
/// overrides and without mutating legacy folders.
pub fn resolve_project_ship_dir(start_dir: &Path) -> Option<PathBuf> {
    resolve_project_dir_from_start(start_dir, false)
        .ok()
        .flatten()
}

/// Resolves the .ship directory by searching upwards from the given directory.
/// Also checks for legacy `.project` and migrates it to `.ship` if found.
/// Supports `SHIP_DIR` environment variable override.
pub fn get_project_dir(start_dir: Option<PathBuf>) -> Result<PathBuf> {
    // 1. Check for environment variable override
    if let Ok(env_path) = env::var("SHIP_DIR") {
        let path = PathBuf::from(env_path);
        if path.exists() && path.is_dir() {
            return Ok(path);
        }
    }

    // 2. Traversal logic — any directory containing a .ship folder is a project.
    // If none is found, attempt git-worktree resolution back to the main checkout.
    let start = start_dir.unwrap_or(env::current_dir()?);
    if let Some(project_dir) = resolve_project_dir_from_start(&start, true)? {
        return Ok(project_dir);
    }

    Err(anyhow!(
        "Project tracking not initialized in this directory or its parents. Run `ship init` to create a .ship directory."
    ))
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct ProjectRegistry {
    pub projects: Vec<ProjectEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Type)]
pub struct ProjectEntry {
    pub name: String,
    #[specta(type = String)]
    pub path: PathBuf,
}

pub fn get_registry_path() -> Result<PathBuf> {
    Ok(get_global_dir()?.join("projects.json"))
}

pub fn load_registry() -> Result<ProjectRegistry> {
    let path = get_registry_path()?;
    if !path.exists() {
        return Ok(ProjectRegistry {
            projects: Vec::new(),
        });
    }
    let content = fs::read_to_string(path)?;
    let registry: ProjectRegistry = serde_json::from_str(&content)?;
    Ok(registry)
}

pub fn save_registry(registry: &ProjectRegistry) -> Result<()> {
    let path = get_registry_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(registry)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn register_project(name: String, path: PathBuf) -> Result<()> {
    let mut registry = load_registry()?;
    let canonical_path = normalize_registry_project_path(&path);

    // De-duplicate entries by canonical path and keep first occurrence.
    let mut seen_target = false;
    registry.projects.retain(|project| {
        let project_path = normalize_registry_project_path(&project.path);
        if project_path == canonical_path {
            if seen_target {
                false
            } else {
                seen_target = true;
                true
            }
        } else {
            true
        }
    });

    if let Some(existing) = registry
        .projects
        .iter_mut()
        .find(|project| normalize_registry_project_path(&project.path) == canonical_path)
    {
        existing.name = name;
        existing.path = canonical_path;
    } else {
        registry.projects.push(ProjectEntry {
            name,
            path: canonical_path,
        });
    }

    save_registry(&registry)?;
    Ok(())
}

pub fn unregister_project(path: PathBuf) -> Result<()> {
    let mut registry = load_registry()?;
    let path = normalize_registry_project_path(&path);
    registry
        .projects
        .retain(|p| normalize_registry_project_path(&p.path) != path);
    save_registry(&registry)?;
    Ok(())
}

pub fn list_registered_projects() -> Result<Vec<ProjectEntry>> {
    let registry = load_registry()?;
    // A git worktree has a `.git` FILE (not a directory) at its root.
    // We never want worktrees to appear as separate projects in the UI,
    // because they share the same `.ship/` data as their main checkout.
    Ok(registry
        .projects
        .into_iter()
        .filter(|entry| {
            let git_path = entry.path.join(".git");
            // Keep: no .git at all (non-git project) OR .git is a directory (real repo)
            !git_path.exists() || git_path.is_dir()
        })
        .collect())
}
/// Returns the global config directory (~/.ship)
pub fn get_global_dir() -> Result<PathBuf> {
    if let Ok(env_path) = env::var("SHIP_GLOBAL_DIR") {
        let path = PathBuf::from(env_path.trim());
        if !path.as_os_str().is_empty() {
            return Ok(path);
        }
    }

    home::home_dir()
        .map(|h| h.join(SHIP_DIR_NAME))
        .ok_or_else(|| anyhow!("Could not find home directory"))
}

// ─── Global App State ─────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, Default, Type)]
pub struct AppState {
    pub active_project: Option<PathBuf>,
    pub recent_projects: Vec<PathBuf>,
}

pub fn load_app_state() -> Result<AppState> {
    let path = get_global_dir()?.join("app_state.json");
    if !path.exists() {
        return Ok(AppState::default());
    }
    let content = fs::read_to_string(path)?;
    serde_json::from_str(&content).context("Failed to parse app state")
}

pub fn save_app_state(state: &AppState) -> Result<()> {
    let path = get_global_dir()?.join("app_state.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(state)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn set_active_project_global(path: PathBuf) -> Result<()> {
    let mut state = load_app_state()?;
    state.active_project = Some(path.clone());

    // Add to recent projects if not already there
    if !state.recent_projects.contains(&path) {
        state.recent_projects.insert(0, path);
        // Keep only last 10
        state.recent_projects.truncate(10);
    }
    save_app_state(&state)
}

pub fn get_active_project_global() -> Result<Option<PathBuf>> {
    let state = load_app_state()?;
    Ok(state.active_project)
}

pub fn get_recent_projects_global() -> Result<Vec<PathBuf>> {
    let state = load_app_state()?;
    Ok(state.recent_projects)
}

fn normalize_registry_project_path(path: &Path) -> PathBuf {
    let canonical = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if canonical
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == SHIP_DIR_NAME)
        .unwrap_or(false)
    {
        return canonical;
    }
    let ship_candidate = canonical.join(SHIP_DIR_NAME);
    if ship_candidate.exists() && ship_candidate.is_dir() {
        fs::canonicalize(&ship_candidate).unwrap_or(ship_candidate)
    } else {
        canonical
    }
}

pub fn sanitize_file_name(name: &str) -> String {
    let mut sanitized = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();

    while sanitized.contains("--") {
        sanitized = sanitized.replace("--", "-");
    }

    sanitized = sanitized.trim_matches('-').to_string();

    if sanitized.len() > 60 {
        sanitized.truncate(60);
        sanitized = sanitized.trim_end_matches('-').to_string();
    }

    sanitized
}

pub fn get_project_name(ship_path: &Path) -> String {
    ship_path
        .parent()
        .and_then(|parent| parent.file_name())
        .and_then(|name| name.to_str())
        .unwrap_or("Unknown")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn resolve_project_ship_dir_follows_git_worktree_pointer() -> Result<()> {
        let tmp = tempdir()?;
        let main_root = tmp.path().join("main");
        let main_ship = main_root.join(".ship");
        let common_git = main_root.join(".git");
        let worktree_git = common_git.join("worktrees").join("feature-auth");
        let wt_root = tmp.path().join("worktrees").join("feature-auth");
        let wt_nested = wt_root.join("src").join("app");

        fs::create_dir_all(&main_ship)?;
        fs::create_dir_all(&worktree_git)?;
        fs::create_dir_all(&wt_nested)?;
        fs::write(
            wt_root.join(".git"),
            format!("gitdir: {}\n", worktree_git.display()),
        )?;

        let resolved = resolve_project_ship_dir(&wt_nested).expect("expected .ship resolution");
        assert_eq!(resolved, canonicalize_lossy(&main_ship));
        Ok(())
    }

    #[test]
    fn resolve_project_ship_dir_prefers_main_ship_over_worktree_copy() -> Result<()> {
        let tmp = tempdir()?;
        let main_root = tmp.path().join("main");
        let main_ship = main_root.join(".ship");
        let common_git = main_root.join(".git");
        let worktree_git = common_git.join("worktrees").join("feature-auth");
        let wt_root = tmp.path().join("worktrees").join("feature-auth");
        let wt_nested = wt_root.join("src").join("app");

        fs::create_dir_all(&main_ship)?;
        fs::create_dir_all(&worktree_git)?;
        fs::create_dir_all(wt_root.join(".ship"))?;
        fs::create_dir_all(&wt_nested)?;
        fs::write(
            wt_root.join(".git"),
            format!("gitdir: {}\n", worktree_git.display()),
        )?;

        let resolved = resolve_project_ship_dir(&wt_nested).expect("expected .ship resolution");
        assert_eq!(resolved, canonicalize_lossy(&main_ship));
        Ok(())
    }

    #[test]
    fn resolve_project_ship_dir_follows_relative_git_worktree_pointer() -> Result<()> {
        let tmp = tempdir()?;
        let main_root = tmp.path().join("main");
        let main_ship = main_root.join(".ship");
        let common_git = main_root.join(".git");
        let worktree_git = common_git.join("worktrees").join("feature-auth");
        let wt_root = tmp.path().join("worktrees").join("feature-auth");
        let wt_nested = wt_root.join("src");

        fs::create_dir_all(&main_ship)?;
        fs::create_dir_all(&worktree_git)?;
        fs::create_dir_all(&wt_nested)?;

        // Use a relative pointer to mirror setups where worktree metadata is not
        // expressed as an absolute path.
        fs::write(
            wt_root.join(".git"),
            "gitdir: ../../main/.git/worktrees/feature-auth\n",
        )?;

        let resolved = resolve_project_ship_dir(&wt_nested).expect("expected .ship resolution");
        assert_eq!(resolved, canonicalize_lossy(&main_ship));
        Ok(())
    }
}

pub fn register_ship_namespace(
    ship_path: &Path,
    namespace: crate::config::NamespaceConfig,
) -> Result<()> {
    let mut config = crate::config::get_config(Some(ship_path.to_path_buf()))?;
    if let Some(existing) = config
        .namespaces
        .iter_mut()
        .find(|entry| entry.id == namespace.id)
    {
        *existing = namespace;
    } else {
        config.namespaces.push(namespace);
    }
    crate::config::save_config(&config, Some(ship_path.to_path_buf()))?;
    crate::config::ensure_registered_namespaces(ship_path, &config.namespaces)
}

fn write_if_missing(path: &Path, content: &str) -> Result<()> {
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

/// Lightweight project bootstrap for runtime unit tests.
/// The full production project scaffolding lives in `ship-module-project`.
pub fn init_project(base_dir: PathBuf) -> Result<PathBuf> {
    let ship_path = base_dir.join(SHIP_DIR_NAME);
    fs::create_dir_all(&ship_path)?;

    for rel in [
        "project/adrs",
        "project/features",
        "project/releases",
        "project/notes",
        "workflow/issues/backlog",
        "workflow/issues/in-progress",
        "workflow/issues/review",
        "workflow/issues/blocked",
        "workflow/issues/done",
        "workflow/specs",
        "agents/modes",
        "agents/skills/task-policy",
        "agents/prompts",
        "generated",
    ] {
        fs::create_dir_all(ship_path.join(rel))?;
    }

    write_if_missing(
        &ship_path.join("project/features/TEMPLATE.md"),
        "+++\nrelease_id = \"\"\n+++\n\n## Why\n\n## Delivery Todos\n",
    )?;
    write_if_missing(
        &ship_path.join("project/releases/TEMPLATE.md"),
        "+++\nversion = \"\"\n+++\n\n## Scope\n",
    )?;
    write_if_missing(
        &ship_path.join("project/notes/TEMPLATE.md"),
        "+++\ntitle = \"\"\n+++\n\n",
    )?;
    write_if_missing(
        &ship_path.join("project/TEMPLATE.md"),
        "# Vision\n\nDescribe what this project is trying to achieve.\n",
    )?;
    write_if_missing(
        &ship_path.join("project/vision.md"),
        "# Vision\n\nDescribe what this project is trying to achieve.\n",
    )?;
    write_if_missing(&ship_path.join("README.md"), "# Ship Project\n")?;
    write_if_missing(
        &ship_path.join("project/README.md"),
        "# Project Namespace\n",
    )?;
    write_if_missing(
        &ship_path.join("workflow/README.md"),
        "# Workflow Namespace\n",
    )?;
    write_if_missing(
        &ship_path.join("agents/modes/planning.toml"),
        "id = \"planning\"\nname = \"Planning\"\n",
    )?;
    write_if_missing(
        &ship_path.join("agents/modes/execution.toml"),
        "id = \"execution\"\nname = \"Execution\"\n",
    )?;
    crate::events::ensure_event_log(&ship_path)?;
    write_if_missing(
        &ship_path.join("agents/skills/task-policy/index.md"),
        "# task-policy\n\nShipwright Workflow Policy\n",
    )?;
    write_if_missing(
        &ship_path.join("agents/skills/task-policy/skill.toml"),
        "id = \"task-policy\"\nname = \"Task Policy\"\n",
    )?;

    if !ship_path.join(crate::config::PRIMARY_CONFIG_FILE).exists() {
        let config = crate::config::ProjectConfig::default();
        crate::config::save_config(&config, Some(ship_path.clone()))?;
    }

    let config = crate::config::get_config(Some(ship_path.clone()))?;
    crate::config::ensure_registered_namespaces(&ship_path, &config.namespaces)?;
    crate::config::generate_gitignore(&ship_path, &config.git)?;

    Ok(ship_path)
}
