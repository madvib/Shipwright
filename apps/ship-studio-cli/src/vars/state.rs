//! State file I/O for skill variables.
//!
//! State files:
//! - Project scope: `.ship/state/skills/{id}.json`
//! - User scope:    `~/.ship/state/skills/{id}.json`
//!
//! Merge order (last wins): defaults → user state → project state.

use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::schema::{VarDef, VarScope};

// ── Validation ────────────────────────────────────────────────────────────────

/// Validate that a skill_id is safe for use in path construction.
///
/// Allowed: lowercase letters, digits, and hyphens. Must start with a letter or digit.
/// Rejects path traversal attempts (`..`, `/`, `\`) and other unsafe characters.
pub fn validate_skill_id(skill_id: &str) -> Result<()> {
    if skill_id.is_empty() {
        anyhow::bail!("skill id must not be empty");
    }
    if !skill_id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        anyhow::bail!(
            "invalid skill id '{}': only lowercase letters, digits, and hyphens are allowed",
            skill_id
        );
    }
    if skill_id.starts_with('-') || skill_id.ends_with('-') {
        anyhow::bail!(
            "invalid skill id '{}': must not start or end with a hyphen",
            skill_id
        );
    }
    Ok(())
}

// ── Paths ─────────────────────────────────────────────────────────────────────

/// Path to project-scoped state file: `.ship/state/skills/{id}.json`
pub fn project_state_path(skill_id: &str, ship_dir: &Path) -> PathBuf {
    ship_dir
        .join("state")
        .join("skills")
        .join(format!("{}.json", skill_id))
}

/// Path to user-scoped state file: `~/.ship/state/skills/{id}.json`
pub fn user_state_path(skill_id: &str) -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".ship")
        .join("state")
        .join("skills")
        .join(format!("{}.json", skill_id))
}

pub(super) fn changes_log_path(skill_id: &str, ship_dir: &Path) -> PathBuf {
    ship_dir
        .join("state")
        .join("skills")
        .join(format!("{}.changes.jsonl", skill_id))
}

// ── File I/O ──────────────────────────────────────────────────────────────────

pub(super) fn read_state_file(path: &Path) -> HashMap<String, Value> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return HashMap::new();
    };
    let mut map: HashMap<String, Value> =
        serde_json::from_str(&content).unwrap_or_default();
    // Strip runtime metadata — callers only need var values.
    map.remove("_meta");
    map
}

