use std::path::Path;

use runtime::{
    get_config, list_models, list_providers, read_log,
    workspace::{
        get_active_workspace_session as runtime_get_active_workspace_session,
        get_workspace as runtime_get_workspace,
        get_workspace_provider_matrix as runtime_get_workspace_provider_matrix,
        list_workspace_sessions as runtime_list_workspace_sessions,
        list_workspaces as runtime_list_workspaces,
    },
};
use runtime::{get_effective_skill, list_effective_skills};

use crate::resources::render_events_resource;

/// Resolve a `ship://` URI to its text content, or `None` if not found.
pub async fn resolve_resource_uri(
    uri: &str,
    dir: &Path,
    project_info: impl std::future::Future<Output = String>,
) -> Option<String> {
    if uri == "ship://project_info" {
        return Some(project_info.await);
    }
    if uri == "ship://specs" {
        return resolve_specs_list(dir);
    }
    if let Some(id) = uri.strip_prefix("ship://specs/") {
        let spec_path = runtime::project::specs_dir(dir).join(format!("{id}.md"));
        return std::fs::read_to_string(&spec_path).ok();
    }
    if uri == "ship://adrs" {
        return resolve_adr_list();
    }
    if let Some(id) = uri.strip_prefix("ship://adrs/") {
        return runtime::db::adrs::get_adr(id).ok().flatten().map(|a| {
            format!(
                "Title: {}\nStatus: {}\nDate: {}\n\n## Context\n\n{}\n\n## Decision\n\n{}",
                a.title, a.status, a.date, a.context, a.decision
            )
        });
    }
    if uri == "ship://skills" {
        let entries = list_effective_skills(dir).ok()?;
        if entries.is_empty() {
            return Some("No skills found.".to_string());
        }
        let mut out = String::from("Skills:\n");
        for s in &entries {
            out.push_str(&format!("- {} ({})\n", s.id, s.name));
        }
        return Some(out);
    }
    if let Some(id) = uri.strip_prefix("ship://skills/") {
        return get_effective_skill(dir, id).ok().map(|s| s.content);
    }
    if uri == "ship://log" {
        return match read_log(dir) {
            Ok(content) if content.trim().is_empty() || content.trim() == "# Project Log" => {
                Some("No log entries yet.".to_string())
            }
            Ok(content) => Some(content),
            Err(err) => Some(format!("Error: {}", err)),
        };
    }
    if uri == "ship://events" {
        return render_events_resource(dir, 100);
    }
    if let Some(limit_str) = uri.strip_prefix("ship://events/") {
        let Ok(limit) = limit_str.parse::<usize>() else {
            return Some(format!("Error: invalid limit '{}'", limit_str));
        };
        return render_events_resource(dir, limit);
    }
    if let Some(result) = resolve_workspace_uri(uri, dir) {
        return Some(result);
    }
    None
}

fn resolve_specs_list(dir: &Path) -> Option<String> {
    let specs_path = runtime::project::specs_dir(dir);
    match std::fs::read_dir(&specs_path) {
        Ok(entries) => {
            let mut specs: Vec<String> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("md"))
                .filter_map(|e| {
                    e.path()
                        .file_stem()
                        .and_then(|s| s.to_str().map(String::from))
                })
                .collect();
            if specs.is_empty() {
                return Some("No specs found.".to_string());
            }
            specs.sort();
            let mut out = String::from("Specs:\n");
            for s in &specs {
                out.push_str(&format!("- {s}\n"));
            }
            Some(out)
        }
        Err(_) => Some("No specs found.".to_string()),
    }
}

fn resolve_adr_list() -> Option<String> {
    match runtime::db::adrs::list_adrs() {
        Ok(adrs) if adrs.is_empty() => Some("No ADRs found.".to_string()),
        Ok(adrs) => {
            let mut out = String::from("ADRs:\n");
            for a in &adrs {
                out.push_str(&format!("- {} [{}] {}\n", a.id, a.status, a.title));
            }
            Some(out)
        }
        Err(_) => Some("No ADRs found.".to_string()),
    }
}

fn resolve_workspace_uri(uri: &str, dir: &Path) -> Option<String> {
    if uri == "ship://workspaces" {
        return match runtime_list_workspaces(dir) {
            Ok(workspaces) => serde_json::to_string_pretty(&workspaces).ok(),
            Err(err) => Some(format!("Error: {}", err)),
        };
    }
    if let Some(rest) = uri.strip_prefix("ship://workspaces/")
        && let Some(branch) = rest.strip_suffix("/provider-matrix")
    {
        return match runtime_get_workspace_provider_matrix(dir, branch, None) {
            Ok(matrix) => serde_json::to_string_pretty(&matrix).ok(),
            Err(err) => Some(format!("Error: {}", err)),
        };
    }
    if let Some(rest) = uri.strip_prefix("ship://workspaces/")
        && let Some(branch) = rest.strip_suffix("/session")
    {
        return match runtime_get_active_workspace_session(dir, branch) {
            Ok(Some(session)) => serde_json::to_string_pretty(&session).ok(),
            Ok(None) => Some(format!("No active workspace session for '{}'", branch)),
            Err(err) => Some(format!("Error: {}", err)),
        };
    }
    if let Some(branch) = uri.strip_prefix("ship://workspaces/") {
        return match runtime_get_workspace(dir, branch) {
            Ok(Some(workspace)) => serde_json::to_string_pretty(&workspace).ok(),
            Ok(None) => Some(format!("Workspace '{}' not found", branch)),
            Err(err) => Some(format!("Error: {}", err)),
        };
    }
    if uri == "ship://sessions" {
        return match runtime_list_workspace_sessions(dir, None, 50) {
            Ok(sessions) => serde_json::to_string_pretty(&sessions).ok(),
            Err(err) => Some(format!("Error: {}", err)),
        };
    }
    if let Some(workspace) = uri.strip_prefix("ship://sessions/") {
        return match runtime_list_workspace_sessions(dir, Some(workspace), 50) {
            Ok(sessions) => serde_json::to_string_pretty(&sessions).ok(),
            Err(err) => Some(format!("Error: {}", err)),
        };
    }
    if uri == "ship://modes" {
        return match get_config(Some(dir.to_path_buf())) {
            Ok(config) => {
                let payload = serde_json::json!({
                    "active_mode": config.active_agent,
                    "modes": config.modes,
                });
                serde_json::to_string_pretty(&payload).ok()
            }
            Err(err) => Some(format!("Error: {}", err)),
        };
    }
    if uri == "ship://providers" {
        return match list_providers(dir) {
            Ok(providers) => serde_json::to_string_pretty(&providers).ok(),
            Err(err) => Some(format!("Error: {}", err)),
        };
    }
    if let Some(rest) = uri.strip_prefix("ship://providers/")
        && let Some(provider_id) = rest.strip_suffix("/models")
    {
        return match list_models(provider_id) {
            Ok(models) => serde_json::to_string_pretty(&models).ok(),
            Err(err) => Some(format!("Error: {}", err)),
        };
    }
    None
}

