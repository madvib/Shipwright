use std::path::{Path, PathBuf};

use compiler::PullSkill;
use compiler::types::is_valid_skill_name;
use runtime::{get_skill_vars, list_effective_skills, list_skill_vars, set_skill_var};

use crate::requests::{
    DeleteSkillFileRequest, GetSkillVarsRequest, ListProjectSkillsRequest, ListSkillVarsRequest,
    ListSkillsRequest, SetSkillVarRequest, WriteSkillFileRequest,
};
use crate::tools::studio::{collect_reference_docs, collect_skill_files, parse_skill_frontmatter};

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

/// `list_project_skills` MCP tool — scan skill directories and return all skills as PullSkill objects.
///
/// Reads `project.skill_paths` from the manifest to determine which directories to scan.
/// Falls back to `.ship/skills/` when the field is absent. First-seen ID wins when
/// duplicate skill IDs exist across paths.
pub fn list_project_skills(project_dir: &Path, req: ListProjectSkillsRequest) -> String {
    // project_dir is the project root (e.g. /workspaces/ship).
    // read_skill_paths expects the .ship directory.
    let ship_dir = if project_dir.join(".ship").is_dir() {
        project_dir.join(".ship")
    } else {
        project_dir.to_path_buf()
    };
    let skill_dirs = runtime::read_skill_paths(&ship_dir, project_dir);
    tracing::info!(
        "list_project_skills: ship_dir={}, skill_dirs={:?}",
        ship_dir.display(),
        skill_dirs
    );
    let mut skills: Vec<PullSkill> = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    for skills_dir in &skill_dirs {
        let entries = match std::fs::read_dir(skills_dir) {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!("cannot read {}: {e}", skills_dir.display());
                continue;
            }
        };
        for entry in entries.flatten() {
            if !entry.path().is_dir() {
                continue;
            }
            let id = entry.file_name().to_string_lossy().to_string();
            if !seen_ids.insert(id.clone()) {
                continue; // first path wins
            }
            let skill_dir = entry.path();
            if let Some(skill) = read_skill_from_dir(&id, &skill_dir) {
                skills.push(skill);
            }
        }
    }

    if let Some(ref query) = req.query {
        let q = query.to_ascii_lowercase();
        skills.retain(|s| {
            s.id.to_ascii_lowercase().contains(&q)
                || s.name.to_ascii_lowercase().contains(&q)
                || s.description
                    .as_deref()
                    .unwrap_or("")
                    .to_ascii_lowercase()
                    .contains(&q)
        });
    }

    skills.sort_by(|a, b| a.id.cmp(&b.id));
    serde_json::to_string(&skills).unwrap_or_default()
}

/// Read a single skill from a directory. Returns `None` if `SKILL.md` is missing.
fn read_skill_from_dir(id: &str, skill_dir: &Path) -> Option<PullSkill> {
    let skill_md = skill_dir.join("SKILL.md");
    let content = std::fs::read_to_string(&skill_md).ok()?;
    let fm = parse_skill_frontmatter(&content);
    let files = collect_skill_files(skill_dir);
    let vars_schema = std::fs::read_to_string(skill_dir.join("assets/vars.json"))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok());
    let evals = std::fs::read_to_string(skill_dir.join("evals/evals.json"))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok());
    let reference_docs = collect_reference_docs(skill_dir);

    let events_schema = read_events_schema(skill_dir, fm.stable_id.as_deref().unwrap_or(id));

    Some(PullSkill {
        id: id.to_string(),
        name: fm.name.unwrap_or_else(|| id.to_string()),
        description: fm.description,
        content,
        source: "project".into(),
        stable_id: fm.stable_id,
        tags: fm.tags,
        authors: fm.authors,
        vars_schema,
        events_schema,
        files,
        reference_docs,
        evals,
    })
}

/// Read and expand `assets/events.json` for a skill. Returns `None` if absent.
/// Logs a warning and returns `None` on parse or validation errors.
fn read_events_schema(skill_dir: &Path, stable_id: &str) -> Option<serde_json::Value> {
    let raw_str = std::fs::read_to_string(skill_dir.join("assets/events.json")).ok()?;
    let raw: serde_json::Value = match serde_json::from_str(&raw_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("warn: {}/assets/events.json parse error: {e}", skill_dir.display());
            return None;
        }
    };
    match compiler::events::expand_events(stable_id, &raw) {
        Ok(v) => Some(v),
        Err(e) => {
            eprintln!("warn: {}/assets/events.json: {e}", skill_dir.display());
            None
        }
    }
}

