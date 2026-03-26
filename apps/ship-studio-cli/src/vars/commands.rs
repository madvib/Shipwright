//! CLI command handlers for `ship vars`.

use anyhow::{Context, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};

use super::schema::{VarType, load_vars_json};
use super::state::{
    append_to_array, create_default_state, project_state_path, read_skill_state, user_state_path,
    validate_skill_id, write_skill_state,
};

// ── Helper ────────────────────────────────────────────────────────────────────

fn find_vars_json(ship_dir: &Path, skill_id: &str) -> Result<PathBuf> {
    validate_skill_id(skill_id)?;
    let path = ship_dir.join("skills").join(skill_id).join("vars.json");
    if path.exists() {
        return Ok(path);
    }
    anyhow::bail!(
        "skill '{}' has no vars.json (expected at {})",
        skill_id,
        path.display()
    )
}

// ── Command handlers ──────────────────────────────────────────────────────────

/// `ship vars set <skill-id> <key> <value>`
pub fn run_vars_set(ship_dir: &Path, skill_id: &str, key: &str, value_str: &str) -> Result<()> {
    let var_defs = load_vars_json(&find_vars_json(ship_dir, skill_id)?)?;

    let def = var_defs.get(key).ok_or_else(|| {
        let mut known: Vec<&str> = var_defs.keys().map(|s| s.as_str()).collect();
        known.sort();
        anyhow::anyhow!("unknown variable '{}'. Known vars: {}", key, known.join(", "))
    })?;

    let value = match def.var_type {
        VarType::Bool => match value_str {
            "true" | "1" | "yes" => Value::Bool(true),
            "false" | "0" | "no" => Value::Bool(false),
            _ => anyhow::bail!("'{}' is a bool var; use true or false", key),
        },
        VarType::Enum => {
            if !def.values.is_empty() && !def.values.contains(&value_str.to_string()) {
                anyhow::bail!(
                    "invalid value '{}' for '{}'. Allowed: {}",
                    value_str,
                    key,
                    def.values.join(", ")
                );
            }
            Value::String(value_str.to_string())
        }
        _ => Value::String(value_str.to_string()),
    };

    write_skill_state(skill_id, key, &value, &def.storage_hint, ship_dir)?;
    println!("set {}.{} = {}", skill_id, key, value_str);
    Ok(())
}

/// `ship vars get <skill-id> [key]`
pub fn run_vars_get(ship_dir: &Path, skill_id: &str, key: Option<&str>) -> Result<()> {
    let var_defs = load_vars_json(&find_vars_json(ship_dir, skill_id)?)?;
    let state = read_skill_state(skill_id, ship_dir, &var_defs);

    match key {
        Some(k) => {
            let val = state
                .get(k)
                .ok_or_else(|| anyhow::anyhow!("unknown variable '{}'", k))?;
            println!("{}", serde_json::to_string_pretty(val)?);
        }
        None => {
            let mut keys: Vec<&String> = state.keys().collect();
            keys.sort();
            for k in keys {
                let def = var_defs.get(k);
                let label = def.and_then(|d| d.label.as_deref()).unwrap_or(k.as_str());
                println!("{} ({}): {}", k, label, serde_json::to_string(&state[k])?);
            }
        }
    }
    Ok(())
}

/// `ship vars edit <skill-id>`
pub fn run_vars_edit(ship_dir: &Path, skill_id: &str, editor: Option<&str>) -> Result<()> {
    let var_defs = load_vars_json(&find_vars_json(ship_dir, skill_id)?)?;

    let state_path = project_state_path(skill_id, ship_dir);
    if !state_path.exists() {
        create_default_state(skill_id, &var_defs, ship_dir)?;
    }

    let editor = editor
        .map(str::to_string)
        .or_else(|| std::env::var("EDITOR").ok())
        .unwrap_or_else(|| "vi".to_string());

    std::process::Command::new(&editor)
        .arg(&state_path)
        .status()
        .with_context(|| format!("opening {} with {}", state_path.display(), editor))?;

    Ok(())
}

/// `ship vars append <skill-id> <key> <json>`
pub fn run_vars_append(
    ship_dir: &Path,
    skill_id: &str,
    key: &str,
    json_str: &str,
) -> Result<()> {
    let var_defs = load_vars_json(&find_vars_json(ship_dir, skill_id)?)?;

    let def = var_defs
        .get(key)
        .ok_or_else(|| anyhow::anyhow!("unknown variable '{}'", key))?;

    if def.var_type != VarType::Array {
        anyhow::bail!("'{}' is not an array var (type: {:?})", key, def.var_type);
    }

    let element: Value =
        serde_json::from_str(json_str).with_context(|| format!("invalid JSON: {}", json_str))?;

    append_to_array(skill_id, key, &element, &def.storage_hint, ship_dir)?;
    println!("appended to {}.{}", skill_id, key);
    Ok(())
}

/// `ship vars reset <skill-id>`
pub fn run_vars_reset(ship_dir: &Path, skill_id: &str) -> Result<()> {
    let proj = project_state_path(skill_id, ship_dir);
    let user = user_state_path(skill_id);

    let mut removed = false;
    if proj.exists() {
        std::fs::remove_file(&proj)?;
        removed = true;
    }
    if user.exists() {
        std::fs::remove_file(&user)?;
        removed = true;
    }

    if removed {
        println!("reset state for '{}' — next compile uses defaults", skill_id);
    } else {
        println!("no state found for '{}' (already at defaults)", skill_id);
    }
    Ok(())
}
