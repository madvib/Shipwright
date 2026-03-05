use anyhow::{Context, Result, anyhow};
use runtime::config::{
    ModeConfig, NamespaceConfig, ProjectConfig, add_mode as runtime_add_mode,
    ensure_registered_namespaces as runtime_ensure_registered_namespaces, get_config,
    remove_mode as runtime_remove_mode, save_config, set_active_mode as runtime_set_active_mode,
};
use runtime::fs_util::write_atomic;
pub use runtime::project::{ProjectEntry, ProjectRegistry};
use runtime::project::{
    SHIP_DIR_NAME, adrs_dir as runtime_adrs_dir, agents_ns as runtime_agents_ns,
    features_dir as runtime_features_dir, generated_ns as runtime_generated_ns, get_global_dir,
    get_project_dir as runtime_get_project_dir, get_project_name as runtime_get_project_name,
    issues_dir as runtime_issues_dir, mcp_config_path as runtime_mcp_config_path,
    modes_dir as runtime_modes_dir, notes_dir as runtime_notes_dir,
    permissions_config_path as runtime_permissions_config_path, project_ns as runtime_project_ns,
    register_ship_namespace as runtime_register_ship_namespace,
    releases_dir as runtime_releases_dir,
    resolve_project_ship_dir as runtime_resolve_project_ship_dir, rules_dir as runtime_rules_dir,
    sanitize_file_name as runtime_sanitize_file_name,
    ship_dir_from_path as runtime_ship_dir_from_path, skills_dir as runtime_skills_dir,
    specs_dir as runtime_specs_dir, upcoming_releases_dir as runtime_upcoming_releases_dir,
    workflow_ns as runtime_workflow_ns,
};
use runtime::{EventAction, EventEntity, append_event};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub const DEFAULT_STATUSES: &[&str] = &["backlog", "in-progress", "blocked", "done"];
pub const ISSUE_STATUSES: &[&str] = DEFAULT_STATUSES;
pub const ADR_STATUSES: &[&str] = &[
    "proposed",
    "accepted",
    "rejected",
    "superseded",
    "deprecated",
];
pub const FEATURE_STATUSES: &[&str] = &["planned", "in-progress", "implemented", "deprecated"];
pub const SPEC_STATUSES: &[&str] = &["draft", "active", "archived"];

// ── Namespace path helpers ────────────────────────────────────────────────────

pub fn project_ns(ship_dir: &Path) -> PathBuf {
    runtime_project_ns(ship_dir)
}

