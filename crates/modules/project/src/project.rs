use anyhow::{Context, Result, anyhow};
use runtime::config::{
    McpServerConfig, McpServerType, ModeConfig, NamespaceConfig, ProjectConfig,
    add_mode as runtime_add_mode,
    ensure_registered_namespaces as runtime_ensure_registered_namespaces, get_config,
    remove_mode as runtime_remove_mode, save_config, set_active_mode as runtime_set_active_mode,
};
use runtime::fs_util::write_atomic;
pub use runtime::project::{ProjectEntry, ProjectRegistry};
use runtime::project::{
    SHIP_DIR_NAME, adrs_dir as runtime_adrs_dir, agents_ns as runtime_agents_ns,
    features_dir as runtime_features_dir, generated_ns as runtime_generated_ns, get_global_dir,
    get_project_dir as runtime_get_project_dir, get_project_name as runtime_get_project_name,
    mcp_config_path as runtime_mcp_config_path, modes_dir as runtime_modes_dir,
    notes_dir as runtime_notes_dir, permissions_config_path as runtime_permissions_config_path,
    project_ns as runtime_project_ns, register_ship_namespace as runtime_register_ship_namespace,
    releases_dir as runtime_releases_dir,
    resolve_project_ship_dir as runtime_resolve_project_ship_dir, rules_dir as runtime_rules_dir,
    sanitize_file_name as runtime_sanitize_file_name,
    ship_dir_from_path as runtime_ship_dir_from_path, skills_dir as runtime_skills_dir,
    upcoming_releases_dir as runtime_upcoming_releases_dir,
    vision_doc_path as runtime_vision_doc_path,
    vision_template_path as runtime_vision_template_path,
};
use runtime::{EventAction, EventEntity, append_event};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub const DEFAULT_STATUSES: &[&str] = &["backlog", "in-progress", "blocked", "done"];

// ── Namespace path helpers ────────────────────────────────────────────────────

pub fn project_ns(ship_dir: &Path) -> PathBuf {
    runtime_project_ns(ship_dir)
}

pub fn agents_ns(ship_dir: &Path) -> PathBuf {
    runtime_agents_ns(ship_dir)
}

pub fn generated_ns(ship_dir: &Path) -> PathBuf {
    runtime_generated_ns(ship_dir)
}

pub fn adrs_dir(ship_dir: &Path) -> PathBuf {
    runtime_adrs_dir(ship_dir)
}

pub fn releases_dir(ship_dir: &Path) -> PathBuf {
    runtime_releases_dir(ship_dir)
}

pub fn upcoming_releases_dir(ship_dir: &Path) -> PathBuf {
    runtime_upcoming_releases_dir(ship_dir)
}

pub fn notes_dir(ship_dir: &Path) -> PathBuf {
    runtime_notes_dir(ship_dir)
}

pub fn features_dir(ship_dir: &Path) -> PathBuf {
    runtime_features_dir(ship_dir)
}

pub fn modes_dir(ship_dir: &Path) -> PathBuf {
    runtime_modes_dir(ship_dir)
}

pub fn skills_dir(ship_dir: &Path) -> PathBuf {
    runtime_skills_dir(ship_dir)
}

pub fn rules_dir(ship_dir: &Path) -> PathBuf {
    runtime_rules_dir(ship_dir)
}

pub fn mcp_config_path(ship_dir: &Path) -> PathBuf {
    runtime_mcp_config_path(ship_dir)
}

pub fn permissions_config_path(ship_dir: &Path) -> PathBuf {
    runtime_permissions_config_path(ship_dir)
}

pub fn vision_doc_path(ship_dir: &Path) -> PathBuf {
    runtime_vision_doc_path(ship_dir)
}

pub fn vision_template_path(ship_dir: &Path) -> PathBuf {
    runtime_vision_template_path(ship_dir)
}

pub fn ship_dir_from_path(path: &Path) -> Option<PathBuf> {
    runtime_ship_dir_from_path(path)
}

