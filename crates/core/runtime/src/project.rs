use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::OnceLock;

pub const SHIP_DIR_NAME: &str = ".ship";
const TEST_GLOBAL_DIR_PREFIX: &str = "ship-test-global-";

static TEST_GLOBAL_CLEANUP_PATH: OnceLock<PathBuf> = OnceLock::new();
#[cfg(unix)]
static TEST_GLOBAL_CLEANUP_REGISTERED: OnceLock<()> = OnceLock::new();

#[cfg(unix)]
unsafe extern "C" {
    fn atexit(cb: extern "C" fn()) -> i32;
    fn kill(pid: i32, sig: i32) -> i32;
}

// ── Namespace path helpers ────────────────────────────────────────────────────
// All document paths are derived from these. Never construct paths with raw
// string joins outside of these helpers.

/// `.ship/project/` — specs, features, releases, notes, ADRs
pub fn project_ns(ship_dir: &Path) -> PathBuf {
    ship_dir.join("project")
}

/// `.ship/agents/` — rules, permissions, MCP config
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
    project_ns(ship_dir).join("specs")
}

pub fn features_dir(ship_dir: &Path) -> PathBuf {
    project_ns(ship_dir).join("features")
}

pub fn modes_dir(ship_dir: &Path) -> PathBuf {
    agents_ns(ship_dir).join("modes")
}

pub fn skills_dir(ship_dir: &Path) -> PathBuf {
    project_skills_dir(ship_dir)
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

/// `.ship/vision.md` — project north-star document
pub fn vision_doc_path(ship_dir: &Path) -> PathBuf {
    ship_dir.join("vision.md")
}

/// Legacy path for vision template (`.ship/TEMPLATE.md`).
/// New projects should rely on `.ship/vision.md` as the canonical document.
pub fn vision_template_path(ship_dir: &Path) -> PathBuf {
    ship_dir.join("TEMPLATE.md")
}

/// Derive a stable, filesystem-safe project slug from a .ship path.
/// Uses "<project-name>-<project-id>" (or "tmp-..." for transient test projects).
pub fn project_slug_from_ship_dir(ship_dir: &Path) -> String {
    project_namespace_slug(ship_dir)
}

fn project_namespace_slug(ship_dir: &Path) -> String {
    let project_root = project_root_from_ship_dir(ship_dir);
    let mut identity = read_project_identity_from_toml(ship_dir).unwrap_or_default();

    if identity.id.is_none()
        && let Ok(id) = ensure_project_id(ship_dir)
    {
        identity.id = Some(id);
    }

    let name_token = project_name_token(project_root, identity.name.as_deref());
    let id_token = project_id_token(ship_dir, identity.id.as_deref());
    let transient_prefix = if is_transient_project_root(project_root) {
        "tmp-"
    } else {
        ""
    };

    format!("{transient_prefix}{name_token}-{id_token}")
}

fn project_root_from_ship_dir(ship_dir: &Path) -> &Path {
    if ship_dir
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == SHIP_DIR_NAME)
    {
        ship_dir.parent().unwrap_or(ship_dir)
    } else {
        ship_dir
    }
}

#[derive(Debug, Default)]
struct ProjectIdentity {
    id: Option<String>,
    name: Option<String>,
}

fn read_project_identity_from_toml(ship_dir: &Path) -> Option<ProjectIdentity> {
    let path = ship_dir.join(crate::config::PRIMARY_CONFIG_FILE);
    let content = fs::read_to_string(path).ok()?;
    let parsed: toml::Value = toml::from_str(&content).ok()?;

    let id = parsed
        .get("id")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let name = parsed
        .get("name")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    Some(ProjectIdentity { id, name })
}

fn project_name_token(project_root: &Path, configured_name: Option<&str>) -> String {
    let raw = configured_name
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            project_root
                .file_name()
                .and_then(|name| name.to_str())
                .map(ToOwned::to_owned)
        })
        .unwrap_or_else(|| "project".to_string());

    let mut token = sanitize_file_name(&raw);
    if token.len() > 40 {
        token.truncate(40);
        token = token.trim_end_matches('-').to_string();
    }
    if token.is_empty() {
        "project".to_string()
    } else {
        token
    }
}