pub fn workflow_ns(ship_dir: &Path) -> PathBuf {
    runtime_workflow_ns(ship_dir)
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

pub fn specs_dir(ship_dir: &Path) -> PathBuf {
    runtime_specs_dir(ship_dir)
}

pub fn features_dir(ship_dir: &Path) -> PathBuf {
    runtime_features_dir(ship_dir)
}

pub fn issues_dir(ship_dir: &Path) -> PathBuf {
    runtime_issues_dir(ship_dir)
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

    path_contains_component_sequence(path, &["example", "projects-e2e"])
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
        ship_path.join(runtime::config::SECONDARY_CONFIG_FILE),
        ship_path.join(runtime::config::LEGACY_CONFIG_FILE),
    ]
    .iter()
    .any(|path| path.exists());

    let mut config = if config_exists {
        get_config(Some(ship_path.clone()))?
    } else {
        ProjectConfig::default()
    };
    ensure_first_party_namespaces(&mut config.namespaces);
    if config_exists {
        save_config(&config, Some(ship_path.clone()))?;
        cleanup_legacy_config_files(&ship_path)?;
    } else {
        write_initial_config_with_comments(&ship_path, &config)?;
    }
    ensure_registered_namespaces(&ship_path, &config.namespaces)?;

    fs::create_dir_all(releases_dir(&ship_path))?;
    fs::create_dir_all(upcoming_releases_dir(&ship_path))?;

    let adrs = adrs_dir(&ship_path);
    for status in ADR_STATUSES {
        fs::create_dir_all(adrs.join(status))?;
    }

    fs::create_dir_all(notes_dir(&ship_path))?;

    let features = features_dir(&ship_path);
    for status in FEATURE_STATUSES {
        fs::create_dir_all(features.join(status))?;
    }

    let issues = issues_dir(&ship_path);
    for status in DEFAULT_STATUSES {
        fs::create_dir_all(issues.join(status))?;
    }

    let specs = specs_dir(&ship_path);
    for status in SPEC_STATUSES {
        fs::create_dir_all(specs.join(status))?;
    }

    fs::create_dir_all(skills_dir(&ship_path))?;
    fs::create_dir_all(rules_dir(&ship_path))?;

    runtime::events::ensure_event_log(&ship_path)?;

    write_default_templates(&ship_path)?;
    write_directory_readmes(&ship_path)?;
    write_default_agent_mode_files(&ship_path)?;
    write_default_skills(&ship_path)?;
    write_if_missing(
        &mcp_config_path(&ship_path),
        include_str!("../../../../core/runtime/src/templates/MCP.toml"),
    )?;
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

fn write_default_templates(ship_path: &Path) -> Result<()> {
    write_if_missing(
        &issues_dir(ship_path).join("TEMPLATE.md"),
        include_str!("../../../../core/runtime/src/templates/ISSUE.md"),
    )?;
    write_if_missing(
        &specs_dir(ship_path).join("TEMPLATE.md"),
        include_str!("../../../../core/runtime/src/templates/SPEC.md"),
    )?;
    write_if_missing(
        &features_dir(ship_path).join("TEMPLATE.md"),
        include_str!("../../../../core/runtime/src/templates/FEATURE.md"),
    )?;
    write_if_missing(
        &releases_dir(ship_path).join("TEMPLATE.md"),
        include_str!("../../../../core/runtime/src/templates/RELEASE.md"),
    )?;
    write_if_missing(
        &adrs_dir(ship_path).join("TEMPLATE.md"),
        include_str!("../../../../core/runtime/src/templates/ADR.md"),
    )?;
    write_if_missing(
        &notes_dir(ship_path).join("TEMPLATE.md"),
        include_str!("../../../../core/runtime/src/templates/NOTE.md"),
    )?;
    write_if_missing(
        &project_ns(ship_path).join("TEMPLATE.md"),
        include_str!("../../../../core/runtime/src/templates/VISION.md"),
    )?;

    let vision_doc = project_ns(ship_path).join("vision.md");
    write_if_missing(
        &vision_doc,
        include_str!("../../../../core/runtime/src/templates/VISION.md"),
    )?;
    Ok(())
}

fn write_directory_readmes(ship_path: &Path) -> Result<()> {
    let readmes = [
        (
            ship_path.to_path_buf(),
            "# .ship\n\nShip runtime data for this project. Files here are created and updated by Ship tools.\n".to_string(),
        ),
        (
            project_ns(ship_path),
            "# project/\n\nProject-level docs and long-lived context.\n- `vision.md`\n- `releases/`\n- `adrs/`\n- `notes/`\n".to_string(),
        ),
        (
            releases_dir(ship_path),
            "# project/releases/\n\nRelease plans and release state. Workflow items can reference these files.\n".to_string(),
        ),
        (
            upcoming_releases_dir(ship_path),
            "# project/releases/upcoming/\n\nPlanned or active releases that have not shipped yet.\n".to_string(),
        ),
        (
            adrs_dir(ship_path),
            format!("# project/adrs/\n\nArchitecture Decision Records, organized by status:\n- {}\n", ADR_STATUSES.join("\n- ")),
        ),
        (
            notes_dir(ship_path),
            "# project/notes/\n\nProject-scoped notes.\n".to_string(),
        ),
        (
            workflow_ns(ship_path),
            "# workflow/\n\nExecution artifacts for ongoing work.\n- `issues/`\n- `specs/`\n- `features/`\n".to_string(),
        ),
        (
            issues_dir(ship_path),
            format!("# workflow/issues/\n\nGranular implementation tasks, organized by status:\n- {}\n", DEFAULT_STATUSES.join("\n- ")),
        ),
        (
            specs_dir(ship_path),
            format!("# workflow/specs/\n\nProduct/technical specifications, organized by status:\n- {}\n", SPEC_STATUSES.join("\n- ")),
        ),
        (
            features_dir(ship_path),
            format!("# project/features/\n\nHigh-level project features, organized by status:\n- {}\n", FEATURE_STATUSES.join("\n- ")),
        ),
        (
            agents_ns(ship_path),
            "# agents/\n\nAgent runtime config and policy.\n- `mcp.toml`: Model Context Protocol server configuration.\n- `permissions.toml`: Agent capability and access controls.\n- `rules/`: Development and project principles.\n- Skills are stored outside repo state (`~/.ship/skills` and `~/.ship/projects/<project>/skills`).\n- Modes: persisted in SQLite runtime state.\n".to_string(),
        ),
        (
            rules_dir(ship_path),
            "# agents/rules/\n\nProject-scoped development rules and principles. Agents should consult these for every task.\n".to_string(),
        ),
        (
            generated_ns(ship_path),
            "# generated/\n\nRuntime-generated transient artifacts.\n".to_string(),
        ),
    ];

    for (dir, content) in readmes {
        write_if_missing(&dir.join("README.md"), &content)?;
    }

    Ok(())
}

fn write_initial_config_with_comments(ship_path: &Path, config: &ProjectConfig) -> Result<()> {
    let config_path = ship_path.join(runtime::config::PRIMARY_CONFIG_FILE);
    let mut content = String::from(
        "# Ship project configuration\n\
         # - Edit with care; prefer `ship config`, `ship mode`, and `ship git` commands where possible.\n\
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

    for legacy_name in [
        runtime::config::SECONDARY_CONFIG_FILE,
        runtime::config::LEGACY_CONFIG_FILE,
    ] {
        let legacy = ship_path.join(legacy_name);
        if legacy.exists() {
            fs::remove_file(legacy)?;
        }
    }

    Ok(())
}

fn default_agent_modes() -> Vec<ModeConfig> {
    vec![
        ModeConfig {
            id: "planning".to_string(),
            name: "Planning".to_string(),
            description: Some(
                "High-context planning for specs, issues, and ADR prep before coding.".to_string(),
            ),
            active_tools: vec![
                "ship_list_notes".to_string(),
                "ship_create_note".to_string(),
                "ship_list_specs".to_string(),
                "ship_get_spec".to_string(),
                "ship_create_spec".to_string(),
                "ship_update_spec".to_string(),
                "ship_list_issues".to_string(),
                "ship_create_issue".to_string(),
                "ship_draft_adr".to_string(),
                "ship_get_project_info".to_string(),
            ],
            ..Default::default()
        },
        ModeConfig {
            id: "code".to_string(),
            name: "Code".to_string(),
            description: Some(
                "Execution-focused mode for implementing and moving work through issue status."
                    .to_string(),
            ),
            active_tools: vec![
                "ship_list_issues".to_string(),
                "ship_get_issue".to_string(),
                "ship_update_issue".to_string(),
                "ship_move_issue".to_string(),
                "ship_list_specs".to_string(),
                "ship_get_spec".to_string(),
                "ship_update_spec".to_string(),
                "ship_list_notes".to_string(),
                "ship_create_note".to_string(),
                "ship_git_feature_sync".to_string(),
            ],
            ..Default::default()
        },
        ModeConfig {
            id: "config".to_string(),
            name: "Config".to_string(),
            description: Some(
                "Configuration and environment mode for skills, providers, hooks, and project policy."
                    .to_string(),
            ),
            active_tools: vec![
                "ship_get_project_info".to_string(),
                "ship_create_skill".to_string(),
                "ship_update_skill".to_string(),
                "ship_delete_skill".to_string(),
                "ship_git_config_set".to_string(),
                "ship_git_hooks_install".to_string(),
                "ship_detect_providers".to_string(),
                "ship_connect_provider".to_string(),
                "ship_disconnect_provider".to_string(),
            ],
            ..Default::default()
        },
    ]
}

fn write_default_agent_mode_files(ship_path: &Path) -> Result<()> {
    let project_dir = Some(ship_path.to_path_buf());
    let config = get_config(project_dir.clone())?;
    for mode in default_agent_modes() {
        if !config.modes.iter().any(|existing| existing.id == mode.id) {
            runtime_add_mode(project_dir.clone(), mode)?;
        }
    }
    Ok(())
}

fn write_default_skills(ship_path: &Path) -> Result<()> {
    let skill_root = skills_dir(ship_path).join("task-policy");
    fs::create_dir_all(&skill_root)?;

    write_if_missing(
        &skill_root.join("SKILL.md"),
        r#"---
name: task-policy
description: Ship workflow policy and execution guardrails for daily delivery.
metadata:
  display_name: Shipwright Workflow Policy
  source: builtin
---

# Shipwright Workflow Policy

Use Ship as the system of record for workflow state changes.

## Canonical Flow

Vision -> Release -> Feature -> Spec -> Issues -> Close Feature -> Ship Release
"#,
    )?;

    let mut config = get_config(Some(ship_path.to_path_buf()))?;
    if !config.agent.skills.contains(&"task-policy".to_string()) {
        config.agent.skills.push("task-policy".to_string());
        save_config(&config, Some(ship_path.to_path_buf()))?;
    }
    seed_builtin_user_skills(&runtime::project::user_skills_dir())?;
    Ok(())
}

fn seed_builtin_user_skills(user_skills_root: &Path) -> Result<()> {
    fs::create_dir_all(user_skills_root)?;
    let ship_workflow_path = user_skills_root.join("ship-workflow").join("SKILL.md");
    write_if_missing(
        &ship_workflow_path,
        include_str!("../../../../core/runtime/src/templates/skills/ship-workflow.SKILL.md"),
    )?;

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
    namespaces.retain(|ns| !(ns.id == "plugins" && ns.path == "plugins"));

    let required = [
        ("project", "project", "project"),
        ("workflow", "workflow", "workflow"),
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

fn ensure_registered_namespaces(ship_path: &Path, namespaces: &[NamespaceConfig]) -> Result<()> {
    runtime_ensure_registered_namespaces(ship_path, namespaces)
}

fn template_rel_path(kind: &str) -> Result<&'static str> {
    match kind {
        "issue" | "issues" => Ok("workflow/issues/TEMPLATE.md"),
        "adr" | "adrs" => Ok("project/adrs/TEMPLATE.md"),
        "note" | "notes" => Ok("project/notes/TEMPLATE.md"),
        "spec" | "specs" => Ok("workflow/specs/TEMPLATE.md"),
        "release" | "releases" => Ok("project/releases/TEMPLATE.md"),
        "feature" | "features" => Ok("project/features/TEMPLATE.md"),
        "vision" => Ok("project/TEMPLATE.md"),
        _ => Err(anyhow!("Unknown template kind: {}", kind)),
    }
}

fn legacy_template_file_name(kind: &str) -> Option<&'static str> {
    match kind {
        "issue" | "issues" => Some("ISSUE.md"),
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
        "issue" | "issues" => Ok(include_str!(
            "../../../../core/runtime/src/templates/ISSUE.md"
        )),
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
    #[serde(default)]
    pub issue_count: usize,
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
                    issue_count: 0,
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
                    name: "Shipwright Runtime".to_string(),
                    path: main_root.clone(),
                },
            ],
        };

        let (normalized, changed) = normalize_registry(registry, false);
        assert!(changed);
        assert_eq!(normalized.projects.len(), 1);
        assert_eq!(normalized.projects[0].name, "Shipwright Runtime");
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
                    path: PathBuf::from("/Users/micah/dev/shipwright/.ship"),
                },
                ProjectEntry {
                    name: "Tmp".to_string(),
                    path: PathBuf::from("/private/tmp/ship-init-abc/.ship"),
                },
                ProjectEntry {
                    name: "E2E".to_string(),
                    path: PathBuf::from("/Users/micah/dev/shipwright/example/projects-e2e/.ship"),
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
            "/Users/me/dev/shipwright/example/projects-e2e/.ship"
        )));
        assert!(!should_filter_transient_registry_path(Path::new(
            "/Users/me/dev/shipwright/.ship"
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
    fn seed_builtin_user_skills_writes_ship_workflow_and_skill_creator() -> Result<()> {
        let tmp = tempdir()?;
        let user_skills = tmp.path().join("skills");
        seed_builtin_user_skills(&user_skills)?;

        let ship_workflow = user_skills.join("ship-workflow").join("SKILL.md");
        let skill_creator = user_skills.join("skill-creator").join("SKILL.md");
        assert!(ship_workflow.is_file());
        assert!(skill_creator.is_file());
        let ship_workflow_content = fs::read_to_string(&ship_workflow)?;
        let skill_creator_content = fs::read_to_string(&skill_creator)?;
        assert!(
            ship_workflow_content.contains("name: ship-workflow"),
            "ship-workflow template should be seeded"
        );
        assert!(
            ship_workflow_content.contains("## System Of Record Contract"),
            "ship-workflow template should include execution contract guidance"
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
    fn init_project_seeds_default_planning_code_and_config_modes() -> Result<()> {
        let tmp = tempdir()?;
        let ship_path = init_project(tmp.path().to_path_buf())?;
        let config = get_config(Some(ship_path))?;

        assert!(config.modes.iter().any(|mode| mode.id == "planning"));
        assert!(config.modes.iter().any(|mode| mode.id == "code"));
        assert!(config.modes.iter().any(|mode| mode.id == "config"));
        assert!(
            !config.modes.iter().any(|mode| mode.id == "execution"),
            "new init should not seed legacy execution mode"
        );
        Ok(())
    }

    #[test]
    fn write_default_agent_mode_files_is_idempotent() -> Result<()> {
        let tmp = tempdir()?;
        let ship_path = init_project(tmp.path().to_path_buf())?;
        write_default_agent_mode_files(&ship_path)?;
        let config = get_config(Some(ship_path))?;

        for id in ["planning", "code", "config"] {
            let count = config.modes.iter().filter(|mode| mode.id == id).count();
            assert_eq!(count, 1, "mode '{}' should be present exactly once", id);
        }
        Ok(())
    }
}
