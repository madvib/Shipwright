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

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VarScope {
    Project,
    User,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VarDef {
    #[serde(rename = "type", default)]
    pub var_type: VarType,
    pub default: Option<Value>,
    #[serde(default)]
    pub scope: VarScope,
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

impl Default for VarScope {
    fn default() -> Self {
        VarScope::User
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
            "scope": "user",
            "values": ["conventional", "gitmoji", "angular"],
            "label": "Commit style",
            "description": "Format applied to every commit message"
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
    }"#;

    #[test]
    fn parse_vars_json_basic() {
        let defs = parse_vars_json(VARS_JSON).unwrap();
        assert_eq!(defs.len(), 3);

        let commit = defs.get("commit_style").unwrap();
        assert_eq!(commit.var_type, VarType::Enum);
        assert_eq!(commit.default, Some(Value::String("conventional".into())));
        assert_eq!(commit.scope, VarScope::User);
        assert_eq!(commit.values, ["conventional", "gitmoji", "angular"]);
        assert_eq!(commit.label.as_deref(), Some("Commit style"));

        let verbose = defs.get("verbose_mode").unwrap();
        assert_eq!(verbose.var_type, VarType::Bool);
        assert_eq!(verbose.default, Some(Value::Bool(false)));
        assert_eq!(verbose.scope, VarScope::Project);

        let members = defs.get("team_members").unwrap();
        assert_eq!(members.var_type, VarType::Array);
        assert!(members.default.is_none());
        assert_eq!(members.scope, VarScope::Project);
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
    fn defaults_apply_for_missing_fields() {
        let minimal = r#"{"myvar": {}}"#;
        let defs = parse_vars_json(minimal).unwrap();
        let def = defs.get("myvar").unwrap();
        assert_eq!(def.var_type, VarType::String);
        assert_eq!(def.scope, VarScope::User);
        assert!(def.default.is_none());
    }
}