fn project_id_token(ship_dir: &Path, configured_id: Option<&str>) -> String {
    let raw = configured_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| legacy_path_slug_from_ship_dir(ship_dir));

    let mut token = sanitize_file_name(&raw);
    if token.len() > 16 {
        token.truncate(16);
        token = token.trim_end_matches('-').to_string();
    }
    if token.is_empty() {
        "unknown".to_string()
    } else {
        token
    }
}

fn project_components(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(value.to_string_lossy().to_ascii_lowercase()),
            _ => None,
        })
        .collect()
}

fn starts_with_components(path: &Path, components: &[&str]) -> bool {
    let normalized = project_components(path);
    if normalized.len() < components.len() {
        return false;
    }
    normalized
        .iter()
        .take(components.len())
        .zip(components.iter())
        .all(|(lhs, rhs)| lhs == rhs)
}

fn contains_component_sequence(path: &Path, sequence: &[&str]) -> bool {
    let normalized = project_components(path);
    if sequence.is_empty() || normalized.len() < sequence.len() {
        return false;
    }
    normalized.windows(sequence.len()).any(|window| {
        window
            .iter()
            .zip(sequence.iter())
            .all(|(lhs, rhs)| lhs == rhs)
    })
}

fn is_transient_project_root(project_root: &Path) -> bool {
    let canonical = fs::canonicalize(project_root).unwrap_or_else(|_| project_root.to_path_buf());
    starts_with_components(&canonical, &["tmp"])
        || starts_with_components(&canonical, &["private", "tmp"])
        || starts_with_components(&canonical, &["var", "folders"])
        || starts_with_components(&canonical, &["private", "var", "folders"])
        || contains_component_sequence(&canonical, &["appdata", "local", "temp"])
        || contains_component_sequence(&canonical, &["examples", "e2e"])
        || contains_component_sequence(&canonical, &["examples", "projects-e2e"])
        || contains_component_sequence(&canonical, &["target", "tmp"])
}

fn legacy_path_slug_from_ship_dir(ship_dir: &Path) -> String {
    let project_root = project_root_from_ship_dir(ship_dir);
    let canonical =
        std::fs::canonicalize(project_root).unwrap_or_else(|_| project_root.to_path_buf());
    let raw = canonical.to_string_lossy();
    let slug: String = raw
        .trim_start_matches('/')
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    let slug = slug
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if slug.is_empty() {
        "unknown-project".to_string()
    } else {
        slug
    }
}

/// Ensure a project has a persistent `ship.toml:id` and return it.
pub fn ensure_project_id(ship_dir: &Path) -> Result<String> {
    if let Some(identity) = read_project_identity_from_toml(ship_dir)
        && let Some(id) = identity.id
    {
        return Ok(id);
    }

    let path = ship_dir.join(crate::config::PRIMARY_CONFIG_FILE);
    let content = fs::read_to_string(&path).with_context(|| {
        format!(
            "Project at {} has no readable ship.toml. Re-run `ship init` to initialize it.",
            ship_dir.display()
        )
    })?;

    let mut parsed: toml::Value = toml::from_str(&content).with_context(|| {
        format!(
            "Project at {} has an invalid ship.toml. Re-run `ship init` or fix the file.",
            ship_dir.display()
        )
    })?;

    let table = parsed
        .as_table_mut()
        .ok_or_else(|| anyhow!("ship.toml must contain a top-level table."))?;

    let generated_id = crate::gen_nanoid();
    table.insert("id".to_string(), toml::Value::String(generated_id.clone()));

    let updated = toml::to_string_pretty(&parsed)?;
    crate::fs_util::write_atomic(&path, updated)?;

    Ok(generated_id)
}

/// Global/shared skills store: `~/.ship/skills/`
pub fn user_skills_dir() -> PathBuf {
    get_global_dir()
        .unwrap_or_else(|_| PathBuf::from(".ship"))
        .join("skills")
}

/// Project-scoped skills store: `.ship/agents/skills/`
pub fn project_skills_dir(ship_dir: &Path) -> PathBuf {
    agents_ns(ship_dir).join("skills")
}