pub fn get_project_dir(start_dir: Option<PathBuf>) -> Result<PathBuf> {
    runtime_get_project_dir(start_dir)
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
    let (registry, changed) = normalize_registry(
        registry,
        should_filter_transient_registry_entries(&get_registry_path()?),
    );
    if changed {
        save_registry(&registry)?;
    }
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

fn normalize_registry_project_path(path: &Path) -> PathBuf {
    if let Some(ship_path) = runtime_resolve_project_ship_dir(path) {
        return ship_path;
    }

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

fn normalize_registry_project_name(name: &str, default_name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        default_name.to_string()
    } else {
        trimmed.to_string()
    }
}

fn should_prefer_candidate_name(
    existing_name: &str,
    candidate_name: &str,
    default_name: &str,
) -> bool {
    let existing_is_default = existing_name.trim() == default_name;
    let candidate_is_default = candidate_name.trim() == default_name;
    existing_is_default && !candidate_is_default
}

fn path_starts_with_components(path: &Path, parts: &[&str]) -> bool {
    let mut normal_components = path.components().filter_map(|component| match component {
        Component::Normal(value) => value.to_str(),
        _ => None,
    });

    for expected in parts {
        if normal_components.next() != Some(*expected) {
            return false;
        }
    }
    true
}

fn path_contains_component_sequence(path: &Path, parts: &[&str]) -> bool {
    let normal_components: Vec<&str> = path
        .components()
        .filter_map(|component| match component {
            Component::Normal(value) => value.to_str(),
            _ => None,
        })
        .collect();
    if parts.is_empty() || normal_components.len() < parts.len() {
        return false;
    }
    normal_components
        .windows(parts.len())
        .any(|window| window == parts)
}

fn should_filter_transient_registry_path(path: &Path) -> bool {
    if let Some(global_dir) =
        default_registry_path().and_then(|registry| registry.parent().map(Path::to_path_buf))
    {
        let canonical_path = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        let canonical_global_dir = fs::canonicalize(&global_dir).unwrap_or(global_dir);
        if canonical_path == canonical_global_dir {
            return true;
        }
    }

    if path_starts_with_components(path, &["tmp"])
        || path_starts_with_components(path, &["private", "tmp"])
        || path_starts_with_components(path, &["var", "folders"])
        || path_starts_with_components(path, &["private", "var", "folders"])
    {
        return true;
    }

    path_contains_component_sequence(path, &["examples", "e2e"])
        || path_contains_component_sequence(path, &["examples", "projects-e2e"])
        || path_contains_component_sequence(path, &["target", "tmp"])
}

fn default_registry_path() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| {
        PathBuf::from(home)
            .join(SHIP_DIR_NAME)
            .join("projects.json")
    })
}

fn should_filter_transient_registry_entries(registry_path: &Path) -> bool {
    let Some(default_path) = default_registry_path() else {
        return false;
    };
    let canonical_registry_path =
        fs::canonicalize(registry_path).unwrap_or_else(|_| registry_path.to_path_buf());
    let canonical_default_path = fs::canonicalize(&default_path).unwrap_or(default_path);
    canonical_registry_path == canonical_default_path
}

fn normalize_registry(
    registry: ProjectRegistry,
    filter_transient_paths: bool,
) -> (ProjectRegistry, bool) {
    let mut changed = false;
    let mut deduped: Vec<ProjectEntry> = Vec::new();
    let mut index_by_path: HashMap<PathBuf, usize> = HashMap::new();

    for project in registry.projects {
        let normalized_path = normalize_registry_project_path(&project.path);
        if filter_transient_paths && should_filter_transient_registry_path(&normalized_path) {
            changed = true;
            continue;
        }

        let default_name = runtime_get_project_name(&normalized_path);
        let normalized_name = normalize_registry_project_name(&project.name, &default_name);

        if normalized_path != project.path || normalized_name != project.name {
            changed = true;
        }

        if let Some(existing_index) = index_by_path.get(&normalized_path).copied() {
            changed = true;
            let existing = &mut deduped[existing_index];
            if should_prefer_candidate_name(&existing.name, &normalized_name, &default_name)
                && existing.name != normalized_name
            {
                existing.name = normalized_name;
            }
            continue;
        }

        let insert_index = deduped.len();
        deduped.push(ProjectEntry {
            name: normalized_name,
            path: normalized_path.clone(),
        });
        index_by_path.insert(normalized_path, insert_index);
    }

    (ProjectRegistry { projects: deduped }, changed)
}

fn should_keep_existing_name(existing_name: &str, incoming_name: &str, default_name: &str) -> bool {
    let existing_is_custom = existing_name.trim() != default_name;
    let incoming_is_default = incoming_name == default_name;
    existing_is_custom && incoming_is_default
}

pub fn register_project(name: String, path: PathBuf) -> Result<()> {
    let mut registry = load_registry()?;
    let canonical_path = normalize_registry_project_path(&path);
    if should_filter_transient_registry_entries(&get_registry_path()?)
        && should_filter_transient_registry_path(&canonical_path)
    {
        return Ok(());
    }

    let default_name = runtime_get_project_name(&canonical_path);
    let incoming_name = {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            default_name.clone()
        } else {
            trimmed.to_string()
        }
    };

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
        if !should_keep_existing_name(&existing.name, &incoming_name, &default_name) {
            existing.name = incoming_name;
        }
        existing.path = canonical_path;
    } else {
        registry.projects.push(ProjectEntry {
            name: incoming_name,
            path: canonical_path,
        });
    }

    save_registry(&registry)?;
    Ok(())
}

