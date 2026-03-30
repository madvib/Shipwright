//! Resolve Ship built-in event definitions.
//!
//! Ship platform events (e.g. `annotation`, `feedback`) are defined in
//! `ship-events.json`. This module provides a lookup utility for the runtime
//! (e.g. EventRelay) to resolve event names to their full definitions.

use serde_json::{Map, Value, json};

/// Ship built-in event definitions, embedded at compile time.
const SHIP_EVENTS_RAW: &str = include_str!("../../../../schemas/ship-events.json");

/// Resolve a list of Ship built-in event names into their full definitions.
///
/// - `names`: event names to resolve (e.g. `["annotation", "feedback"]`).
///
/// Returns a JSON array of fully-resolved event objects, each with an `"id"` field.
pub fn resolve_builtin_events(names: &[&str]) -> Result<Value, String> {
    let builtins = load_builtins()?;
    let mut result: Vec<Value> = Vec::new();

    for name in names {
        let def = builtins
            .get(*name)
            .ok_or_else(|| format!("unknown ship event: {name}"))?;
        let mut entry = def.clone();
        if let Some(o) = entry.as_object_mut() {
            o.insert("id".into(), json!(name));
        }
        result.push(entry);
    }

    Ok(Value::Array(result))
}

/// Look up a single Ship built-in event definition by name.
pub fn get_builtin_event(name: &str) -> Result<Value, String> {
    let builtins = load_builtins()?;
    let def = builtins
        .get(name)
        .ok_or_else(|| format!("unknown ship event: {name}"))?;
    let mut entry = def.clone();
    if let Some(o) = entry.as_object_mut() {
        o.insert("id".into(), json!(name));
    }
    Ok(entry)
}

/// Return all Ship built-in event names.
pub fn builtin_event_names() -> Result<Vec<String>, String> {
    let builtins = load_builtins()?;
    Ok(builtins.keys().cloned().collect())
}

fn load_builtins() -> Result<Map<String, Value>, String> {
    let parsed: Value =
        serde_json::from_str(SHIP_EVENTS_RAW).map_err(|e| format!("bad ship-events.json: {e}"))?;
    parsed
        .get("events")
        .and_then(|v| v.as_object())
        .cloned()
        .ok_or_else(|| "ship-events.json missing \"events\" object".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_known_events() {
        let result = resolve_builtin_events(&["annotation", "feedback"]).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        for entry in arr {
            assert!(entry.get("direction").is_some());
            assert!(entry.get("label").is_some());
        }
        assert_eq!(arr[0]["id"], "annotation");
        assert_eq!(arr[1]["id"], "feedback");
    }

    #[test]
    fn unknown_event_errors() {
        let err = resolve_builtin_events(&["nonexistent"]).unwrap_err();
        assert!(err.contains("nonexistent"));
    }

    #[test]
    fn empty_list_is_ok() {
        let result = resolve_builtin_events(&[]).unwrap();
        assert_eq!(result.as_array().unwrap().len(), 0);
    }

    #[test]
    fn get_single_event() {
        let event = get_builtin_event("selection").unwrap();
        assert_eq!(event["id"], "selection");
        assert!(event.get("direction").is_some());
    }

    #[test]
    fn builtin_names_has_all_five() {
        let names = builtin_event_names().unwrap();
        assert_eq!(names.len(), 5);
        for expected in &["annotation", "feedback", "selection", "artifact_created", "artifact_deleted"] {
            assert!(names.contains(&expected.to_string()), "missing: {expected}");
        }
    }
}
