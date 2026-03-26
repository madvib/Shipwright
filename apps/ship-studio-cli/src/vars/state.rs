//! Thin CLI state layer for skill variables.
//!
//! Storage is handled by `runtime::skill_vars`:
//! - User-scoped: platform.db KV (`skill_vars:{skill_id}`)
//! - Project-scoped: `.ship/state.json`
//!
//! This module adds only what the CLI needs on top: skill_id validation
//! and the array-append operation.

use anyhow::Result;
use serde_json::Value;
use std::path::Path;

// ── Validation ────────────────────────────────────────────────────────────────

/// Validate that a skill_id is safe for use in path and DB key construction.
///
/// Allowed: lowercase letters, digits, and hyphens. Must start with a letter or digit.
/// Rejects path traversal attempts and other unsafe characters.
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

// ── Wrappers over runtime ─────────────────────────────────────────────────────


/// Append an element to an array variable. Reads current value, appends, writes back.
pub fn append_to_array(skill_id: &str, key: &str, element: &Value, ship_dir: &Path) -> Result<()> {
    let current = runtime::skill_vars::get_skill_vars(ship_dir, skill_id)
        .ok()
        .flatten()
        .unwrap_or_default();

    let mut arr = match current.get(key) {
        Some(Value::Array(a)) => a.clone(),
        None => vec![],
        _ => anyhow::bail!("'{}' is not an array", key),
    };
    arr.push(element.clone());
    runtime::skill_vars::set_skill_var(ship_dir, skill_id, key, Value::Array(arr))
}

/// Reset all state for a skill: clear KV entries and remove from state.json.
pub fn reset_skill_state(skill_id: &str, ship_dir: &Path) -> Result<bool> {
    let mut removed = false;

    // Clear user-scoped vars from KV
    let ns = format!("skill_vars:{}", skill_id);
    let keys = runtime::db::kv::list_keys(&ns)?;
    for k in &keys {
        runtime::db::kv::delete(&ns, k)?;
        removed = true;
    }

    // Clear project-scoped vars from state.json
    let state_path = ship_dir.join("state.json");
    if state_path.exists() {
        let content = std::fs::read_to_string(&state_path)?;
        let mut state: std::collections::HashMap<String, serde_json::Value> =
            serde_json::from_str(&content).unwrap_or_default();
        if state.remove(skill_id).is_some() {
            let json = serde_json::to_string_pretty(&state)?;
            std::fs::write(&state_path, json)?;
            removed = true;
        }
    }

    Ok(removed)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

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
