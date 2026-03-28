use std::path::{Path, PathBuf};
use std::sync::Arc;

use runtime::project::{get_active_project_global, get_project_dir, set_active_project_global};
use runtime::project::{get_project_name, list_registered_projects};
use runtime::{
    get_config, read_log, read_recent_events,
    workspace::{
        get_active_workspace_session as runtime_get_active_workspace_session,
        list_workspaces as runtime_list_workspaces,
    },
};
use tokio::sync::Mutex;

/// Set the active project; returns (response_message, resolved_project_root).
pub async fn open_project(
    path: &str,
    active_project: &Arc<Mutex<Option<PathBuf>>>,
) -> (String, Option<PathBuf>) {
    let path = PathBuf::from(path);
    match get_project_dir(Some(path.clone())) {
        Ok(ship_dir) => {
            let resolved = resolve_project_root(ship_dir.clone());
            {
                let mut lock = active_project.lock().await;
                *lock = Some(resolved.clone());
            }
            if let Err(e) = set_active_project_global(ship_dir.clone()) {
                return (
                    format!(
                        "Opened project at {} (warning: failed to persist global active project: {})",
                        ship_dir.display(),
                        e
                    ),
                    Some(resolved),
                );
            }
            (
                format!("Opened project at {}", ship_dir.display()),
                Some(resolved),
            )
        }
        Err(e) => (format!("Error: {}", e), None),
    }
}

/// Resolve the effective project directory (parent of `.ship/`).
pub async fn get_effective_project_dir(
    active_project: &Arc<Mutex<Option<PathBuf>>>,
) -> Result<PathBuf, String> {
    {
        let active = active_project.lock().await;
        if let Some(ref path) = *active {
            return Ok(path.clone());
        }
    }

    if let Ok(dir) = get_project_dir(None) {
        return Ok(resolve_project_root(dir));
    }

    if let Ok(Some(global_active)) = get_active_project_global()
        && let Ok(dir) = get_project_dir(Some(global_active.clone()))
    {
        return Ok(resolve_project_root(dir));
    }

    if let Ok(registry) = list_registered_projects()
        && registry.len() == 1
        && let Ok(dir) = get_project_dir(Some(registry[0].path.clone()))
    {
        return Ok(resolve_project_root(dir));
    }

    get_project_dir(None)
        .map(resolve_project_root)
        .map_err(|e| {
            format!(
                "No active project and auto-detection failed: {}. Checked process cwd, global active project, and registered projects.",
                e
            )
        })
}

fn resolve_project_root(ship_dir: PathBuf) -> PathBuf {
    if ship_dir.file_name().and_then(|n| n.to_str()) == Some(".ship") {
        ship_dir.parent().unwrap_or(&ship_dir).to_path_buf()
    } else {
        ship_dir
    }
}

/// Build the full project context snapshot used by resources.
pub async fn get_project_info(project_dir: &Path) -> String {
    let name = get_project_name(project_dir);
    let config = get_config(Some(project_dir.to_path_buf())).unwrap_or_default();

    let adrs = runtime::db::adrs::list_adrs().unwrap_or_default();

    let mut out = format!("# Project: {}\n\n", name);
    out.push_str("## Current Context\n");

    let workspaces = runtime_list_workspaces(project_dir).unwrap_or_default();
    let active_workspace = workspaces
        .iter()
        .find(|w| matches!(w.status, runtime::WorkspaceStatus::Active));

    if let Some(ws) = active_workspace {
        out.push_str(&format!(
            "- Workspace: {} [{:?}]",
            ws.branch, ws.workspace_type
        ));
        if let Some(ref mode) = ws.active_agent {
            out.push_str(&format!(" mode={}", mode));
        }

        out.push('\n');

        match runtime_get_active_workspace_session(project_dir, &ws.branch) {
            Ok(Some(session)) => {
                out.push_str(&format!("- Session: ACTIVE (id: {})", session.id));
                if let Some(ref goal) = session.goal {
                    out.push_str(&format!(" — goal: {}", goal));
                }
                if let Some(ref provider) = session.primary_provider {
                    out.push_str(&format!(" [{}]", provider));
                }
                out.push('\n');
            }
            Ok(None) => out.push_str("- Session: none (call start_session to begin)\n"),
            Err(_) => {}
        }
    } else {
        out.push_str("- Workspace: none active (call activate_workspace to begin)\n");
    }

    if let Some(active_id) = config.active_agent.as_deref() {
        if let Some(mode) = config.modes.iter().find(|m| m.id == active_id) {
            out.push_str(&format!("- Mode: {} ({})\n", mode.name, mode.id));
        }
    } else if !config.modes.is_empty() {
        out.push_str("- Mode: none (available: ");
        let names: Vec<_> = config.modes.iter().map(|m| m.id.as_str()).collect();
        out.push_str(&names.join(", "));
        out.push_str(")\n");
    }

    out.push_str("\n## ADRs\n");
    if adrs.is_empty() {
        out.push_str("No ADRs.\n");
    } else {
        for a in &adrs {
            out.push_str(&format!("- {} [{}] {}\n", a.id, a.status, a.title));
        }
    }

    if let Ok(log) = read_log(project_dir) {
        let recent: Vec<&str> = log
            .lines()
            .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
            .rev()
            .take(10)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        if !recent.is_empty() {
            out.push_str("\n## Recent Activity\n");
            for line in recent {
                out.push_str(&format!("{}\n", line));
            }
        }
    }

    if let Ok(events) = read_recent_events(project_dir, 10)
        && !events.is_empty()
    {
        out.push_str("\n## Recent Events\n");
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
    }

    out
}
