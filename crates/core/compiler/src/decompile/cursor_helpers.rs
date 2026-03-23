//! Cursor decompile helpers — frontmatter parsing, CLI permissions, reverse-mapping.

use std::collections::HashMap;

use serde_json::Value as Json;

use crate::types::{Permissions, ToolPermissions};
use crate::ProjectLibrary;

use super::json_string_array;

/// Parse simple YAML frontmatter from .mdc files.
/// Returns (key-value map, body after frontmatter).
pub(super) fn parse_mdc_frontmatter(content: &str) -> (HashMap<String, String>, &str) {
    let mut map = HashMap::new();

    if !content.starts_with("---") {
        return (map, content);
    }

    let rest = &content[3..];
    let end = match rest.find("\n---") {
        Some(idx) => idx,
        None => return (map, content),
    };

    let fm_block = &rest[..end];
    let body_start = 3 + end + 4; // "---" + fm + "\n---"
    let body = if body_start < content.len() {
        &content[body_start..]
    } else {
        ""
    };

    for line in fm_block.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('-') {
            continue;
        }
        if let Some((key, val)) = line.split_once(':') {
            map.insert(key.trim().to_string(), val.trim().to_string());
        }
    }

    (map, body)
}

// ── cli.json parsing ─────────────────────────────────────────────────────────

/// Known top-level keys in `.cursor/cli.json` that map to structured Ship fields.
const KNOWN_CLI_KEYS: &[&str] = &["version", "permissions"];

/// Parse `.cursor/cli.json` into permissions and provider_defaults.
pub(super) fn parse_cursor_cli(library: &mut ProjectLibrary, json: &Json) {
    let obj = match json.as_object() {
        Some(o) => o,
        None => return,
    };

    if let Some(perms) = obj.get("permissions").and_then(|v| v.as_object()) {
        let allow: Vec<String> = json_string_array(perms.get("allow"))
            .into_iter()
            .filter_map(|p| reverse_cursor_permission(&p))
            .collect();

        let deny: Vec<String> = json_string_array(perms.get("deny"))
            .into_iter()
            .filter_map(|p| reverse_cursor_permission(&p))
            .collect();

        if !allow.is_empty() || !deny.is_empty() {
            library.permissions = Permissions {
                tools: ToolPermissions {
                    allow,
                    deny,
                    ask: vec![],
                },
                ..Default::default()
            };
        }
    }

    let mut extra = serde_json::Map::new();
    for (k, v) in obj {
        if !KNOWN_CLI_KEYS.contains(&k.as_str()) {
            extra.insert(k.clone(), v.clone());
        }
    }
    if !extra.is_empty() {
        library
            .provider_defaults
            .insert("cursor".to_string(), Json::Object(extra));
    }
}

/// Reverse-map a Cursor permission pattern to Ship's internal format.
pub(super) fn reverse_cursor_permission(pattern: &str) -> Option<String> {
    if let Some(inner) = pattern
        .strip_prefix("Shell(")
        .and_then(|s| s.strip_suffix(')'))
    {
        return Some(format!("Bash({inner})"));
    }
    if pattern.starts_with("Read(") {
        return Some(pattern.to_string());
    }
    if pattern.starts_with("Write(") {
        return Some(pattern.to_string());
    }
    if pattern.starts_with("WebFetch(") {
        return Some(pattern.to_string());
    }
    if let Some(inner) = pattern
        .strip_prefix("Mcp(")
        .and_then(|s| s.strip_suffix(')'))
    {
        let parts: Vec<&str> = inner.splitn(2, ':').collect();
        if parts.len() == 2 {
            return Some(format!("mcp__{}__{}", parts[0], parts[1]));
        }
    }
    None
}