/// `get_skill_vars` MCP tool — return merged variable state for a skill.
pub fn get_skill_vars_tool(ship_dir: &Path, req: GetSkillVarsRequest) -> String {
    match get_skill_vars(ship_dir, &req.skill_id) {
        Ok(Some(vars)) => match serde_json::to_string_pretty(&vars) {
            Ok(json) => json,
            Err(e) => format!("Error serializing vars: {e}"),
        },
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

/// Validate and resolve a skill file path. Returns the absolute path or an error message.
fn resolve_skill_file_path(
    ship_dir: &Path,
    skill_id: &str,
    file_path: &str,
) -> Result<PathBuf, String> {
    if !is_valid_skill_name(skill_id) {
        return Err(format!(
            "Invalid skill_id '{}': must be lowercase alphanumeric with hyphens, 1-64 chars.",
            skill_id
        ));
    }
    if file_path.is_empty() {
        return Err("file_path must not be empty.".into());
    }
    if file_path.starts_with('/') || file_path.starts_with('\\') {
        return Err("file_path must be relative, not absolute.".into());
    }
    if file_path.contains("..") {
        return Err("file_path must not contain '..' (path traversal).".into());
    }
    // Write to the first configured skill_path (default: "skills/")
    let project_root = ship_dir.parent().unwrap_or(ship_dir);
    let write_base = runtime::read_skill_paths(ship_dir, project_root)
        .into_iter()
        .next()
        .unwrap_or_else(|| ship_dir.join("skills"));
    let skill_dir = write_base.join(skill_id);
    let dest = skill_dir.join(file_path);
    // Canonicalize the skill_dir base to ensure the resolved path stays within it.
    // The dest may not exist yet, so we canonicalize the skill_dir (creating it if needed)
    // and check that the dest starts with it.
    if skill_dir.exists() {
        let canon_base = skill_dir
            .canonicalize()
            .map_err(|e| format!("Cannot resolve skill directory: {e}"))?;
        // For dest, resolve through parent if the file does not exist yet.
        let canon_dest = if dest.exists() {
            dest.canonicalize()
                .map_err(|e| format!("Cannot resolve file path: {e}"))?
        } else if let Some(parent) = dest.parent() {
            if parent.exists() {
                let canon_parent = parent
                    .canonicalize()
                    .map_err(|e| format!("Cannot resolve parent directory: {e}"))?;
                canon_parent.join(
                    dest.file_name()
                        .ok_or_else(|| "file_path has no file name component".to_string())?,
                )
            } else {
                // Parent directories will be created; trust the string checks above.
                dest.clone()
            }
        } else {
            dest.clone()
        };
        if !canon_dest.starts_with(&canon_base) {
            return Err("file_path resolves outside the skill directory.".into());
        }
    }
    Ok(dest)
}

/// `write_skill_file` MCP tool — write a file into a skill directory on disk.
pub fn write_skill_file(ship_dir: &Path, req: WriteSkillFileRequest) -> String {
    let dest = match resolve_skill_file_path(ship_dir, &req.skill_id, &req.file_path) {
        Ok(p) => p,
        Err(e) => return format!("Error: {e}"),
    };
    if let Some(parent) = dest.parent()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        return format!("Error creating directories: {e}");
    }
    match std::fs::write(&dest, &req.content) {
        Ok(()) => format!("Wrote {}", dest.display()),
        Err(e) => format!("Error writing file: {e}"),
    }
}

/// `delete_skill_file` MCP tool — delete a single file from a skill directory.
pub fn delete_skill_file(ship_dir: &Path, req: DeleteSkillFileRequest) -> String {
    if req.file_path == "SKILL.md" {
        return "Error: refusing to delete SKILL.md — it defines the skill itself.".into();
    }
    let dest = match resolve_skill_file_path(ship_dir, &req.skill_id, &req.file_path) {
        Ok(p) => p,
        Err(e) => return format!("Error: {e}"),
    };
    if !dest.exists() {
        return format!(
            "Error: file '{}' does not exist in skill '{}'.",
            req.file_path, req.skill_id
        );
    }
    match std::fs::remove_file(&dest) {
        Ok(()) => format!("Deleted {}", dest.display()),
        Err(e) => format!("Error deleting file: {e}"),
    }
}