/// Legacy project-scoped skills store used by older builds: `.ship/skills/`
pub fn legacy_repo_project_skills_dir(ship_dir: &Path) -> PathBuf {
    ship_dir.join("skills")
}

/// Legacy project-scoped skills store used by pre-release builds:
/// `~/.ship/projects/<project-slug>/skills/`
pub fn legacy_project_skills_dir(ship_dir: &Path) -> PathBuf {
    legacy_project_skills_dir_candidates(ship_dir)
        .into_iter()
        .next()
        .unwrap_or_else(|| {
            PathBuf::from(".ship")
                .join("projects")
                .join("unknown-project")
                .join("skills")
        })
}

/// Candidate legacy project skill roots in preferred order.
/// First candidate is the new sync-safe namespace key.
/// Second candidate (if different) preserves migration from historic path-derived slugs.
pub fn legacy_project_skills_dir_candidates(ship_dir: &Path) -> Vec<PathBuf> {
    let global_dir = get_global_dir().unwrap_or_else(|_| PathBuf::from(".ship"));
    let preferred = global_dir
        .join("projects")
        .join(project_slug_from_ship_dir(ship_dir))
        .join("skills");
    let legacy = global_dir
        .join("projects")
        .join(legacy_path_slug_from_ship_dir(ship_dir))
        .join("skills");

    if preferred == legacy {
        vec![preferred]
    } else {
        vec![preferred, legacy]
    }
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

    if let Some(test_dir) = auto_test_global_dir() {
        fs::create_dir_all(&test_dir).with_context(|| {
            format!(
                "Failed to create auto-isolated test global dir at {}",
                test_dir.display()
            )
        })?;
        return Ok(test_dir);
    }

    home::home_dir()
        .map(|h| h.join(SHIP_DIR_NAME))
        .ok_or_else(|| anyhow!("Could not find home directory"))
}

fn auto_test_global_dir() -> Option<PathBuf> {
    if std::env::var_os("SHIP_DISABLE_AUTO_TEST_GLOBAL_DIR").is_some() {
        return None;
    }

    let exe = std::env::current_exe().ok()?;
    if !is_likely_rust_test_binary(&exe) {
        return None;
    }

    thread_local! {
        static TEST_GLOBAL_DIR: std::cell::RefCell<Option<PathBuf>> = const { std::cell::RefCell::new(None) };
    }
    let dir = TEST_GLOBAL_DIR.with(|cell| {
        let mut slot = cell.borrow_mut();
        slot.get_or_insert_with(|| {
            let thread_suffix = format!("{:?}", std::thread::current().id())
                .chars()
                .filter(|c| c.is_ascii_alphanumeric())
                .collect::<String>();
            std::env::temp_dir()
                .join(format!(
                    "{}{}-{}",
                    TEST_GLOBAL_DIR_PREFIX,
                    std::process::id(),
                    thread_suffix
                ))
                .join(SHIP_DIR_NAME)
        })
        .clone()
    });
    register_test_global_cleanup(&dir);
    cleanup_stale_test_global_dirs();
    Some(dir)
}

fn is_likely_rust_test_binary(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    let in_test_deps = path_str.contains("/target/debug/deps/")
        || path_str.contains("/target/release/deps/")
        || path_str.contains("\\target\\debug\\deps\\")
        || path_str.contains("\\target\\release\\deps\\");
    if !in_test_deps {
        return false;
    }
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.contains('-'))
}

#[cfg(unix)]
extern "C" fn cleanup_test_global_dir_on_exit() {
    if let Some(path) = TEST_GLOBAL_CLEANUP_PATH.get() {
        if let Some(run_root) = path.parent() {
            let _ = fs::remove_dir_all(run_root);
        }
    }
}

fn register_test_global_cleanup(path: &Path) {
    let _ = TEST_GLOBAL_CLEANUP_PATH.set(path.to_path_buf());
    #[cfg(unix)]
    if TEST_GLOBAL_CLEANUP_REGISTERED.get().is_none() {
        // SAFETY: registering a process-exit callback once is safe here; callback
        // does best-effort cleanup and ignores failures.
        let _ = unsafe { atexit(cleanup_test_global_dir_on_exit) };
        let _ = TEST_GLOBAL_CLEANUP_REGISTERED.set(());
    }
}