pub fn rename_project(path: PathBuf, name: String) -> Result<()> {
    let normalized = normalize_registry_project_path(&path);
    if should_filter_transient_registry_entries(&get_registry_path()?)
        && should_filter_transient_registry_path(&normalized)
    {
        return Err(anyhow!(
            "Refusing to rename transient project path: {}",
            normalized.display()
        ));
    }
    let renamed = name.trim();
    if renamed.is_empty() {
        return Err(anyhow!("Project name cannot be empty"));
    }

    let mut registry = load_registry()?;
    if let Some(existing) = registry
        .projects
        .iter_mut()
        .find(|project| normalize_registry_project_path(&project.path) == normalized)
    {
        existing.name = renamed.to_string();
        existing.path = normalized;
        save_registry(&registry)?;
        return Ok(());
    }

    registry.projects.push(ProjectEntry {
        name: renamed.to_string(),
        path: normalized,
    });
    save_registry(&registry)?;
    Ok(())
}

pub fn unregister_project(path: PathBuf) -> Result<()> {
    let mut registry = load_registry()?;
    let path = normalize_registry_project_path(&path);
    if should_filter_transient_registry_entries(&get_registry_path()?)
        && should_filter_transient_registry_path(&path)
    {
        return Ok(());
    }
    registry
        .projects
        .retain(|p| normalize_registry_project_path(&p.path) != path);
    save_registry(&registry)?;
    Ok(())
}

pub fn list_registered_projects() -> Result<Vec<ProjectEntry>> {
    let registry = load_registry()?;
    Ok(registry.projects)
}

pub fn init_project(base_dir: PathBuf) -> Result<PathBuf> {
    let ship_path = base_dir.join(SHIP_DIR_NAME);
    fs::create_dir_all(&ship_path)?;

    let config_exists = [
        ship_path.join(runtime::config::PRIMARY_CONFIG_FILE),
        ship_path.join(runtime::config::LEGACY_CONFIG_FILE),
    ]
    .iter()
    .any(|path| path.exists());

    let mut config = if config_exists {
        get_config(Some(ship_path.clone()))?
    } else {
        ProjectConfig::default()
    };
    if config.id.trim().is_empty() {
        config.id = runtime::gen_nanoid();
    }
    ensure_first_party_namespaces(&mut config.namespaces);
    if config_exists {
        save_config(&config, Some(ship_path.clone()))?;
        cleanup_legacy_config_files(&ship_path)?;
    } else {
        write_initial_config_with_comments(&ship_path, &config)?;
    }
    ensure_registered_namespaces(&ship_path, &config.namespaces)?;

    fs::create_dir_all(skills_dir(&ship_path))?;
    fs::create_dir_all(rules_dir(&ship_path))?;

    runtime::events::ensure_event_log(&ship_path)?;

    // Seed only the canonical project vision doc at init. Other planning artifacts
    // are DB-first and can be exported on demand.
    write_if_missing(
        &vision_doc_path(&ship_path),
        include_str!("../../../../core/runtime/src/templates/VISION.md"),
    )?;
    write_default_skills(&ship_path)?;
    write_if_missing(
        &mcp_config_path(&ship_path),
        include_str!("../../../../core/runtime/src/templates/MCP.toml"),
    )?;
    ensure_ship_mcp_server(&ship_path)?;
    write_if_missing(
        &permissions_config_path(&ship_path),
        include_str!("../../../../core/runtime/src/templates/PERMISSIONS.toml"),
    )?;
    let principles_path = rules_dir(&ship_path).join("core-principles.md");
    write_if_missing(
        &principles_path,
        include_str!("../../../../core/runtime/src/templates/RULE.md"),
    )?;
    let gitignore_path = ship_path.join(".gitignore");
    if !gitignore_path.exists() {
        let default_git = runtime::config::GitConfig::default();
        runtime::config::generate_gitignore(&ship_path, &default_git)?;
    }

    // Seed the project-manager workspace ("ship") so it's ready on first use.
    if let Err(e) = runtime::workspace::seed_service_workspace(&ship_path) {
        eprintln!("[ship] warning: could not seed project workspace: {}", e);
    }

    let _ = append_event(
        &ship_path,
        "logic",
        EventEntity::Project,
        EventAction::Init,
        "project",
        Some("Project initialized".to_string()),
    );

    Ok(ship_path)
}

fn write_initial_config_with_comments(ship_path: &Path, config: &ProjectConfig) -> Result<()> {
    let config_path = ship_path.join(runtime::config::PRIMARY_CONFIG_FILE);
    let mut content = String::from(
        "# Ship project configuration\n\
         # - Edit with care; prefer `ship config` and `ship git` commands where possible.\n\
         # - `namespaces` controls top-level directories under `.ship/`.\n\
         # - Plugin namespaces are dynamically registered when plugins are used.\n\n",
    );
    content.push_str(&toml::to_string_pretty(config)?);
    write_atomic(&config_path, content)?;
    Ok(())
}

