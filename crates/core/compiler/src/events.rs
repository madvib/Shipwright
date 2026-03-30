//! Expand skill `events.json` into a resolved event list.
//!
//! Ship built-in event refs (e.g. `"ship": ["annotation"]`) are expanded to
//! their full definitions from `ship-events.json`. Custom events are namespaced
//! with the skill's `stable_id`.

use serde_json::{Map, Value, json};

/// Ship built-in event definitions, embedded at compile time.
const SHIP_EVENTS_RAW: &str = include_str!("../../../../schemas/ship-events.json");

/// Expand an `events.json` value into a resolved array of event definitions.
///
/// - `stable_id`: the skill's stable identifier, used to namespace custom events.
/// - `raw`: the parsed content of the skill's `assets/events.json`.
///
/// Returns a JSON array of fully-resolved event objects.
pub fn expand_events(stable_id: &str, raw: &Value) -> Result<Value, String> {
    let obj = raw
        .as_object()
        .ok_or_else(|| "events.json must be a JSON object".to_string())?;

    // Reject unknown top-level keys
    for key in obj.keys() {
        match key.as_str() {
            "$schema" | "ship" | "custom" => {}
            other => return Err(format!("unknown top-level key: {other}")),
        }
    }

    let builtins = load_builtins()?;
    let mut result: Vec<Value> = Vec::new();

    // Expand ship built-in refs
    if let Some(ship_val) = obj.get("ship") {
        let refs = ship_val
            .as_array()
            .ok_or_else(|| "\"ship\" must be an array".to_string())?;
        for r in refs {
            let name = r
                .as_str()
                .ok_or_else(|| "ship event refs must be strings".to_string())?;
            let def = builtins.get(name).ok_or_else(|| {
                format!("unknown ship event: {name}")
            })?;
            let mut entry = def.clone();
            if let Some(o) = entry.as_object_mut() {
                o.insert("id".into(), json!(name));
            }
            result.push(entry);
        }
    }

    // Expand custom events
    if let Some(custom_val) = obj.get("custom") {
        let customs = custom_val
            .as_array()
            .ok_or_else(|| "\"custom\" must be an array".to_string())?;
        for c in customs {
            let co = c
                .as_object()
                .ok_or_else(|| "custom events must be objects".to_string())?;
            let id = co
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "custom event missing \"id\"".to_string())?;

            // Validate id pattern: ^[a-z][a-z0-9_]*$
            if !is_valid_event_id(id) {
                return Err(format!(
                    "invalid custom event id \"{id}\": must match ^[a-z][a-z0-9_]*$"
                ));
            }

            let mut entry = c.clone();
            if let Some(o) = entry.as_object_mut() {
                o.insert("id".into(), json!(format!("{stable_id}.{id}")));
            }
            result.push(entry);
        }
    }

    Ok(Value::Array(result))
}

fn is_valid_event_id(id: &str) -> bool {
    let mut chars = id.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
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
    use serde_json::json;

    #[test]
    fn expand_ship_refs() {
        let input = json!({"ship": ["annotation", "feedback"]});
        let result = expand_events("test-skill", &input).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        // Both should have direction and label keys (from ship-events.json)
        for entry in arr {
            assert!(entry.get("direction").is_some());
            assert!(entry.get("label").is_some());
        }
        assert_eq!(arr[0]["id"], "annotation");
        assert_eq!(arr[1]["id"], "feedback");
    }

    #[test]
    fn unknown_ship_ref_errors() {
        let input = json!({"ship": ["nonexistent"]});
        let err = expand_events("test-skill", &input).unwrap_err();
        assert!(err.contains("nonexistent"), "error should mention the name: {err}");
    }

    #[test]
    fn custom_events_namespaced() {
        let input = json!({
            "custom": [{"id": "node_selected", "direction": "out"}]
        });
        let result = expand_events("visual-brainstorm", &input).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["id"], "visual-brainstorm.node_selected");
    }

    #[test]
    fn invalid_custom_event_id() {
        let input = json!({
            "custom": [{"id": "BadName", "direction": "out"}]
        });
        let err = expand_events("test-skill", &input).unwrap_err();
        assert!(err.contains("BadName"));
    }

    #[test]
    fn unknown_top_level_key_errors() {
        let input = json!({"ship": [], "bogus": true});
        let err = expand_events("test-skill", &input).unwrap_err();
        assert!(err.contains("bogus"));
    }

    #[test]
    fn ship_absent_is_ok() {
        let input = json!({"custom": []});
        let result = expand_events("test-skill", &input).unwrap();
        assert_eq!(result.as_array().unwrap().len(), 0);
    }

    #[test]
    fn custom_absent_is_ok() {
        let input = json!({"ship": ["annotation"]});
        let result = expand_events("test-skill", &input).unwrap();
        assert_eq!(result.as_array().unwrap().len(), 1);
    }

    #[test]
    fn empty_object_is_ok() {
        let input = json!({});
        let result = expand_events("test-skill", &input).unwrap();
        assert_eq!(result.as_array().unwrap().len(), 0);
    }

    #[test]
    fn schema_key_allowed() {
        let input = json!({"$schema": "https://example.com/events.schema.json", "ship": ["selection"]});
        let result = expand_events("test-skill", &input).unwrap();
        assert_eq!(result.as_array().unwrap().len(), 1);
    }
}
