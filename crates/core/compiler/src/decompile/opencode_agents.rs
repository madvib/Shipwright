//! OpenCode agent decompilation — parse `"agent"` entries from `opencode.json`
//! into Ship [`AgentProfile`] structs.

use std::collections::HashMap;

use serde_json::Value as Json;

use crate::types::{
    AgentProfile, McpRefs, PluginRefs, ProfileMeta, ProfilePermissions, ProfileRules, SkillRefs,
};

/// Known agent-level keys that map to structured Ship fields (not passthrough).
const KNOWN_AGENT_KEYS: &[&str] = &["description", "model", "prompt", "permission"];

/// Parse an OpenCode agent entry into an [`AgentProfile`].
pub(super) fn parse_opencode_agent(name: &str, entry: &Json) -> Option<AgentProfile> {
    let obj = entry.as_object()?;

    let description = obj
        .get("description")
        .and_then(|v| v.as_str())
        .map(String::from);
    let prompt = obj
        .get("prompt")
        .and_then(|v| v.as_str())
        .map(String::from);
    let permissions = obj
        .get("permission")
        .map(parse_agent_permissions)
        .unwrap_or_default();

    // Collect unknown fields + model into provider_settings.opencode
    let mut oc_settings = toml::map::Map::new();
    if let Some(Json::String(m)) = obj.get("model") {
        oc_settings.insert("model".to_string(), toml::Value::String(m.clone()));
    }
    for (k, v) in obj {
        if !KNOWN_AGENT_KEYS.contains(&k.as_str())
            && let Some(tv) = json_to_toml(v)
        {
            oc_settings.insert(k.clone(), tv);
        }
    }

    let mut provider_settings = HashMap::new();
    if !oc_settings.is_empty() {
        provider_settings.insert("opencode".to_string(), toml::Value::Table(oc_settings));
    }

    Some(AgentProfile {
        profile: ProfileMeta {
            id: name.to_string(),
            name: name.to_string(),
            version: None,
            description,
            providers: vec![],
        },
        skills: SkillRefs::default(),
        mcp: McpRefs::default(),
        plugins: PluginRefs::default(),
        permissions,
        rules: ProfileRules { inline: prompt },
        provider_settings,
    })
}

/// Reverse-translate OpenCode permission object into Ship's [`ProfilePermissions`].
///
/// OpenCode keys map to Ship tool names:
///   bash -> Bash, edit -> Edit/Write, read -> Read, glob -> Glob,
///   grep -> Grep, list -> LS, webfetch -> WebFetch, websearch -> WebSearch.
///
/// Values: "allow" -> tools_allow, "deny" -> tools_deny, "ask" -> tools_ask.
/// Granular objects like `{ "git *": "allow" }` -> `Bash(git *)` in the appropriate list.
fn parse_agent_permissions(value: &Json) -> ProfilePermissions {
    let mut perms = ProfilePermissions::default();
    let obj = match value.as_object() {
        Some(o) => o,
        None => return perms,
    };

    for (key, val) in obj {
        let tool_names = opencode_key_to_ship_tools(key);
        if tool_names.is_empty() {
            continue;
        }
        match val {
            Json::String(action) => {
                for tool in &tool_names {
                    push_permission(&mut perms, tool, action);
                }
            }
            Json::Object(globs) => {
                // Granular: { "git *": "allow", "rm *": "deny" }
                for (glob, action_val) in globs {
                    if let Some(action) = action_val.as_str() {
                        let base = &tool_names[0];
                        let pattern = format!("{base}({glob})");
                        push_permission(&mut perms, &pattern, action);
                    }
                }
            }
            _ => {}
        }
    }

    perms
}

/// Map an OpenCode permission key to Ship tool name(s).
fn opencode_key_to_ship_tools(key: &str) -> Vec<String> {
    match key {
        "bash" => vec!["Bash".to_string()],
        "edit" => vec!["Edit".to_string(), "Write".to_string()],
        "read" => vec!["Read".to_string()],
        "glob" => vec!["Glob".to_string()],
        "grep" => vec!["Grep".to_string()],
        "list" => vec!["LS".to_string()],
        "webfetch" => vec!["WebFetch".to_string()],
        "websearch" => vec!["WebSearch".to_string()],
        _ => vec![],
    }
}

/// Push a tool pattern into the appropriate permission list.
fn push_permission(perms: &mut ProfilePermissions, tool: &str, action: &str) {
    match action {
        "allow" => perms.tools_allow.push(tool.to_string()),
        "deny" => perms.tools_deny.push(tool.to_string()),
        "ask" => perms.tools_ask.push(tool.to_string()),
        _ => {}
    }
}

/// Convert a serde_json::Value to a toml::Value for provider_settings passthrough.
fn json_to_toml(json: &Json) -> Option<toml::Value> {
    match json {
        Json::String(s) => Some(toml::Value::String(s.clone())),
        Json::Number(n) => {
            if let Some(i) = n.as_i64() {
                Some(toml::Value::Integer(i))
            } else {
                n.as_f64().map(toml::Value::Float)
            }
        }
        Json::Bool(b) => Some(toml::Value::Boolean(*b)),
        Json::Array(arr) => {
            let items: Vec<toml::Value> = arr.iter().filter_map(json_to_toml).collect();
            Some(toml::Value::Array(items))
        }
        Json::Object(obj) => {
            let mut table = toml::map::Map::new();
            for (k, v) in obj {
                if let Some(tv) = json_to_toml(v) {
                    table.insert(k.clone(), tv);
                }
            }
            Some(toml::Value::Table(table))
        }
        Json::Null => None,
    }
}
