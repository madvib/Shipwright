use std::path::Path;

use rmcp::model::{AnnotateAble, RawResource, RawResourceTemplate};
use runtime::{
    get_config, list_events_since, list_models, list_providers, read_log,
    workspace::{
        get_active_workspace_session as runtime_get_active_workspace_session,
        get_workspace as runtime_get_workspace,
        get_workspace_provider_matrix as runtime_get_workspace_provider_matrix,
        list_workspace_sessions as runtime_list_workspace_sessions,
        list_workspaces as runtime_list_workspaces,
    },
};
use runtime::{get_effective_skill, list_effective_skills};

pub fn static_resource_list() -> Vec<rmcp::model::Annotated<RawResource>> {
    vec![
        RawResource::new("ship://project_info", "Project Info").no_annotation(),
        RawResource::new("ship://specs", "Specs").no_annotation(),
        RawResource::new("ship://adrs", "ADRs").no_annotation(),
        RawResource::new("ship://notes", "Project Notes").no_annotation(),
        RawResource::new("ship://skills", "Skills").no_annotation(),
        RawResource::new("ship://workspaces", "Workspaces").no_annotation(),
        RawResource::new("ship://sessions", "Workspace Sessions").no_annotation(),
        RawResource::new("ship://modes", "Modes").no_annotation(),
        RawResource::new("ship://providers", "Providers").no_annotation(),
        RawResource::new("ship://log", "Project Log").no_annotation(),
        RawResource::new("ship://events", "Event Stream").no_annotation(),
    ]
}

fn tmpl(uri: &str, name: &str, mime: &str) -> rmcp::model::Annotated<RawResourceTemplate> {
    RawResourceTemplate {
        uri_template: uri.to_string(),
        name: name.to_string(),
        title: None,
        description: None,
        mime_type: Some(mime.to_string()),
        icons: None,
    }
    .no_annotation()
}

pub fn static_resource_template_list() -> Vec<rmcp::model::Annotated<RawResourceTemplate>> {
    vec![
        tmpl("ship://specs/{id}", "Spec", "text/markdown"),
        tmpl("ship://adrs/{id}", "ADR", "text/markdown"),
        tmpl("ship://notes/{id}", "Note", "text/markdown"),
        tmpl("ship://workspaces/{branch}", "Workspace", "application/json"),
        tmpl(
            "ship://workspaces/{branch}/provider-matrix",
            "Workspace Provider Matrix",
            "application/json",
        ),
        tmpl(
            "ship://workspaces/{branch}/session",
            "Workspace Active Session",
            "application/json",
        ),
        tmpl("ship://sessions/{workspace}", "Workspace Sessions", "application/json"),
        tmpl("ship://providers/{id}/models", "Provider Models", "application/json"),
        tmpl("ship://events/{since}", "Event Stream Since Seq", "text/plain"),
        tmpl("ship://skills/{id}", "Skill", "text/markdown"),
    ]
}

pub fn render_events_resource(project_dir: &Path, since: u64, limit: usize) -> Option<String> {
    match list_events_since(project_dir, since, Some(limit)) {
        Ok(events) => {
            if events.is_empty() {
                return Some("No events found.".to_string());
            }
            let mut out = String::from("Events:\n");
            for e in events {
                let details = e
                    .details
                    .as_ref()
                    .map(|d| format!(" — {}", d))
                    .unwrap_or_default();
                out.push_str(&format!(
                    "- #{} {} [{}] {:?}.{:?} {}{}\n",
                    e.seq,
                    e.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    e.actor,
                    e.entity,
                    e.action,
                    e.subject,
                    details
                ));
            }
            Some(out)
        }
        Err(err) => Some(format!("Error: {}", err)),
    }
}

/// Resolve a `ship://` URI to its text content, or `None` if not found.
pub async fn resolve_resource_uri(
    uri: &str,
    dir: &Path,
    project_info: impl std::future::Future<Output = String>,
) -> Option<String> {
    if uri == "ship://project_info" {
        return Some(project_info.await);
    }
    if uri == "ship://adrs" {
        let ship_dir = dir.join(".ship");
        return match runtime::db::adrs::list_adrs(&ship_dir) {
            Ok(adrs) if adrs.is_empty() => Some("No ADRs found.".to_string()),
            Ok(adrs) => {
                let mut out = String::from("ADRs:\n");
                for a in &adrs {
                    out.push_str(&format!("- {} [{}] {}\n", a.id, a.status, a.title));
                }
                Some(out)
            }
            Err(_) => Some("No ADRs found.".to_string()),
        };
    }
    if let Some(id) = uri.strip_prefix("ship://adrs/") {
        let ship_dir = dir.join(".ship");
        return runtime::db::adrs::get_adr(&ship_dir, id)
            .ok()
            .flatten()
            .map(|a| {
                format!(
                    "Title: {}\nStatus: {}\nDate: {}\n\n## Context\n\n{}\n\n## Decision\n\n{}",
                    a.title, a.status, a.date, a.context, a.decision
                )
            });
    }
    if uri == "ship://notes" {
        let ship_dir = dir.join(".ship");
        return match runtime::db::notes::list_notes(&ship_dir, None) {
            Ok(notes) if notes.is_empty() => Some("No notes found.".to_string()),
            Ok(notes) => {
                let mut out = String::from("Notes:\n");
                for n in &notes {
                    out.push_str(&format!("- {} {}\n", n.id, n.title));
                }
                Some(out)
            }
            Err(_) => Some("No notes found.".to_string()),
        };
    }
    if let Some(id) = uri.strip_prefix("ship://notes/") {
        let ship_dir = dir.join(".ship");
        return runtime::db::notes::get_note(&ship_dir, id)
            .ok()
            .flatten()
            .map(|n| format!("Title: {}\n\n{}", n.title, n.content));
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
            Ok(content)
                if content.trim().is_empty() || content.trim() == "# Project Log" =>
            {
                Some("No log entries yet.".to_string())
            }
            Ok(content) => Some(content),
            Err(err) => Some(format!("Error: {}", err)),
        };
    }
    if uri == "ship://events" {
        return render_events_resource(dir, 0, 100);
    }
    if let Some(since) = uri.strip_prefix("ship://events/") {
        let Ok(since) = since.parse::<u64>() else {
            return Some(format!("Error: invalid event sequence '{}'", since));
        };
        return render_events_resource(dir, since, 100);
    }
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