fn write_if_missing(path: &Path, content: &str) -> Result<bool> {
    if path.exists() {
        return Ok(false);
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(true)
}

fn cleanup_legacy_config_files(ship_path: &Path) -> Result<()> {
    let primary = ship_path.join(runtime::config::PRIMARY_CONFIG_FILE);
    if !primary.exists() {
        return Ok(());
    }

    for legacy_name in [runtime::config::LEGACY_CONFIG_FILE] {
        let legacy = ship_path.join(legacy_name);
        if legacy.exists() {
            fs::remove_file(legacy)?;
        }
    }

    Ok(())
}

fn write_default_skills(ship_path: &Path) -> Result<()> {
    let project_skills_root = skills_dir(ship_path);
    fs::create_dir_all(&project_skills_root)?;
    migrate_legacy_project_skills_dir(ship_path, &project_skills_root)?;

    write_if_missing(
        &project_skills_root.join("task-policy").join("SKILL.md"),
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

    const PROJECT_BUILTINS: &[(&str, &str)] = &[
        (
            "ship-workflow",
            include_str!("../../../../core/runtime/src/templates/skills/ship-workflow.SKILL.md"),
        ),
        (
            "start-session",
            include_str!("../../../../core/runtime/src/templates/skills/start-session/SKILL.md"),
        ),
        (
            "create-document",
            include_str!("../../../../core/runtime/src/templates/skills/create-document/SKILL.md"),
        ),
        (
            "workspace-session-lifecycle",
            include_str!(
                "../../../../core/runtime/src/templates/skills/workspace-session-lifecycle/SKILL.md"
            ),
        ),
    ];
    for (id, content) in PROJECT_BUILTINS {
        write_if_missing(&project_skills_root.join(id).join("SKILL.md"), content)?;
    }

    seed_builtin_user_skills(&runtime::project::user_skills_dir())?;
    Ok(())
}

fn migrate_legacy_project_skills_dir(ship_path: &Path, target_root: &Path) -> Result<()> {
    let legacy_root = ship_path.join("skills");
    if legacy_root == target_root || !legacy_root.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(&legacy_root)? {
        let entry = entry?;
        let source_path = entry.path();
        if !source_path.is_dir() || !source_path.join("SKILL.md").is_file() {
            continue;
        }

        let destination_path = target_root.join(entry.file_name());
        if destination_path.exists() {
            continue;
        }

        match fs::rename(&source_path, &destination_path) {
            Ok(_) => {}
            Err(_) => {
                copy_dir_recursive(&source_path, &destination_path)?;
                fs::remove_dir_all(&source_path)?;
            }
        }
    }

    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = dst.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir_recursive(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path)?;
        }
    }
    Ok(())
}

fn seed_builtin_user_skills(user_skills_root: &Path) -> Result<()> {
    fs::create_dir_all(user_skills_root)?;

    // Builtins always overwrite — they're owned by Ship and update with each release.
    const SINGLE_FILE_BUILTINS: &[(&str, &str)] = &[
        (
            "ship-workflow",
            include_str!("../../../../core/runtime/src/templates/skills/ship-workflow.SKILL.md"),
        ),
        (
            "create-document",
            include_str!("../../../../core/runtime/src/templates/skills/create-document/SKILL.md"),
        ),
        (
            "workspace-session-lifecycle",
            include_str!(
                "../../../../core/runtime/src/templates/skills/workspace-session-lifecycle/SKILL.md"
            ),
        ),
        (
            "release-orchestration",
            include_str!(
                "../../../../core/runtime/src/templates/skills/release-orchestration/SKILL.md"
            ),
        ),
        (
            "workspace-profile-onboarding",
            include_str!(
                "../../../../core/runtime/src/templates/skills/workspace-profile-onboarding/SKILL.md"
            ),
        ),
        (
            "start-session",
            include_str!("../../../../core/runtime/src/templates/skills/start-session/SKILL.md"),
        ),
    ];
    for (id, content) in SINGLE_FILE_BUILTINS {
        let skill_dir = user_skills_root.join(id);
        fs::create_dir_all(&skill_dir)?;
        fs::write(skill_dir.join("SKILL.md"), content)?;
    }

    // skill-creator is a multi-file skill; only seed if missing.
    let skill_creator_root = user_skills_root.join("skill-creator");
    if !skill_creator_root.exists() {
        seed_skill_creator_template(&skill_creator_root)?;
    }

    Ok(())
}

