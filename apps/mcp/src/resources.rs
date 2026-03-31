use std::path::Path;

use rmcp::model::{AnnotateAble, RawResource, RawResourceTemplate};
use runtime::read_recent_events;

pub fn static_resource_list() -> Vec<rmcp::model::Annotated<RawResource>> {
    vec![
        RawResource::new("ship://project_info", "Project Info").no_annotation(),
        RawResource::new("ship://specs", "Specs").no_annotation(),
        RawResource::new("ship://adrs", "ADRs").no_annotation(),
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
        tmpl(
            "ship://session/{path}",
            "Session File",
            "application/octet-stream",
        ),
        tmpl("ship://specs/{id}", "Spec", "text/markdown"),
        tmpl("ship://adrs/{id}", "ADR", "text/markdown"),
        tmpl(
            "ship://workspaces/{branch}",
            "Workspace",
            "application/json",
        ),
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
        tmpl(
            "ship://sessions/{workspace}",
            "Workspace Sessions",
            "application/json",
        ),
        tmpl(
            "ship://providers/{id}/models",
            "Provider Models",
            "application/json",
        ),
        tmpl("ship://events/{limit}", "Recent Events", "text/plain"),
        tmpl("ship://skills/{id}", "Skill", "text/markdown"),
    ]
}

pub fn render_events_resource(project_dir: &Path, limit: usize) -> Option<String> {
    match read_recent_events(project_dir, limit) {
        Ok(events) => {
            if events.is_empty() {
                return Some("No events found.".to_string());
            }
            let mut out = String::from("Events:\n");
            for e in events {
                out.push_str(&format!(
                    "- {} {} [{}] {} {}\n",
                    e.id,
                    e.created_at.format("%Y-%m-%d %H:%M:%S"),
                    e.actor,
                    e.event_type,
                    e.entity_id,
                ));
            }
            Some(out)
        }
        Err(err) => Some(format!("Error: {}", err)),
    }
}