pub(super) fn write_state_file(
    path: &Path,
    skill_id: &str,
    state: &HashMap<String, Value>,
    applied_migrations: &[String],
) -> Result<()> {
    let parent = path.parent().unwrap_or(path);
    std::fs::create_dir_all(parent)
        .with_context(|| format!("creating state dir {}", parent.display()))?;

    // Merge _meta into the output without mutating the caller's map.
    let mut output = state.clone();
    output.insert(
        "_meta".to_string(),
        serde_json::json!({
            "v": 1,
            "skill": skill_id,
            "migrations": applied_migrations,
        }),
    );

    let json = serde_json::to_string_pretty(&output)?;

    // Atomic write: temp file in same directory + rename (POSIX atomic).
    let mut tmp =
        tempfile::NamedTempFile::new_in(parent).context("creating temp file for state write")?;
    use std::io::Write;
    tmp.write_all(json.as_bytes())
        .context("writing state to temp file")?;
    tmp.persist(path)
        .with_context(|| format!("persisting state file {}", path.display()))?;

    Ok(())
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Read merged variable state for a skill.
///
/// Merge order (last wins): var defaults → user state → project state.
/// Only keys present in `var_defs` are included.
/// Returns empty state if `skill_id` fails validation (invalid ids can't have state files).
pub fn read_skill_state(
    skill_id: &str,
    ship_dir: &Path,
    var_defs: &HashMap<String, VarDef>,
) -> HashMap<String, Value> {
    let mut state: HashMap<String, Value> = HashMap::new();

    for (name, def) in var_defs {
        if let Some(v) = &def.default {
            state.insert(name.clone(), v.clone());
        }
    }

    let user_state = read_state_file(&user_state_path(skill_id));
    for (k, v) in user_state {
        if var_defs.contains_key(&k) {
            state.insert(k, v);
        }
    }

    let project_state = read_state_file(&project_state_path(skill_id, ship_dir));
    for (k, v) in project_state {
        if var_defs.contains_key(&k) {
            state.insert(k, v);
        }
    }

    state
}

/// Write a single variable value to the appropriate state file.
pub fn write_skill_state(
    skill_id: &str,
    key: &str,
    value: &Value,
    scope: &VarScope,
    ship_dir: &Path,
) -> Result<()> {
    let path = match scope {
        VarScope::Project => project_state_path(skill_id, ship_dir),
        VarScope::User => user_state_path(skill_id),
    };
    let mut state = read_state_file(&path);
    let old = state.get(key).cloned();
    state.insert(key.to_string(), value.clone());
    write_state_file(&path, skill_id, &state, &[])?;
    append_changes_log(skill_id, key, old.as_ref(), value, "user:cli", ship_dir)?;
    Ok(())
}

/// Append an element to an array variable in the appropriate state file.
pub fn append_to_array(
    skill_id: &str,
    key: &str,
    element: &Value,
    scope: &VarScope,
    ship_dir: &Path,
) -> Result<()> {
    let path = match scope {
        VarScope::Project => project_state_path(skill_id, ship_dir),
        VarScope::User => user_state_path(skill_id),
    };
    let mut state = read_state_file(&path);
    let old = state.get(key).cloned();
    let arr = state
        .entry(key.to_string())
        .or_insert_with(|| Value::Array(vec![]));
    match arr {
        Value::Array(a) => a.push(element.clone()),
        _ => anyhow::bail!("'{}' is not an array", key),
    }
    let new = state[key].clone();
    write_state_file(&path, skill_id, &state, &[])?;
    append_changes_log(skill_id, key, old.as_ref(), &new, "user:cli", ship_dir)?;
    Ok(())
}

/// Create default state files from var definitions.
///
/// Preserves existing values; adds new key defaults; drops unknown keys.
pub fn create_default_state(
    skill_id: &str,
    var_defs: &HashMap<String, VarDef>,
    ship_dir: &Path,
) -> Result<()> {
    let proj_path = project_state_path(skill_id, ship_dir);
    let mut proj_state = read_state_file(&proj_path);
    let proj_vars: Vec<(&String, &VarDef)> = var_defs
        .iter()
        .filter(|(_, d)| d.scope == VarScope::Project)
        .collect();
    if !proj_vars.is_empty() {
        proj_state.retain(|k, _| var_defs.contains_key(k));
        for (name, def) in &proj_vars {
            if !proj_state.contains_key(*name) {
                if let Some(v) = &def.default {
                    proj_state.insert(name.to_string(), v.clone());
                }
            }
        }
        write_state_file(&proj_path, skill_id, &proj_state, &[])?;
    }

    let user_path = user_state_path(skill_id);
    let mut user_state = read_state_file(&user_path);
    let user_vars: Vec<(&String, &VarDef)> = var_defs
        .iter()
        .filter(|(_, d)| d.scope == VarScope::User)
        .collect();
    if !user_vars.is_empty() {
        user_state.retain(|k, _| var_defs.contains_key(k));
        for (name, def) in &user_vars {
            if !user_state.contains_key(*name) {
                if let Some(v) = &def.default {
                    user_state.insert(name.to_string(), v.clone());
                }
            }
        }
        write_state_file(&user_path, skill_id, &user_state, &[])?;
    }

    Ok(())
}

/// Append an entry to the changes JSONL log.
fn append_changes_log(
    skill_id: &str,
    key: &str,
    old: Option<&Value>,
    new: &Value,
    actor: &str,
    ship_dir: &Path,
) -> Result<()> {
    let log_path = changes_log_path(skill_id, ship_dir);
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let ts = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let entry = serde_json::json!({
        "ts": ts,
        "key": key,
        "from": old,
        "to": new,
        "actor": actor,
    });
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .with_context(|| format!("opening changes log {}", log_path.display()))?;
    writeln!(f, "{}", serde_json::to_string(&entry)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn sample_defs() -> HashMap<String, VarDef> {
        super::super::schema::parse_vars_json(r#"{
            "commit_style": {
                "type": "enum",
                "default": "conventional",
                "scope": "user",
                "values": ["conventional", "gitmoji"]
            },
            "verbose_mode": {
                "type": "bool",
                "default": false,
                "scope": "project"
            },
            "team_members": {
                "type": "array",
                "scope": "project"
            }
        }"#).unwrap()
    }

    #[test]
    fn read_skill_state_uses_defaults() {
        let tmp = tempdir().unwrap();
        let defs = sample_defs();
        let state = read_skill_state("my-skill", tmp.path(), &defs);
        assert_eq!(state.get("commit_style"), Some(&Value::String("conventional".into())));
        assert_eq!(state.get("verbose_mode"), Some(&Value::Bool(false)));
        assert!(!state.contains_key("team_members"));
    }

    #[test]
    fn project_state_overlays_defaults() {
        let tmp = tempdir().unwrap();
        let defs = sample_defs();
        let proj_path = project_state_path("my-skill", tmp.path());
        std::fs::create_dir_all(proj_path.parent().unwrap()).unwrap();
        std::fs::write(&proj_path, r#"{"verbose_mode":true}"#).unwrap();
        let state = read_skill_state("my-skill", tmp.path(), &defs);
        assert_eq!(state.get("verbose_mode"), Some(&Value::Bool(true)));
        assert_eq!(state.get("commit_style"), Some(&Value::String("conventional".into())));
    }

    #[test]
    fn write_skill_state_creates_file() {
        let tmp = tempdir().unwrap();
        write_skill_state("my-skill", "verbose_mode", &Value::Bool(true), &VarScope::Project, tmp.path()).unwrap();
        let proj_path = project_state_path("my-skill", tmp.path());
        assert!(proj_path.exists());
        let content = std::fs::read_to_string(&proj_path).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["verbose_mode"], Value::Bool(true));
    }

    #[test]
    fn create_default_state_writes_files() {
        let tmp = tempdir().unwrap();
        let defs = sample_defs();
        create_default_state("my-skill", &defs, tmp.path()).unwrap();
        let proj_path = project_state_path("my-skill", tmp.path());
        assert!(proj_path.exists());
        let content = std::fs::read_to_string(&proj_path).unwrap();
        let parsed: serde_json::Map<String, Value> = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.get("verbose_mode"), Some(&Value::Bool(false)));
        assert!(!parsed.contains_key("commit_style"));
    }

    #[test]
    fn create_default_state_preserves_existing_values() {
        let tmp = tempdir().unwrap();
        let defs = sample_defs();
        let proj_path = project_state_path("my-skill", tmp.path());
        std::fs::create_dir_all(proj_path.parent().unwrap()).unwrap();
        std::fs::write(&proj_path, r#"{"verbose_mode":true}"#).unwrap();
        create_default_state("my-skill", &defs, tmp.path()).unwrap();
        let content = std::fs::read_to_string(&proj_path).unwrap();
        let parsed: serde_json::Map<String, Value> = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.get("verbose_mode"), Some(&Value::Bool(true)));
    }

    #[test]
    fn valid_skill_ids_pass() {
        assert!(validate_skill_id("my-skill").is_ok());
        assert!(validate_skill_id("commit").is_ok());
        assert!(validate_skill_id("review-pr").is_ok());
        assert!(validate_skill_id("skill123").is_ok());
    }

    #[test]
    fn path_traversal_rejected() {
        assert!(validate_skill_id("../etc/passwd").is_err());
        assert!(validate_skill_id("../../evil").is_err());
        assert!(validate_skill_id("skill/evil").is_err());
        assert!(validate_skill_id("skill\\evil").is_err());
    }

    #[test]
    fn invalid_skill_ids_rejected() {
        assert!(validate_skill_id("").is_err());
        assert!(validate_skill_id("-starts-with-hyphen").is_err());
        assert!(validate_skill_id("ends-with-hyphen-").is_err());
        assert!(validate_skill_id("UPPERCASE").is_err());
        assert!(validate_skill_id("has space").is_err());
    }
}