fn seed_skill_creator_template(skill_root: &Path) -> Result<()> {
    const FILES: &[(&str, &str)] = &[
        (
            "SKILL.md",
            include_str!("../../../../core/runtime/src/templates/skills/skill-creator/SKILL.md"),
        ),
        (
            "LICENSE.txt",
            include_str!("../../../../core/runtime/src/templates/skills/skill-creator/LICENSE.txt"),
        ),
        (
            "agents/analyzer.md",
            include_str!(
                "../../../../core/runtime/src/templates/skills/skill-creator/agents/analyzer.md"
            ),
        ),
        (
            "agents/comparator.md",
            include_str!(
                "../../../../core/runtime/src/templates/skills/skill-creator/agents/comparator.md"
            ),
        ),
        (
            "agents/grader.md",
            include_str!(
                "../../../../core/runtime/src/templates/skills/skill-creator/agents/grader.md"
            ),
        ),
        (
            "assets/eval_review.html",
            include_str!(
                "../../../../core/runtime/src/templates/skills/skill-creator/assets/eval_review.html"
            ),
        ),
        (
            "eval-viewer/generate_review.py",
            include_str!(
                "../../../../core/runtime/src/templates/skills/skill-creator/eval-viewer/generate_review.py"
            ),
        ),
        (
            "eval-viewer/viewer.html",
            include_str!(
                "../../../../core/runtime/src/templates/skills/skill-creator/eval-viewer/viewer.html"
            ),
        ),
        (
            "references/schemas.md",
            include_str!(
                "../../../../core/runtime/src/templates/skills/skill-creator/references/schemas.md"
            ),
        ),
        (
            "scripts/__init__.py",
            include_str!(
                "../../../../core/runtime/src/templates/skills/skill-creator/scripts/__init__.py"
            ),
        ),
        (
            "scripts/aggregate_benchmark.py",
            include_str!(
                "../../../../core/runtime/src/templates/skills/skill-creator/scripts/aggregate_benchmark.py"
            ),
        ),
        (
            "scripts/generate_report.py",
            include_str!(
                "../../../../core/runtime/src/templates/skills/skill-creator/scripts/generate_report.py"
            ),
        ),
        (
            "scripts/improve_description.py",
            include_str!(
                "../../../../core/runtime/src/templates/skills/skill-creator/scripts/improve_description.py"
            ),
        ),
        (
            "scripts/package_skill.py",
            include_str!(
                "../../../../core/runtime/src/templates/skills/skill-creator/scripts/package_skill.py"
            ),
        ),
        (
            "scripts/quick_validate.py",
            include_str!(
                "../../../../core/runtime/src/templates/skills/skill-creator/scripts/quick_validate.py"
            ),
        ),
        (
            "scripts/run_eval.py",
            include_str!(
                "../../../../core/runtime/src/templates/skills/skill-creator/scripts/run_eval.py"
            ),
        ),
        (
            "scripts/run_loop.py",
            include_str!(
                "../../../../core/runtime/src/templates/skills/skill-creator/scripts/run_loop.py"
            ),
        ),
        (
            "scripts/utils.py",
            include_str!(
                "../../../../core/runtime/src/templates/skills/skill-creator/scripts/utils.py"
            ),
        ),
    ];

    for (rel, content) in FILES {
        write_if_missing(&skill_root.join(rel), content)?;
    }

    Ok(())
}

fn ensure_first_party_namespaces(namespaces: &mut Vec<NamespaceConfig>) {
    namespaces.retain(|ns| {
        !(ns.id == "project" && ns.path == "project")
            && !(ns.id == "project" && ns.path == "project/")
            && !(ns.id == "plugins" && ns.path == "plugins")
            && !(ns.id == "workflow" && ns.path == "workflow")
    });

    let required = [
        ("agents", "agents", "agents"),
        ("generated", "generated", "runtime"),
    ];
    for (id, path, owner) in required {
        let exists = namespaces.iter().any(|ns| ns.id == id);
        if !exists {
            namespaces.push(NamespaceConfig {
                id: id.to_string(),
                path: path.to_string(),
                owner: owner.to_string(),
            });
        }
    }
}

fn ship_runtime_mcp_server() -> McpServerConfig {
    McpServerConfig {
        id: "ship".to_string(),
        name: "Ship Runtime".to_string(),
        command: "ship".to_string(),
        args: vec!["mcp".to_string(), "serve".to_string()],
        env: HashMap::new(),
        scope: "project".to_string(),
        server_type: McpServerType::Stdio,
        url: None,
        disabled: false,
        timeout_secs: None,
    }
}

fn ensure_ship_mcp_server(ship_path: &Path) -> Result<()> {
    let mut config = get_config(Some(ship_path.to_path_buf()))?;
    let has_ship_server = config.mcp_servers.iter().any(|server| server.id == "ship");
    if has_ship_server {
        return Ok(());
    }
    config.mcp_servers.push(ship_runtime_mcp_server());
    save_config(&config, Some(ship_path.to_path_buf()))
}

fn ensure_registered_namespaces(ship_path: &Path, namespaces: &[NamespaceConfig]) -> Result<()> {
    runtime_ensure_registered_namespaces(ship_path, namespaces)
}

fn template_rel_path(kind: &str) -> Result<&'static str> {
    match kind {
        "adr" | "adrs" => Ok("project/adrs/TEMPLATE.md"),
        "note" | "notes" => Ok("project/notes/TEMPLATE.md"),
        "spec" | "specs" => Ok("project/specs/TEMPLATE.md"),
        "release" | "releases" => Ok("project/releases/TEMPLATE.md"),
        "feature" | "features" => Ok("project/features/TEMPLATE.md"),
        "vision" => Ok("TEMPLATE.md"),
        _ => Err(anyhow!("Unknown template kind: {}", kind)),
    }
}

