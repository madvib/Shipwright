//! Skill variable state — read/write for CLI and MCP tools.
//!
//! Storage:
//! - User-scoped vars  → `platform.db` KV store, namespace `skill_vars:{skill_id}`
//! - Project-scoped vars → `.ship/state.json`, keyed by skill id
//!
//! Merge order (last wins): defaults → user state → project state.

use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ── Project state file ────────────────────────────────────────────────────────

fn project_state_path(ship_dir: &Path) -> PathBuf {
    ship_dir.join("state.json")
}

fn read_project_state(ship_dir: &Path) -> HashMap<String, HashMap<String, Value>> {
    let path = project_state_path(ship_dir);
    let Ok(content) = std::fs::read_to_string(&path) else {
        return HashMap::new();
    };
    serde_json::from_str(&content).unwrap_or_default()
}

fn write_project_state(
    ship_dir: &Path,
    state: &HashMap<String, HashMap<String, Value>>,
) -> Result<()> {
    let json = serde_json::to_string_pretty(state)?;
    let path = project_state_path(ship_dir);
    let mut tmp = tempfile::NamedTempFile::new_in(ship_dir)
        .context("creating temp file for state.json")?;
    use std::io::Write;
    tmp.write_all(json.as_bytes())?;
    tmp.persist(&path)
        .with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

// ── vars.json schema (minimal) ────────────────────────────────────────────────

#[derive(PartialEq)]
enum StorageHint {
    User,
    Project,
}

struct VarMeta {
    storage_hint: StorageHint,
    default: Option<Value>,
}

fn parse_vars_schema(ship_dir: &Path, skill_id: &str) -> Option<HashMap<String, VarMeta>> {
    let path = ship_dir
        .join("skills")
        .join(skill_id)
        .join("vars.json");
    let content = std::fs::read_to_string(&path).ok()?;
    let raw: serde_json::Map<String, Value> = serde_json::from_str(&content).ok()?;

    let mut result = HashMap::new();
    for (key, val) in &raw {
        if key == "$schema" {
            continue;
        }
        let obj = val.as_object()?;
        let hint = obj
            .get("storage-hint")
            .and_then(|v| v.as_str())
            .unwrap_or("user");
        let storage_hint = if hint == "project" {
            StorageHint::Project
        } else {
            StorageHint::User
        };
        result.insert(
            key.clone(),
            VarMeta {
                storage_hint,
                default: obj.get("default").cloned(),
            },
        );
    }
    Some(result)
}

// ── KV namespace ──────────────────────────────────────────────────────────────

fn kv_namespace(skill_id: &str) -> String {
    format!("skill_vars:{}", skill_id)
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Read merged variable state for a skill.
///
/// Returns `None` if the skill has no `vars.json`.
/// Merge order: defaults → user state (KV) → project state (state.json).
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

    // 2. User state from platform.db KV
    let ns = kv_namespace(skill_id);
    for name in schema.keys().filter(|k| {
        schema
            .get(*k)
            .is_some_and(|m| m.storage_hint == StorageHint::User)
    }) {
        if let Ok(Some(val)) = crate::db::kv::get(&ns, name) {
            merged.insert(name.clone(), val);
        }
    }

    // 3. Project state from .ship/state.json
    let project_state = read_project_state(ship_dir);
    if let Some(skill_state) = project_state.get(skill_id) {
        for (k, v) in skill_state {
            if schema.contains_key(k) {
                merged.insert(k.clone(), v.clone());
            }
        }
    }

    Ok(Some(merged))
}

/// Set a single variable value for a skill.
///
/// Routes to KV (user-scoped) or `state.json` (project-scoped)
/// based on the var's `storage-hint` in `vars.json`.
pub fn set_skill_var(
    ship_dir: &Path,
    skill_id: &str,
    key: &str,
    value: Value,
) -> Result<()> {
    let schema = parse_vars_schema(ship_dir, skill_id)
        .ok_or_else(|| anyhow::anyhow!("skill '{}' has no vars.json", skill_id))?;

    let meta = schema
        .get(key)
        .ok_or_else(|| anyhow::anyhow!("unknown variable '{}' for skill '{}'", key, skill_id))?;

    match meta.storage_hint {
        StorageHint::User => {
            crate::db::kv::set(&kv_namespace(skill_id), key, &value)?;
        }
        StorageHint::Project => {
            let mut state = read_project_state(ship_dir);
            state
                .entry(skill_id.to_string())
                .or_default()
                .insert(key.to_string(), value);
            write_project_state(ship_dir, &state)?;
        }
    }
    Ok(())
}

/// List all skills in the project that have a `vars.json`, with their merged state.
pub fn list_skill_vars(ship_dir: &Path) -> Result<Vec<(String, HashMap<String, Value>)>> {
    let skills_path = ship_dir.join("skills");
    if !skills_path.exists() {
        return Ok(vec![]);
    }

    let mut result = Vec::new();
    for entry in std::fs::read_dir(&skills_path)?.flatten() {
        let path = entry.path();
        if !path.is_dir() || !path.join("vars.json").exists() {
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
    use crate::db::ensure_db;
    use crate::project::init_project;
    use serde_json::json;
    use tempfile::tempdir;

    fn setup() -> (tempfile::TempDir, PathBuf) {
        let tmp = tempdir().unwrap();
        let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
        ensure_db().unwrap();
        (tmp, ship_dir)
    }

    fn write_vars_json(ship_dir: &Path, skill_id: &str, content: &str) {
        let dir = ship_dir.join("skills").join(skill_id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("vars.json"), content).unwrap();
    }

    const VARS_JSON: &str = r#"{
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
    }"#;

    #[test]
    fn get_skill_vars_uses_defaults() {
        let (_tmp, ship_dir) = setup();
        write_vars_json(&ship_dir, "commit", VARS_JSON);
        let vars = get_skill_vars(&ship_dir, "commit").unwrap().unwrap();
        assert_eq!(vars["commit_style"], json!("conventional"));
        assert_eq!(vars["verbose_mode"], json!(false));
    }

    #[test]
    fn get_skill_vars_returns_none_without_vars_json() {
        let (_tmp, ship_dir) = setup();
        std::fs::create_dir_all(ship_dir.join("skills/no-vars")).unwrap();
        assert!(get_skill_vars(&ship_dir, "no-vars").unwrap().is_none());
    }

    #[test]
    fn set_skill_var_user_scoped_goes_to_kv() {
        let (_tmp, ship_dir) = setup();
        write_vars_json(&ship_dir, "commit", VARS_JSON);
        set_skill_var(&ship_dir, "commit", "commit_style", json!("gitmoji")).unwrap();
        let val = crate::db::kv::get("skill_vars:commit", "commit_style")
            .unwrap()
            .unwrap();
        assert_eq!(val, json!("gitmoji"));
    }

    #[test]
    fn set_skill_var_project_scoped_goes_to_state_json() {
        let (_tmp, ship_dir) = setup();
        write_vars_json(&ship_dir, "commit", VARS_JSON);
        set_skill_var(&ship_dir, "commit", "verbose_mode", json!(true)).unwrap();
        let state = read_project_state(&ship_dir);
        assert_eq!(state["commit"]["verbose_mode"], json!(true));
        // state.json should NOT exist in the wrong place
        assert!(!ship_dir.join("state/skills/commit.json").exists());
    }

    #[test]
    fn set_skill_var_unknown_key_errors() {
        let (_tmp, ship_dir) = setup();
        write_vars_json(&ship_dir, "commit", VARS_JSON);
        assert!(set_skill_var(&ship_dir, "commit", "nonexistent", json!(null)).is_err());
    }

    #[test]
    fn user_state_overlays_default() {
        let (_tmp, ship_dir) = setup();
        write_vars_json(&ship_dir, "commit", VARS_JSON);
        set_skill_var(&ship_dir, "commit", "commit_style", json!("gitmoji")).unwrap();
        let vars = get_skill_vars(&ship_dir, "commit").unwrap().unwrap();
        assert_eq!(vars["commit_style"], json!("gitmoji"));
        assert_eq!(vars["verbose_mode"], json!(false));
    }

    #[test]
    fn project_state_overlays_default() {
        let (_tmp, ship_dir) = setup();
        write_vars_json(&ship_dir, "commit", VARS_JSON);
        set_skill_var(&ship_dir, "commit", "verbose_mode", json!(true)).unwrap();
        let vars = get_skill_vars(&ship_dir, "commit").unwrap().unwrap();
        assert_eq!(vars["verbose_mode"], json!(true));
    }

    #[test]
    fn list_skill_vars_returns_skills_with_vars_json() {
        let (_tmp, ship_dir) = setup();
        write_vars_json(&ship_dir, "commit", VARS_JSON);
        std::fs::create_dir_all(ship_dir.join("skills/no-vars")).unwrap();
        let list = list_skill_vars(&ship_dir).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].0, "commit");
    }
}
