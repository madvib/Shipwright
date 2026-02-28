use crate::state_db::{get_workspace_db, upsert_workspace_db};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::Path;
use std::str::FromStr;

// ─── Data types ───────────────────────────────────────────────────────────────

/// Branch session state — SQLite only, no frontmatter file.
/// Created/updated automatically via the post-checkout hook.
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct Workspace {
    pub branch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_mode: Option<String>,
    pub providers: Vec<String>,
    pub resolved_at: DateTime<Utc>,
    pub is_worktree: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worktree_path: Option<String>,
}

// ─── CRUD ─────────────────────────────────────────────────────────────────────

pub fn get_workspace(ship_dir: &Path, branch: &str) -> Result<Option<Workspace>> {
    let row = get_workspace_db(ship_dir, branch)?;
    Ok(row.map(
        |(feature_id, spec_id, active_mode, providers, resolved_at, is_worktree, worktree_path)| {
            let resolved_at = DateTime::from_str(&resolved_at).unwrap_or_else(|_| Utc::now());
            Workspace {
                branch: branch.to_string(),
                feature_id,
                spec_id,
                active_mode,
                providers,
                resolved_at,
                is_worktree,
                worktree_path,
            }
        },
    ))
}

pub fn upsert_workspace(ship_dir: &Path, workspace: &Workspace) -> Result<()> {
    upsert_workspace_db(
        ship_dir,
        &workspace.branch,
        workspace.feature_id.as_deref(),
        workspace.spec_id.as_deref(),
        workspace.active_mode.as_deref(),
        &workspace.providers,
        &workspace.resolved_at.to_rfc3339(),
        workspace.is_worktree,
        workspace.worktree_path.as_deref(),
    )
}
