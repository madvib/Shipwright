use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub const SHIP_DIR_NAME: &str = ".ship";
pub const DEFAULT_STATUSES: &[&str] = &["backlog", "in-progress", "blocked", "done"];
/// Kept for backwards compatibility — prefer DEFAULT_STATUSES or get_project_statuses().
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

    // 2. Traversal logic — any directory containing a .ship folder is a project
    let mut current_dir = start_dir.unwrap_or(env::current_dir()?);
    loop {
        let ship_path = current_dir.join(SHIP_DIR_NAME);
        if ship_path.exists() && ship_path.is_dir() {
            return Ok(ship_path);
        }

        // Check for legacy .project
        let legacy_path = current_dir.join(".project");
        if legacy_path.exists() && legacy_path.is_dir() {
            let ship_path = current_dir.join(SHIP_DIR_NAME);
            fs::rename(&legacy_path, &ship_path).context("Failed to migrate .project to .ship")?;
            return Ok(ship_path);
        }

        if let Some(parent) = current_dir.parent() {
            current_dir = parent.to_path_buf();
        } else {
            return Err(anyhow!(
                "Project tracking not initialized in this directory or its parents. Run `ship init` to create a .ship directory."
            ));
        }
    }
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
    Ok(registry.projects)
}

/// Returns the global config directory (~/.ship)
pub fn get_global_dir() -> Result<PathBuf> {
    home::home_dir()
        .map(|h| h.join(SHIP_DIR_NAME))
        .ok_or_else(|| anyhow!("Could not find home directory"))
}

/// Initializes the .ship directory structure in the given directory.
pub fn init_project(base_dir: PathBuf) -> Result<PathBuf> {
    let ship_path = base_dir.join(SHIP_DIR_NAME);
    fs::create_dir_all(&ship_path)?;

    // Ensure config exists and is normalized with any newly added default fields.
    let config_exists = [
        ship_path.join(crate::config::PRIMARY_CONFIG_FILE),
        ship_path.join(crate::config::SECONDARY_CONFIG_FILE),
        ship_path.join(crate::config::LEGACY_CONFIG_FILE),
    ]
    .iter()
    .any(|path| path.exists());

    let mut config = if config_exists {
        crate::config::get_config(Some(ship_path.clone()))?
    } else {
        crate::config::ProjectConfig::default()
    };
    ensure_first_party_namespaces(&mut config.namespaces);
    if config_exists {
        crate::config::save_config(&config, Some(ship_path.clone()))?;
        cleanup_legacy_config_files(&ship_path)?;
    } else {
        write_initial_config_with_comments(&ship_path, &config)?;
    }
    ensure_registered_namespaces(&ship_path, &config.namespaces)?;

    // project/ namespace
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

    // workflow/ namespace
    let issues = issues_dir(&ship_path);
    for status in DEFAULT_STATUSES {
        fs::create_dir_all(issues.join(status))?;
    }

    let specs = specs_dir(&ship_path);
    for status in SPEC_STATUSES {
        fs::create_dir_all(specs.join(status))?;
    }

    // agents/ namespace
    fs::create_dir_all(modes_dir(&ship_path))?;
    fs::create_dir_all(skills_dir(&ship_path))?;
    fs::create_dir_all(prompts_dir(&ship_path))?;
    fs::create_dir_all(rules_dir(&ship_path))?;

    crate::events::ensure_event_log(&ship_path)?;

    // Write default templates and docs.
    write_default_templates(&ship_path)?;
    write_directory_readmes(&ship_path)?;
    write_default_agent_mode_files(&ship_path)?;
    write_default_skills(&ship_path)?;
    // Write default agent configurations.
    write_if_missing(
        &mcp_config_path(&ship_path),
        include_str!("templates/MCP.toml"),
    )?;
    write_if_missing(
        &permissions_config_path(&ship_path),
        include_str!("templates/PERMISSIONS.toml"),
    )?;

    // Seed core principles in rules
    let principles_path = rules_dir(&ship_path).join("001-core-principles.md");
    write_if_missing(&principles_path, include_str!("templates/RULE.md"))?;

    // Write default .gitignore (opinionated alpha defaults)
    let gitignore_path = ship_path.join(".gitignore");
    if !gitignore_path.exists() {
        let default_git = crate::config::GitConfig::default();
        crate::config::generate_gitignore(&ship_path, &default_git)?;
    }

    // Best-effort init marker in the event stream.
    let _ = crate::events::append_event(
        &ship_path,
        "logic",
        crate::events::EventEntity::Project,
        crate::events::EventAction::Init,
        "project",
        Some("Project initialized".to_string()),
    );

    Ok(ship_path)
}

