//! Skill variable state access for MCP tools and runtime consumers.
//!
//! Reads `vars.json` and merged state files from the `.ship/` directory.
//! State file paths:
//! - Project-scoped: `.ship/state/skills/{id}.json`
//! - User-scoped:    `~/.ship/state/skills/{id}.json`
//!
//! This module provides read/write access without type validation or audit logging —
//! those are CLI concerns. The MCP surface is intentionally thin.

use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ── Path helpers ───────────────────────────────────────────────────────────────

fn user_state_path(skill_id: &str) -> PathBuf {
    home::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".ship")
        .join("state")
        .join("skills")
        .join(format!("{}.json", skill_id))
}

fn project_state_path(skill_id: &str, ship_dir: &Path) -> PathBuf {
    ship_dir
        .join("state")
        .join("skills")
        .join(format!("{}.json", skill_id))
}

fn skills_dir(ship_dir: &Path) -> PathBuf {
    ship_dir.join("skills")
}

// ── State file I/O ────────────────────────────────────────────────────────────

fn read_state_file(path: &Path) -> HashMap<String, Value> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return HashMap::new();
    };
    let mut map: HashMap<String, Value> = serde_json::from_str(&content).unwrap_or_default();
    map.remove("_meta");
    map
}

fn write_state_file(
    path: &Path,
    skill_id: &str,
    state: &HashMap<String, Value>,
) -> Result<()> {
    let parent = path.parent().unwrap_or(path);
    std::fs::create_dir_all(parent)
        .with_context(|| format!("creating state dir {}", parent.display()))?;

    let mut output = state.clone();
    output.insert(
        "_meta".to_string(),
        serde_json::json!({
            "v": 1,
            "skill": skill_id,
            "migrations": [],
        }),
    );

    let json = serde_json::to_string_pretty(&output)?;
    let mut tmp =
        tempfile::NamedTempFile::new_in(parent).context("creating temp file for state write")?;
    use std::io::Write;
    tmp.write_all(json.as_bytes())
        .context("writing state to temp file")?;
    tmp.persist(path)
        .with_context(|| format!("persisting state file {}", path.display()))?;
    Ok(())
}

// ── vars.json schema (minimal) ────────────────────────────────────────────────

/// Storage hint from vars.json: where the runtime should persist this var.
#[derive(PartialEq)]
enum StorageHint {
    User,
    Project,
}

struct VarMeta {
    storage_hint: StorageHint,
    default: Option<Value>,
}

