//! Agent and skill config endpoints — serve .ship/agents/ and .ship/skills/ data.

use axum::{Json, extract::Path, http::StatusCode};

use crate::rest_api::MeshResponse;

use super::{err_response, ok_response, resolve_worktree};

/// Strip JSONC comments (// and /* */) and trailing commas to produce valid JSON.
fn strip_jsonc_comments(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_block_comment = false;
    let mut in_string = false;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if in_block_comment {
            if c == '*' && chars.peek() == Some(&'/') {
                chars.next();
                in_block_comment = false;
            }
            continue;
        }

        if in_string {
            out.push(c);
            if c == '\\' {
                if let Some(&next) = chars.peek() {
                    out.push(next);
                    chars.next();
                }
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }

        if c == '"' {
            in_string = true;
            out.push(c);
            continue;
        }

        if c == '/' {
            match chars.peek() {
                Some(&'/') => {
                    // Line comment — skip to end of line
                    for rest in chars.by_ref() {
                        if rest == '\n' {
                            out.push('\n');
                            break;
                        }
                    }
                    continue;
                }
                Some(&'*') => {
                    chars.next();
                    in_block_comment = true;
                    continue;
                }
                _ => {}
            }
        }

        out.push(c);
    }

    // Strip trailing commas before } or ] (JSONC allows them, JSON does not)
    strip_trailing_commas(&out)
}

fn strip_trailing_commas(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut result = String::with_capacity(input.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b',' {
            // Look ahead past whitespace for } or ]
            let mut j = i + 1;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < bytes.len() && (bytes[j] == b'}' || bytes[j] == b']') {
                // Skip the comma, keep the whitespace
                i += 1;
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

/// GET /api/workspaces/{id}/agents
pub async fn list_agents(
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let worktree = resolve_worktree(&id)?;
    let agents_dir = worktree.join(".ship").join("agents");

    if !agents_dir.exists() {
        return Ok(ok_response(serde_json::json!({ "agents": [] })));
    }

    let entries = std::fs::read_dir(&agents_dir).map_err(|e| {
        err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("failed to read agents dir: {e}"),
        )
    })?;

    let mut agents = Vec::new();
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str());
        if !path.is_file() || !matches!(ext, Some("jsonc" | "json")) {
            continue;
        }

        let id_str = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let raw = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let json_str = strip_jsonc_comments(&raw);
        let parsed: serde_json::Value = match serde_json::from_str(&json_str) {
            Ok(v) => v,
            Err(_) => continue,
        };

        // Agent config may nest under "agent" key or be flat
        let agent_obj = parsed.get("agent").unwrap_or(&parsed);
        agents.push(serde_json::json!({
            "id": agent_obj.get("id").and_then(|v| v.as_str()).unwrap_or(&id_str),
            "name": agent_obj.get("name").and_then(|v| v.as_str()).unwrap_or(&id_str),
            "description": agent_obj.get("description").and_then(|v| v.as_str()),
            "skills": parsed.get("skills").cloned().unwrap_or(serde_json::json!([])),
            "providers": agent_obj.get("providers").cloned().unwrap_or(serde_json::json!([])),
        }));
    }

    Ok(ok_response(serde_json::json!({ "agents": agents })))
}

/// GET /api/workspaces/{id}/skills
pub async fn list_skills(
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let worktree = resolve_worktree(&id)?;
    let skills_dir = worktree.join(".ship").join("skills");

    if !skills_dir.exists() {
        return Ok(ok_response(serde_json::json!({ "skills": [] })));
    }

    let entries = std::fs::read_dir(&skills_dir).map_err(|e| {
        err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("failed to read skills dir: {e}"),
        )
    })?;

    let mut skills = Vec::new();
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let dir_name = entry.file_name().to_string_lossy().to_string();
        let mut name = dir_name.clone();
        let mut stable_id: Option<String> = None;
        let mut description: Option<String> = None;

        // Try to read SKILL.md frontmatter for metadata
        let skill_md = path.join("SKILL.md");
        if skill_md.is_file() {
            if let Ok(content) = std::fs::read_to_string(&skill_md) {
                if content.starts_with("---") {
                    if let Some(end) = content[3..].find("---") {
                        let frontmatter = &content[3..3 + end];
                        for line in frontmatter.lines() {
                            let line = line.trim();
                            if let Some(val) = line.strip_prefix("name:") {
                                name = val.trim().trim_matches('"').to_string();
                            } else if let Some(val) = line.strip_prefix("stable_id:") {
                                stable_id = Some(val.trim().trim_matches('"').to_string());
                            } else if let Some(val) = line.strip_prefix("description:") {
                                description = Some(val.trim().trim_matches('"').to_string());
                            }
                        }
                    }
                }
            }
        }

        // List files in the skill directory
        let mut files = Vec::new();
        if let Ok(dir_entries) = std::fs::read_dir(&path) {
            for file_entry in dir_entries.flatten() {
                if file_entry.path().is_file() {
                    files.push(file_entry.file_name().to_string_lossy().to_string());
                }
            }
        }

        skills.push(serde_json::json!({
            "id": dir_name,
            "name": name,
            "stable_id": stable_id,
            "description": description,
            "files": files,
        }));
    }

    Ok(ok_response(serde_json::json!({ "skills": skills })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_line_comments() {
        let input = r#"{
    // This is a comment
    "name": "test"
}"#;
        let result = strip_jsonc_comments(input);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["name"], "test");
    }

    #[test]
    fn strip_block_comments() {
        let input = r#"{
    /* block comment */
    "name": "test"
}"#;
        let result = strip_jsonc_comments(input);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["name"], "test");
    }

    #[test]
    fn preserve_strings_with_slashes() {
        let input = r#"{"url": "https://example.com"}"#;
        let result = strip_jsonc_comments(input);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["url"], "https://example.com");
    }
}