fn legacy_template_file_name(kind: &str) -> Option<&'static str> {
    match kind {
        "adr" | "adrs" => Some("ADR.md"),
        "note" | "notes" => Some("NOTE.md"),
        "spec" | "specs" => Some("SPEC.md"),
        "release" | "releases" => Some("RELEASE.md"),
        "feature" | "features" => Some("FEATURE.md"),
        "vision" => Some("VISION.md"),
        _ => None,
    }
}

fn template_fallback(kind: &str) -> Result<&'static str> {
    match kind {
        "adr" | "adrs" => Ok(include_str!(
            "../../../../core/runtime/src/templates/ADR.md"
        )),
        "note" | "notes" => Ok(include_str!(
            "../../../../core/runtime/src/templates/NOTE.md"
        )),
        "spec" | "specs" => Ok(include_str!(
            "../../../../core/runtime/src/templates/SPEC.md"
        )),
        "release" | "releases" => Ok(include_str!(
            "../../../../core/runtime/src/templates/RELEASE.md"
        )),
        "feature" | "features" => Ok(include_str!(
            "../../../../core/runtime/src/templates/FEATURE.md"
        )),
        "vision" => Ok(include_str!(
            "../../../../core/runtime/src/templates/VISION.md"
        )),
        _ => Err(anyhow!("No fallback for template kind: {}", kind)),
    }
}

pub fn read_template(ship_path: &Path, kind: &str) -> Result<String> {
    let normalized = kind.trim().to_ascii_lowercase();
    let template_path = ship_path.join(template_rel_path(&normalized)?);
    if template_path.exists() {
        return fs::read_to_string(&template_path)
            .with_context(|| format!("Failed to read template: {}", template_path.display()));
    }
    if normalized == "vision" {
        let legacy_project_template = project_ns(ship_path).join("TEMPLATE.md");
        if legacy_project_template.exists() {
            return fs::read_to_string(&legacy_project_template).with_context(|| {
                format!(
                    "Failed to read template: {}",
                    legacy_project_template.display()
                )
            });
        }
    }

    if let Some(file_name) = legacy_template_file_name(&normalized) {
        let legacy_path = ship_path.join("templates").join(file_name);
        if legacy_path.exists() {
            return fs::read_to_string(&legacy_path)
                .with_context(|| format!("Failed to read template: {}", legacy_path.display()));
        }
    }

    Ok(template_fallback(&normalized)?.to_string())
}

pub fn write_template(ship_path: &Path, kind: &str, content: &str) -> Result<()> {
    let normalized = kind.trim().to_ascii_lowercase();
    let template_path = ship_path.join(template_rel_path(&normalized)?);
    if let Some(parent) = template_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create template dir: {}", parent.display()))?;
    }
    fs::write(&template_path, content)
        .with_context(|| format!("Failed to write template: {}", template_path.display()))?;
    Ok(())
}

pub fn validate_status(status: &str) -> Result<()> {
    if status.trim().is_empty() {
        return Err(anyhow!("Status cannot be empty"));
    }
    if status.contains('/') || status.contains('\\') || status.contains("..") {
        return Err(anyhow!(
            "Invalid status '{}': must not contain path separators",
            status
        ));
    }
    Ok(())
}

pub fn list_registered_namespaces(ship_path: &Path) -> Result<Vec<NamespaceConfig>> {
    let config = get_config(Some(ship_path.to_path_buf()))?;
    Ok(config.namespaces)
}

pub fn register_ship_namespace(ship_path: &Path, namespace: NamespaceConfig) -> Result<()> {
    runtime_register_ship_namespace(ship_path, namespace)
}

pub fn sanitize_file_name(name: &str) -> String {
    runtime_sanitize_file_name(name)
}

pub fn get_project_name(ship_path: &Path) -> String {
    runtime_get_project_name(ship_path)
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct ProjectDiscovery {
    pub name: String,
    #[specta(type = String)]
    pub path: PathBuf,
}

pub fn discover_projects(root: PathBuf) -> Result<Vec<ProjectDiscovery>> {
    let mut projects = Vec::new();
    if !root.is_dir() {
        return Ok(projects);
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if name.starts_with('.') && name != ".ship" {
                continue;
            }
            if matches!(
                name.as_ref(),
                "Trash"
                    | ".Trash"
                    | ".DS_Store"
                    | "._*"
                    | "TemporaryItems"
                    | ".Spotlight-V100"
                    | ".fseventsd"
            ) {
                continue;
            }
            let ship_dir = path.join(SHIP_DIR_NAME);
            if ship_dir.exists() && ship_dir.is_dir() {
                projects.push(ProjectDiscovery {
                    name: name.into_owned(),
                    path: ship_dir,
                });
            }
        }
    }
    Ok(projects)
}