fn cleanup_stale_test_global_dirs() {
    let Ok(entries) = fs::read_dir(std::env::temp_dir()) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some(pid) = parse_test_global_dir_pid(name) else {
            continue;
        };
        if pid == std::process::id() {
            continue;
        }
        if is_process_alive(pid) {
            continue;
        }
        if path.is_dir() {
            let _ = fs::remove_dir_all(path);
        }
    }
}

fn parse_test_global_dir_pid(name: &str) -> Option<u32> {
    name.strip_prefix(TEST_GLOBAL_DIR_PREFIX)?
        .split('-')
        .next()?
        .parse()
        .ok()
}

#[cfg(unix)]
fn is_process_alive(pid: u32) -> bool {
    // SAFETY: kill(pid, 0) is a read-only existence/permission probe.
    let rc = unsafe { kill(pid as i32, 0) };
    if rc == 0 {
        return true;
    }
    // ESRCH (3) => process does not exist.
    std::io::Error::last_os_error().raw_os_error() != Some(3)
}

#[cfg(not(unix))]
fn is_process_alive(_pid: u32) -> bool {
    false
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
    let mut state: AppState =
        serde_json::from_str(&content).context("Failed to parse app state")?;
    if normalize_app_state_paths(&mut state) {
        let _ = save_app_state(&state);
    }
    Ok(state)
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

fn normalize_app_state_paths(state: &mut AppState) -> bool {
    let mut changed = false;

    if let Some(active) = state.active_project.clone() {
        let normalized = normalize_registry_project_path(&active);
        if normalized != active {
            state.active_project = Some(normalized);
            changed = true;
        }
    }

    let mut normalized_recent = Vec::with_capacity(state.recent_projects.len());
    for path in &state.recent_projects {
        let normalized = normalize_registry_project_path(path);
        if normalized != *path {
            changed = true;
        }
        if !normalized_recent.contains(&normalized) {
            normalized_recent.push(normalized);
        } else {
            changed = true;
        }
    }

    if normalized_recent != state.recent_projects {
        state.recent_projects = normalized_recent;
        changed = true;
    }

    changed
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

    #[test]
    fn detects_rust_test_binaries_from_target_deps_path() {
        assert!(is_likely_rust_test_binary(Path::new(
            "/tmp/repo/target/debug/deps/runtime-abc123"
        )));
        assert!(is_likely_rust_test_binary(Path::new(
            "C:\\repo\\target\\release\\deps\\runtime-abc123.exe"
        )));
        assert!(!is_likely_rust_test_binary(Path::new(
            "/tmp/repo/target/debug/ship"
        )));
        assert!(!is_likely_rust_test_binary(Path::new(
            "/tmp/repo/target/debug/deps/ship"
        )));
    }

    #[test]
    fn parses_test_global_dir_pid() -> Result<()> {
        assert_eq!(parse_test_global_dir_pid("ship-test-global-123"), Some(123));
        assert_eq!(
            parse_test_global_dir_pid("ship-test-global-123-ThreadId9"),
            Some(123)
        );
        assert_eq!(parse_test_global_dir_pid("ship-test-global-abc"), None);
        assert_eq!(parse_test_global_dir_pid("other-prefix-123"), None);
        Ok(())
    }
    #[test]
    fn project_slug_uses_name_and_id_tokens() -> Result<()> {
        let tmp = tempdir()?;
        let ship = tmp.path().join(".ship");
        fs::create_dir_all(&ship)?;
        fs::write(
            ship.join(crate::config::PRIMARY_CONFIG_FILE),
            "id = 'AbC123xy'\nname = 'Platform API'\n",
        )?;

        let slug = project_slug_from_ship_dir(&ship);
        assert!(slug.ends_with("platform-api-abc123xy"));
        Ok(())
    }

    #[test]
    fn transient_path_detection_catches_temp_and_e2e_layouts() {
        assert!(is_transient_project_root(Path::new("/tmp/demo")));
        assert!(is_transient_project_root(Path::new("/private/tmp/demo")));
        assert!(is_transient_project_root(Path::new(
            "/repo/target/tmp/ship-e2e"
        )));
        assert!(is_transient_project_root(Path::new(
            "C:/Users/me/AppData/Local/Temp/ship-e2e"
        )));
        assert!(is_transient_project_root(Path::new(
            "/work/examples/e2e/sandbox"
        )));
        assert!(!is_transient_project_root(Path::new(
            "/Users/me/dev/real-project"
        )));
    }

    #[test]
    fn ensure_project_id_populates_missing_id() -> Result<()> {
        let tmp = tempdir()?;
        let ship = tmp.path().join(".ship");
        fs::create_dir_all(&ship)?;
        fs::write(
            ship.join(crate::config::PRIMARY_CONFIG_FILE),
            "version = '1'\nname = 'legacy-project'\n",
        )?;

        let id = ensure_project_id(&ship)?;
        assert!(!id.trim().is_empty());

        let raw = fs::read_to_string(ship.join(crate::config::PRIMARY_CONFIG_FILE))?;
        let parsed: toml::Value = toml::from_str(&raw)?;
        let persisted_id = parsed
            .get("id")
            .and_then(|value| value.as_str())
            .unwrap_or_default();
        assert_eq!(persisted_id, id);
        Ok(())
    }

    #[test]
    fn normalize_app_state_paths_canonicalizes_and_dedupes_entries() -> Result<()> {
        let tmp = tempdir()?;
        let root = tmp.path().join("workspace");
        let ship = root.join(SHIP_DIR_NAME);
        fs::create_dir_all(&ship)?;

        let mut state = AppState {
            active_project: Some(root.clone()),
            recent_projects: vec![root.clone(), ship.clone()],
        };

        assert!(normalize_app_state_paths(&mut state));
        let canonical_ship = canonicalize_lossy(&ship);
        assert_eq!(state.active_project, Some(canonical_ship.clone()));
        assert_eq!(state.recent_projects, vec![canonical_ship]);
        Ok(())
    }

    #[test]
    fn normalize_app_state_paths_reports_no_change_for_canonical_state() -> Result<()> {
        let tmp = tempdir()?;
        let ship = tmp.path().join("workspace").join(SHIP_DIR_NAME);
        fs::create_dir_all(&ship)?;
        let canonical_ship = canonicalize_lossy(&ship);

        let mut state = AppState {
            active_project: Some(canonical_ship.clone()),
            recent_projects: vec![canonical_ship],
        };

        assert!(!normalize_app_state_paths(&mut state));
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
        "project/specs",
        "project/features",
        "project/releases",
        "project/notes",
        "agents/skills",
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
        &vision_doc_path(&ship_path),
        "# Vision\n\nDescribe what this project is trying to achieve.\n",
    )?;

    // Write ship.toml (with a stable project ID) BEFORE any DB access so that
    // project_db_key can read the ID and derive a stable state directory path.
    if !ship_path.join(crate::config::PRIMARY_CONFIG_FILE).exists() {
        let mut config = crate::config::ProjectConfig::default();
        config.id = crate::gen_nanoid();
        crate::config::save_config(&config, Some(ship_path.clone()))?;
    }

    crate::events::ensure_event_log(&ship_path)?;
    write_if_missing(
        &skills_dir(&ship_path).join("task-policy").join("SKILL.md"),
        r#"---
name: task-policy
description: Ship workflow policy and execution guardrails for daily delivery.
metadata:
  display_name: Ship Workflow Policy
  source: builtin
---

# Ship Workflow Policy

Use Ship as the system of record for workflow state changes.

## Canonical Flow

Vision -> Release -> Feature -> Spec -> Close Feature -> Ship Release
"#,
    )?;

    let config = crate::config::get_config(Some(ship_path.clone()))?;
    crate::config::ensure_registered_namespaces(&ship_path, &config.namespaces)?;
    crate::config::generate_gitignore(&ship_path, &config.git)?;

    Ok(ship_path)
}