/// Parse vars.json and return storage hints + defaults for each variable.
fn parse_vars_schema(ship_dir: &Path, skill_id: &str) -> Option<HashMap<String, VarMeta>> {
    let vars_path = skills_dir(ship_dir)
        .join(skill_id)
        .join("vars.json");
    let content = std::fs::read_to_string(&vars_path).ok()?;
    let raw: serde_json::Map<String, Value> = serde_json::from_str(&content).ok()?;

    let mut result = HashMap::new();
    for (key, val) in &raw {
        if key == "$schema" {
            continue;
        }
        let obj = val.as_object()?;
        let hint_str = obj
            .get("storage-hint")
            .and_then(|v| v.as_str())
            .unwrap_or("user");
        let storage_hint = if hint_str == "project" {
            StorageHint::Project
        } else {
            StorageHint::User
        };
        let default = obj.get("default").cloned();
        result.insert(
            key.clone(),
            VarMeta {
                storage_hint,
                default,
            },
        );
    }
    Some(result)
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Read merged variable state for a skill.
///
/// Returns `None` if the skill has no `vars.json`.
/// Merge order (last wins): defaults → user state → project state.
pub fn get_skill_vars(
    ship_dir: &Path,
    skill_id: &str,
) -> Result<Option<HashMap<String, Value>>> {
    let Some(schema) = parse_vars_schema(ship_dir, skill_id) else {
        return Ok(None);
    };

    let mut merged: HashMap<String, Value> = HashMap::new();

    // 1. Defaults
    for (name, meta) in &schema {
        if let Some(v) = &meta.default {
            merged.insert(name.clone(), v.clone());
        }
    }

    // 2. User state
    let user = read_state_file(&user_state_path(skill_id));
    for (k, v) in user {
        if schema.contains_key(&k) {
            merged.insert(k, v);
        }
    }

    // 3. Project state (last wins)
    let project = read_state_file(&project_state_path(skill_id, ship_dir));
    for (k, v) in project {
        if schema.contains_key(&k) {
            merged.insert(k, v);
        }
    }

    Ok(Some(merged))
}

/// Set a single variable value for a skill.
///
/// Reads the var's storage hint from `vars.json` to determine which state file
/// to write. Returns an error if the skill has no `vars.json` or the var is unknown.
pub fn set_skill_var(
    ship_dir: &Path,
    skill_id: &str,
    key: &str,
    value: Value,
) -> Result<()> {
    let schema =
        parse_vars_schema(ship_dir, skill_id).ok_or_else(|| {
            anyhow::anyhow!("skill '{}' has no vars.json", skill_id)
        })?;

    let meta = schema
        .get(key)
        .ok_or_else(|| anyhow::anyhow!("unknown variable '{}' for skill '{}'", key, skill_id))?;

    let path = match meta.storage_hint {
        StorageHint::Project => project_state_path(skill_id, ship_dir),
        StorageHint::User => user_state_path(skill_id),
    };

    let mut state = read_state_file(&path);
    state.insert(key.to_string(), value);
    write_state_file(&path, skill_id, &state)
}

/// List all skills in the project that have a `vars.json`, with their merged state.
pub fn list_skill_vars(ship_dir: &Path) -> Result<Vec<(String, HashMap<String, Value>)>> {
    let skills_path = skills_dir(ship_dir);
    if !skills_path.exists() {
        return Ok(vec![]);
    }

    let mut result = Vec::new();
    for entry in std::fs::read_dir(&skills_path)?.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let vars_path = path.join("vars.json");
        if !vars_path.exists() {
            continue;
        }
        let skill_id = entry.file_name().to_string_lossy().to_string();
        if let Ok(Some(state)) = get_skill_vars(ship_dir, &skill_id) {
            result.push((skill_id, state));
        }
    }
    result.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(result)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write(dir: &Path, rel: &str, content: &str) {
        let p = dir.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, content).unwrap();
    }

    fn sample_vars_json() -> &'static str {
        r#"{
            "commit_style": {
                "type": "enum",
                "default": "conventional",
                "storage-hint": "user",
                "values": ["conventional", "gitmoji"]
            },
            "verbose_mode": {
                "type": "bool",
                "default": false,
                "storage-hint": "project"
            }
        }"#
    }

    #[test]
    fn get_skill_vars_uses_defaults() {
        let tmp = tempdir().unwrap();
        write(
            tmp.path(),
            "skills/commit/vars.json",
            sample_vars_json(),
        );
        let vars = get_skill_vars(tmp.path(), "commit").unwrap().unwrap();
        assert_eq!(vars["commit_style"], Value::String("conventional".into()));
        assert_eq!(vars["verbose_mode"], Value::Bool(false));
    }

    #[test]
    fn get_skill_vars_returns_none_for_no_vars_json() {
        let tmp = tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("skills/no-vars")).unwrap();
        let result = get_skill_vars(tmp.path(), "no-vars").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn set_skill_var_writes_to_project_state() {
        let tmp = tempdir().unwrap();
        write(tmp.path(), "skills/commit/vars.json", sample_vars_json());
        set_skill_var(tmp.path(), "commit", "verbose_mode", Value::Bool(true)).unwrap();
        let path = project_state_path("commit", tmp.path());
        let content = std::fs::read_to_string(&path).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["verbose_mode"], Value::Bool(true));
    }

    #[test]
    fn set_skill_var_unknown_key_errors() {
        let tmp = tempdir().unwrap();
        write(tmp.path(), "skills/commit/vars.json", sample_vars_json());
        let err = set_skill_var(tmp.path(), "commit", "nonexistent", Value::Null);
        assert!(err.is_err());
    }

    #[test]
    fn set_skill_var_no_vars_json_errors() {
        let tmp = tempdir().unwrap();
        let err = set_skill_var(tmp.path(), "no-skill", "key", Value::Null);
        assert!(err.is_err());
    }

    #[test]
    fn list_skill_vars_returns_skills_with_vars() {
        let tmp = tempdir().unwrap();
        write(tmp.path(), "skills/commit/vars.json", sample_vars_json());
        write(tmp.path(), "skills/no-vars/SKILL.md", "no vars here");
        let list = list_skill_vars(tmp.path()).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].0, "commit");
    }

    #[test]
    fn project_state_overlays_default() {
        let tmp = tempdir().unwrap();
        write(tmp.path(), "skills/commit/vars.json", sample_vars_json());
        let proj_path = project_state_path("commit", tmp.path());
        std::fs::create_dir_all(proj_path.parent().unwrap()).unwrap();
        std::fs::write(&proj_path, r#"{"verbose_mode": true}"#).unwrap();
        let vars = get_skill_vars(tmp.path(), "commit").unwrap().unwrap();
        assert_eq!(vars["verbose_mode"], Value::Bool(true));
        assert_eq!(vars["commit_style"], Value::String("conventional".into()));
    }
}