pub fn add_mode(project_dir: Option<PathBuf>, mode: ModeConfig) -> Result<()> {
    runtime_add_mode(project_dir, mode)
}

pub fn remove_mode(project_dir: Option<PathBuf>, id: &str) -> Result<()> {
    runtime_remove_mode(project_dir, id)
}

pub fn set_active_mode(project_dir: Option<PathBuf>, id: Option<&str>) -> Result<()> {
    runtime_set_active_mode(project_dir, id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn normalize_registry_project_path_maps_worktree_to_main_ship() -> Result<()> {
        let tmp = tempdir()?;
        let main_root = tmp.path().join("main");
        let main_ship = main_root.join(".ship");
        let common_git = main_root.join(".git");
        let worktree_git = common_git.join("worktrees").join("feature-auth");
        let worktree_root = tmp.path().join("worktrees").join("feature-auth");

        fs::create_dir_all(&main_ship)?;
        fs::create_dir_all(&worktree_git)?;
        fs::create_dir_all(&worktree_root)?;
        fs::write(
            worktree_root.join(".git"),
            format!("gitdir: {}\n", worktree_git.display()),
        )?;

        let normalized = normalize_registry_project_path(&worktree_root);
        assert_eq!(normalized, fs::canonicalize(main_ship)?);
        Ok(())
    }

    #[test]
    fn should_keep_existing_name_only_when_default_would_overwrite_custom() {
        assert!(should_keep_existing_name(
            "Acme Platform",
            "my-repo",
            "my-repo"
        ));
        assert!(!should_keep_existing_name(
            "Acme Platform",
            "Acme Platform",
            "my-repo"
        ));
        assert!(!should_keep_existing_name("my-repo", "my-repo", "my-repo"));
    }

    #[test]
    fn normalize_registry_collapses_duplicates_and_prefers_custom_name() -> Result<()> {
        let tmp = tempdir()?;
        let main_root = tmp.path().join("main");
        let main_ship = main_root.join(".ship");
        let common_git = main_root.join(".git");
        let worktree_git = common_git.join("worktrees").join("feature-auth");
        let worktree_root = tmp.path().join("worktrees").join("feature-auth");

        fs::create_dir_all(&main_ship)?;
        fs::create_dir_all(&worktree_git)?;
        fs::create_dir_all(&worktree_root)?;
        fs::write(
            worktree_root.join(".git"),
            format!("gitdir: {}\n", worktree_git.display()),
        )?;

        let registry = ProjectRegistry {
            projects: vec![
                ProjectEntry {
                    name: "main".to_string(),
                    path: worktree_root.clone(),
                },
                ProjectEntry {
                    name: "Ship Runtime".to_string(),
                    path: main_root.clone(),
                },
            ],
        };

        let (normalized, changed) = normalize_registry(registry, false);
        assert!(changed);
        assert_eq!(normalized.projects.len(), 1);
        assert_eq!(normalized.projects[0].name, "Ship Runtime");
        assert_eq!(normalized.projects[0].path, fs::canonicalize(main_ship)?);
        Ok(())
    }

    #[test]
    fn normalize_registry_fills_empty_name_from_default() -> Result<()> {
        let tmp = tempdir()?;
        let root = tmp.path().join("alpha");
        let ship = root.join(".ship");
        fs::create_dir_all(&ship)?;

        let registry = ProjectRegistry {
            projects: vec![ProjectEntry {
                name: "   ".to_string(),
                path: root.clone(),
            }],
        };

        let (normalized, changed) = normalize_registry(registry, false);
        assert!(changed);
        assert_eq!(normalized.projects.len(), 1);
        assert_eq!(normalized.projects[0].name, "alpha");
        assert_eq!(normalized.projects[0].path, fs::canonicalize(ship)?);
        Ok(())
    }

    #[test]
    fn normalize_registry_filters_transient_paths_when_enabled() {
        let registry = ProjectRegistry {
            projects: vec![
                ProjectEntry {
                    name: "Main".to_string(),
                    path: PathBuf::from("/Users/micah/dev/ship/.ship"),
                },
                ProjectEntry {
                    name: "Tmp".to_string(),
                    path: PathBuf::from("/private/tmp/ship-init-abc/.ship"),
                },
                ProjectEntry {
                    name: "E2E".to_string(),
                    path: PathBuf::from("/Users/micah/dev/ship/examples/e2e/.ship"),
                },
                ProjectEntry {
                    name: "TargetTmp".to_string(),
                    path: PathBuf::from("/Users/micah/dev/ship/target/tmp/ship-e2e/.ship"),
                },
            ],
        };

        let (normalized, changed) = normalize_registry(registry, true);
        assert!(changed);
        assert_eq!(normalized.projects.len(), 1);
        assert_eq!(normalized.projects[0].name, "Main");
    }

    #[test]
    fn transient_path_detection_catches_tmp_var_folders_and_e2e() {
        assert!(should_filter_transient_registry_path(Path::new(
            "/private/tmp/ship-init-abc/.ship"
        )));
        assert!(should_filter_transient_registry_path(Path::new(
            "/private/var/folders/x/T/tmp.123/.ship"
        )));
        assert!(should_filter_transient_registry_path(Path::new(
            "/Users/me/dev/ship/examples/e2e/.ship"
        )));
        assert!(should_filter_transient_registry_path(Path::new(
            "/Users/me/dev/ship/target/tmp/ship-e2e/.ship"
        )));
        assert!(!should_filter_transient_registry_path(Path::new(
            "/Users/me/dev/ship/.ship"
        )));
    }

    #[test]
    fn transient_path_detection_filters_default_global_ship_dir() {
        let Some(registry_path) = default_registry_path() else {
            return;
        };
        let global_dir = registry_path
            .parent()
            .expect("default registry path should have a parent");
        assert!(should_filter_transient_registry_path(global_dir));
    }

    #[test]
    fn seed_builtin_user_skills_writes_workflow_library_and_skill_creator() -> Result<()> {
        let tmp = tempdir()?;
        let user_skills = tmp.path().join("skills");
        seed_builtin_user_skills(&user_skills)?;

        let ship_workflow = user_skills.join("ship-workflow").join("SKILL.md");
        let create_document = user_skills.join("create-document").join("SKILL.md");
        let start_session = user_skills.join("start-session").join("SKILL.md");
        let skill_creator = user_skills.join("skill-creator").join("SKILL.md");
        assert!(ship_workflow.is_file());
        assert!(create_document.is_file());
        assert!(start_session.is_file());
        assert!(skill_creator.is_file());
        let ship_workflow_content = fs::read_to_string(&ship_workflow)?;
        let create_document_content = fs::read_to_string(&create_document)?;
        let start_session_content = fs::read_to_string(&start_session)?;
        let skill_creator_content = fs::read_to_string(&skill_creator)?;
        assert!(
            ship_workflow_content.contains("name: ship-workflow"),
            "ship-workflow template should be seeded"
        );
        assert!(
            ship_workflow_content.contains("## System of Record"),
            "ship-workflow template should include execution contract guidance"
        );
        assert!(
            create_document_content.contains("name: create-document"),
            "create-document template should be seeded"
        );
        assert!(
            start_session_content.contains("name: start-session"),
            "start-session template should be seeded"
        );
        assert!(
            skill_creator_content.contains("name: skill-creator"),
            "skill-creator template should be seeded"
        );
        assert!(
            skill_creator_content.contains("# Skill Creator"),
            "skill-creator template should contain canonical upstream body"
        );
        assert!(
            user_skills
                .join("skill-creator")
                .join("scripts")
                .join("quick_validate.py")
                .is_file(),
            "skill-creator template should include bundled scripts"
        );
        Ok(())
    }

    #[test]
    fn init_project_does_not_seed_modes() -> Result<()> {
        let tmp = tempdir()?;
        let ship_path = init_project(tmp.path().to_path_buf())?;
        let config = get_config(Some(ship_path))?;

        assert!(
            config.modes.is_empty(),
            "ship init should not scaffold any default modes"
        );
        Ok(())
    }

    #[test]
    fn init_project_does_not_create_project_namespace_dir() -> Result<()> {
        let tmp = tempdir()?;
        let ship_path = init_project(tmp.path().to_path_buf())?;
        let config = get_config(Some(ship_path.clone()))?;

        assert!(
            !ship_path.join("project").exists(),
            "ship init should not scaffold .ship/project/"
        );
        assert!(
            config.namespaces.iter().all(|ns| ns.id != "project"),
            "project namespace should not be first-party at init"
        );
        Ok(())
    }

    #[test]
    fn init_project_ensures_ship_mcp_server_when_mcp_toml_exists_without_it() -> Result<()> {
        let tmp = tempdir()?;
        let ship_path = tmp.path().join(".ship");
        fs::create_dir_all(ship_path.join("agents"))?;

        fs::write(
            ship_path.join("agents/mcp.toml"),
            r#"[mcp]
[mcp.servers.github]
name = "GitHub"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-github"]
"#,
        )?;

        let _ = init_project(tmp.path().to_path_buf())?;
        let config = get_config(Some(ship_path.clone()))?;

        assert!(
            config.mcp_servers.iter().any(|server| server.id == "ship"
                && server.command == "ship"
                && server.args == vec!["mcp".to_string(), "serve".to_string()]),
            "ship MCP server should always be present after init"
        );
        assert!(
            config
                .mcp_servers
                .iter()
                .any(|server| server.id == "github"),
            "existing MCP servers should be preserved"
        );
        Ok(())
    }
}
