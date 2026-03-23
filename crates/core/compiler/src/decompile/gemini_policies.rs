//! Gemini policy parsing — `.gemini/policies/*.toml` → Ship permissions.

use std::path::Path;

use crate::types::{Permissions, ToolPermissions};

/// Parse all `.toml` policy files from `.gemini/policies/` into Ship permissions.
pub(super) fn parse_gemini_policies(policies_dir: &Path) -> Permissions {
    let mut allow = Vec::new();
    let mut deny = Vec::new();
    let mut ask = Vec::new();

    if let Ok(entries) = std::fs::read_dir(policies_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "toml")
                && let Ok(content) = std::fs::read_to_string(&path)
                && let Ok(table) = content.parse::<toml::Table>()
            {
                parse_policy_file(&table, &mut allow, &mut deny, &mut ask);
            }
        }
    }

    if allow.is_empty() && deny.is_empty() && ask.is_empty() {
        return Permissions::default();
    }

    Permissions {
        tools: ToolPermissions { allow, ask, deny },
        ..Default::default()
    }
}

fn parse_policy_file(
    table: &toml::Table,
    allow: &mut Vec<String>,
    deny: &mut Vec<String>,
    ask: &mut Vec<String>,
) {
    let policies = match table.get("tool_policies").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return,
    };

    for entry in policies {
        let t = match entry.as_table() {
            Some(t) => t,
            None => continue,
        };

        let tool = t.get("tool").and_then(|v| v.as_str()).unwrap_or("");
        let pattern = t.get("pattern").and_then(|v| v.as_str());
        let decision = t.get("decision").and_then(|v| v.as_str()).unwrap_or("");

        let ship_pattern = reverse_gemini_policy(tool, pattern);
        if ship_pattern.is_empty() {
            continue;
        }

        match decision {
            "allow" => allow.push(ship_pattern),
            "deny" => deny.push(ship_pattern),
            "ask_user" => ask.push(ship_pattern),
            _ => {}
        }
    }
}

/// Reverse-map a Gemini policy entry to a Ship permission pattern.
fn reverse_gemini_policy(tool: &str, pattern: Option<&str>) -> String {
    match tool {
        "shell" => match pattern {
            Some(p) => format!("Bash({})", regex_to_glob(p)),
            None => "Bash".to_string(),
        },
        "file_read" => match pattern {
            Some(p) => format!("Read({})", regex_to_glob(p)),
            None => "Read".to_string(),
        },
        "file_write" => match pattern {
            Some(p) => format!("Write({})", regex_to_glob(p)),
            None => "Write".to_string(),
        },
        "web_fetch" => match pattern {
            Some(p) => format!("WebFetch({})", regex_to_glob(p)),
            None => "WebFetch".to_string(),
        },
        "mcp" => match pattern {
            Some(p) => {
                let parts: Vec<&str> = p.splitn(2, '/').collect();
                if parts.len() == 2 {
                    format!(
                        "mcp__{}__{}",
                        regex_to_glob(parts[0]),
                        regex_to_glob(parts[1])
                    )
                } else {
                    String::new()
                }
            }
            None => String::new(),
        },
        _ => String::new(),
    }
}

/// Best-effort regex to glob conversion (reverses glob_to_regex_prefix).
fn regex_to_glob(re: &str) -> String {
    re.replace(".*", "*")
        .replace("\\.", ".")
        .replace("\\+", "+")
        .replace("\\?", "?")
        .replace("\\(", "(")
        .replace("\\)", ")")
        .replace("\\[", "[")
        .replace("\\]", "]")
        .replace("\\{", "{")
        .replace("\\}", "}")
        .replace("\\^", "^")
        .replace("\\$", "$")
        .replace("\\|", "|")
        .replace("\\\\", "\\")
}