fn write_default_templates(ship_path: &std::path::Path) -> Result<()> {
    write_if_missing(
        &issues_dir(ship_path).join("TEMPLATE.md"),
        include_str!("templates/ISSUE.md"),
    )?;
    write_if_missing(
        &specs_dir(ship_path).join("TEMPLATE.md"),
        include_str!("templates/SPEC.md"),
    )?;
    write_if_missing(
        &features_dir(ship_path).join("TEMPLATE.md"),
        include_str!("templates/FEATURE.md"),
    )?;
    write_if_missing(
        &releases_dir(ship_path).join("TEMPLATE.md"),
        include_str!("templates/RELEASE.md"),
    )?;
    write_if_missing(
        &adrs_dir(ship_path).join("TEMPLATE.md"),
        include_str!("templates/ADR.md"),
    )?;
    write_if_missing(
        &notes_dir(ship_path).join("TEMPLATE.md"),
        include_str!("templates/NOTE.md"),
    )?;
    write_if_missing(
        &project_ns(ship_path).join("TEMPLATE.md"),
        include_str!("templates/VISION.md"),
    )?;

    // Seed vision.md under project/ namespace.
    let vision_doc = project_ns(ship_path).join("vision.md");
    write_if_missing(&vision_doc, include_str!("templates/VISION.md"))?;
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
            "# agents/\n\nAgent runtime config: prompts, skills, rules, and modes.\n- `mcp.toml`: Model Context Protocol server configuration.\n- `permissions.toml`: Agent capability and access controls.\n- `rules/`: Development and project principles.\n- `skills/`: Reusable agent capabilities (folder-based).\n- `prompts/`: Global and project system instructions.\n- `modes/`: High-level agent behavior presets.\n".to_string(),
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

fn write_initial_config_with_comments(
    ship_path: &Path,
    config: &crate::config::ProjectConfig,
) -> Result<()> {
    let config_path = ship_path.join(crate::config::PRIMARY_CONFIG_FILE);
    let mut content = String::from(
        "# Ship project configuration\n\
         # - Edit with care; prefer `ship config`, `ship mode`, and `ship git` commands where possible.\n\
         # - `namespaces` controls top-level directories under `.ship/`.\n\
         # - Plugin namespaces are dynamically registered when plugins are used.\n\n",
    );
    content.push_str(&toml::to_string_pretty(config)?);
    crate::fs_util::write_atomic(&config_path, content)?;
    Ok(())
}

/// Write `content` to `path` only if it doesn't already exist.
/// Returns `true` if the file was newly written, `false` if it already existed.
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
    let primary = ship_path.join(crate::config::PRIMARY_CONFIG_FILE);
    if !primary.exists() {
        return Ok(());
    }

    for legacy_name in [
        crate::config::SECONDARY_CONFIG_FILE,
        crate::config::LEGACY_CONFIG_FILE,
    ] {
        let legacy = ship_path.join(legacy_name);
        if legacy.exists() {
            fs::remove_file(legacy)?;
        }
    }

    Ok(())
}

fn write_default_agent_mode_files(ship_path: &Path) -> Result<()> {
    let planning = modes_dir(ship_path).join("planning.toml");
    if !planning.exists() {
        fs::write(
            planning,
            "id = \"planning\"\nname = \"Planning\"\nactive_tools = [\"ship_list_notes\", \"ship_create_note\", \"ship_list_specs\", \"ship_get_spec\", \"ship_create_spec\", \"ship_update_spec\", \"ship_list_issues\", \"ship_create_issue\", \"ship_draft_adr\", \"ship_get_project_info\"]\n",
        )?;
    }
    let execution = modes_dir(ship_path).join("execution.toml");
    if !execution.exists() {
        fs::write(
            execution,
            "id = \"execution\"\nname = \"Execution\"\nactive_tools = [\"ship_list_issues\", \"ship_get_issue\", \"ship_update_issue\", \"ship_move_issue\", \"ship_list_notes\", \"ship_create_note\"]\n",
        )?;
    }
    Ok(())
}

