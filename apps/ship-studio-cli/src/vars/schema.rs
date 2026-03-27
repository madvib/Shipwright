//! Var schema types and vars.json parser.

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VarType {
    String,
    Bool,
    Enum,
    Array,
    Object,
}

/// Storage hint declared by the skill author. All scopes use platform.db KV.
///
/// - `global`  — machine-wide preference, shared across all contexts
/// - `local`   — this context only, not shared (personal override)
/// - `project` — this context, intended to be shared with the team
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageHint {
    Global,
    Local,
    Project,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct VarDef {
    #[serde(rename = "type", default)]
    pub var_type: VarType,
    pub default: Option<Value>,
    /// Storage scope hint. `user` → platform.db KV, `project` → .ship/state.json.
    #[serde(rename = "storage-hint", default)]
    pub storage_hint: StorageHint,
    /// Allowed values (enum type only).
    #[serde(default)]
    pub values: Vec<String>,
    /// Human-readable label for Studio UI.
    pub label: Option<String>,
    /// Description shown in Studio and `ship vars get`.
    pub description: Option<String>,
}

impl Default for VarType {
    fn default() -> Self {
        VarType::String
    }
}

impl Default for StorageHint {
    fn default() -> Self {
        StorageHint::Global
    }
}

// ── Validation ────────────────────────────────────────────────────────────────

/// Warn to stderr for any enum vars whose current state value is not in the allowed list.
///
/// Called at compile time after merging state. Does not fail the build — the compiler
/// renders whatever value is in state, but authors should be aware of the mismatch.
pub fn warn_invalid_enum_vars(
    skill_id: &str,
    var_defs: &HashMap<String, VarDef>,
    state: &HashMap<String, Value>,
) {
    for (name, def) in var_defs {
        if def.var_type != VarType::Enum || def.values.is_empty() {
            continue;
        }
        let Some(val) = state.get(name) else {
            continue;
        };
        let Some(s) = val.as_str() else {
            continue;
        };
        if !def.values.contains(&s.to_string()) {
            eprintln!(
                "warning: skill '{}': var '{}' has value '{}' which is not in allowed values: {}",
                skill_id,
                name,
                s,
                def.values.join(", ")
            );
        }
    }
}

// ── Parser ────────────────────────────────────────────────────────────────────

/// Parse a vars.json file into a map of variable name → definition.
///
/// The JSON object keys are variable names. The `$schema` key is ignored.
pub fn parse_vars_json(content: &str) -> Result<HashMap<String, VarDef>> {
    let raw: serde_json::Map<String, Value> =
        serde_json::from_str(content).context("vars.json is not valid JSON")?;

    let mut result = HashMap::new();
    for (key, val) in raw {
        if key == "$schema" {
            continue;
        }
        let def: VarDef = serde_json::from_value(val)
            .with_context(|| format!("invalid var definition for '{key}'"))?;
        result.insert(key, def);
    }
    Ok(result)
}

/// Load and parse a vars.json file from disk.
pub fn load_vars_json(path: &Path) -> Result<HashMap<String, VarDef>> {
    let content =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    parse_vars_json(&content)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const VARS_JSON: &str = r#"{
        "$schema": "https://agentskills.io/schemas/vars/v1.json",
        "commit_style": {
            "type": "enum",
            "default": "conventional",
            "storage-hint": "global",
            "values": ["conventional", "gitmoji", "angular"],
            "label": "Commit style",
            "description": "Format applied to every commit message"
        },
        "verbose_mode": {
            "type": "bool",
            "default": false,
            "storage-hint": "project"
        },
        "team_members": {
            "type": "array",
            "storage-hint": "local"
        }
    }"#;

    #[test]
    fn parse_vars_json_basic() {
        let defs = parse_vars_json(VARS_JSON).unwrap();
        assert_eq!(defs.len(), 3);

        let commit = defs.get("commit_style").unwrap();
        assert_eq!(commit.var_type, VarType::Enum);
        assert_eq!(commit.default, Some(Value::String("conventional".into())));
        assert_eq!(commit.storage_hint, StorageHint::Global);
        assert_eq!(commit.values, ["conventional", "gitmoji", "angular"]);
        assert_eq!(commit.label.as_deref(), Some("Commit style"));

        let verbose = defs.get("verbose_mode").unwrap();
        assert_eq!(verbose.var_type, VarType::Bool);
        assert_eq!(verbose.default, Some(Value::Bool(false)));
        assert_eq!(verbose.storage_hint, StorageHint::Project);

        let members = defs.get("team_members").unwrap();
        assert_eq!(members.var_type, VarType::Array);
        assert!(members.default.is_none());
        assert_eq!(members.storage_hint, StorageHint::Local);
    }

    #[test]
    fn schema_key_is_ignored() {
        let defs = parse_vars_json(VARS_JSON).unwrap();
        assert!(!defs.contains_key("$schema"));
    }

    #[test]
    fn invalid_json_errors() {
        assert!(parse_vars_json("not json").is_err());
    }

    #[test]
    fn invalid_var_def_errors() {
        let bad = r#"{"myvar": {"type": 999}}"#;
        assert!(parse_vars_json(bad).is_err());
    }

    #[test]
    fn storage_hint_project_parsed() {
        let json = r#"{"myvar": {"storage-hint": "project"}}"#;
        let defs = parse_vars_json(json).unwrap();
        assert_eq!(defs["myvar"].storage_hint, StorageHint::Project);
    }

    #[test]
    fn storage_hint_local_parsed() {
        let json = r#"{"myvar": {"storage-hint": "local"}}"#;
        let defs = parse_vars_json(json).unwrap();
        assert_eq!(defs["myvar"].storage_hint, StorageHint::Local);
    }

    #[test]
    fn defaults_apply_for_missing_fields() {
        let minimal = r#"{"myvar": {}}"#;
        let defs = parse_vars_json(minimal).unwrap();
        let def = defs.get("myvar").unwrap();
        assert_eq!(def.var_type, VarType::String);
        assert_eq!(def.storage_hint, StorageHint::Global);
        assert!(def.default.is_none());
    }

    #[test]
    fn warn_invalid_enum_vars_no_panic_for_valid_value() {
        let json = r#"{"style": {"type": "enum", "values": ["a", "b"]}}"#;
        let defs = parse_vars_json(json).unwrap();
        let mut state = HashMap::new();
        state.insert("style".to_string(), Value::String("a".into()));
        // Should not panic; valid values produce no warning.
        warn_invalid_enum_vars("test-skill", &defs, &state);
    }

    #[test]
    fn warn_invalid_enum_vars_no_panic_for_invalid_value() {
        let json = r#"{"style": {"type": "enum", "values": ["a", "b"]}}"#;
        let defs = parse_vars_json(json).unwrap();
        let mut state = HashMap::new();
        state.insert("style".to_string(), Value::String("invalid".into()));
        // Should not panic; the compiler still compiles, just writes to stderr.
        warn_invalid_enum_vars("test-skill", &defs, &state);
    }
}