fn write_default_skills(ship_path: &Path) -> Result<()> {
    let skill_root = crate::project::skills_dir(ship_path).join("task-policy");
    fs::create_dir_all(&skill_root)?;

    let config_path = skill_root.join("skill.toml");
    let content_path = skill_root.join("index.md");

    if write_if_missing(
        &config_path,
        "id = \"task-policy\"\nname = \"Task Policy\"\nversion = \"0.1.0\"\n",
    )? {
        write_if_missing(&content_path, include_str!("skills/task-policy.md"))?;

        // Newly written — also register in project config's agent.skills so the
        // post-checkout hook includes it automatically without requiring explicit feature config.
        let mut config = crate::config::get_config(Some(ship_path.to_path_buf()))?;
        if !config.agent.skills.contains(&"task-policy".to_string()) {
            config.agent.skills.push("task-policy".to_string());
            crate::config::save_config(&config, Some(ship_path.to_path_buf()))?;
        }
    }
    Ok(())
}

fn ensure_first_party_namespaces(namespaces: &mut Vec<crate::config::NamespaceConfig>) {
    // Legacy compatibility: drop old catch-all plugins/ namespace.
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
            namespaces.push(crate::config::NamespaceConfig {
                id: id.to_string(),
                path: path.to_string(),
                owner: owner.to_string(),
            });
        }
    }
}

fn ensure_registered_namespaces(
    ship_path: &Path,
    namespaces: &[crate::config::NamespaceConfig],
) -> Result<()> {
    const RESERVED_TOP_LEVEL: &[&str] = &[
        "project",
        "workflow",
        "agents",
        "generated",
        "ship.toml",
        "shipwright.toml",
        "config.toml",
        "events.ndjson",
        "log.md",
        "templates",
        "plugins",
    ];

    for ns in namespaces {
        let rel = ns.path.trim();
        if rel.is_empty() {
            continue;
        }
        let rel_path = Path::new(rel);
        if rel_path.is_absolute()
            || rel_path
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return Err(anyhow!(
                "Invalid namespace path '{}' for namespace '{}'",
                ns.path,
                ns.id
            ));
        }
        if ns.id.starts_with("plugin:") {
            let mut components = rel_path.components();
            let first = components
                .next()
                .and_then(|c| c.as_os_str().to_str())
                .ok_or_else(|| anyhow!("Plugin namespace '{}' has an invalid path", ns.id))?;
            if components.next().is_some() {
                return Err(anyhow!(
                    "Plugin namespace '{}' must claim a top-level directory only",
                    ns.id
                ));
            }
            if RESERVED_TOP_LEVEL.contains(&first) {
                return Err(anyhow!(
                    "Plugin namespace '{}' cannot claim reserved path '{}'",
                    ns.id,
                    first
                ));
            }
        }
        fs::create_dir_all(ship_path.join(rel_path))?;
    }
    Ok(())
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
        "issue" | "issues" => Ok(include_str!("templates/ISSUE.md")),
        "adr" | "adrs" => Ok(include_str!("templates/ADR.md")),
        "note" | "notes" => Ok(include_str!("templates/NOTE.md")),
        "spec" | "specs" => Ok(include_str!("templates/SPEC.md")),
        "release" | "releases" => Ok(include_str!("templates/RELEASE.md")),
        "feature" | "features" => Ok(include_str!("templates/FEATURE.md")),
        "vision" => Ok(include_str!("templates/VISION.md")),
        _ => Err(anyhow!("No fallback for template kind: {}", kind)),
    }
}

/// Reads a project template from namespace directories, with legacy and built-in fallback.
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

/// List registered `.ship` namespaces from project config.
pub fn list_registered_namespaces(ship_path: &Path) -> Result<Vec<crate::config::NamespaceConfig>> {
    let config = crate::config::get_config(Some(ship_path.to_path_buf()))?;
    Ok(config.namespaces)
}

/// Register a new namespace and ensure its directory exists.
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
    ensure_first_party_namespaces(&mut config.namespaces);
    crate::config::save_config(&config, Some(ship_path.to_path_buf()))?;
    ensure_registered_namespaces(ship_path, &config.namespaces)
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

    // Collapse consecutive dashes
    while sanitized.contains("--") {
        sanitized = sanitized.replace("--", "-");
    }

    // Trim leading/trailing dashes
    sanitized = sanitized.trim_matches('-').to_string();

    // Limit length to 60 characters for readable filenames
    if sanitized.len() > 60 {
        sanitized.truncate(60);
        sanitized = sanitized.trim_end_matches('-').to_string();
    }

    sanitized
}

/// Returns the human-readable project name from the parent directory of a .ship path.
pub fn get_project_name(ship_path: &std::path::Path) -> String {
    ship_path
        .parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown Project".to_string())
}

// ─── Global App State ─────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AppState {
    #[serde(default)]
    pub active_project: Option<PathBuf>,
    #[serde(default)]
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
