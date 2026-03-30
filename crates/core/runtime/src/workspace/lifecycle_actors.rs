//! Actor auto-creation helpers for workspace lifecycle.
//!
//! Keeps lifecycle.rs under the 300-line cap. These functions are private
//! to the workspace module — called only from lifecycle.rs.

use anyhow::Result;

use crate::db::actor_events::{emit_actor_created, emit_actor_stopped};
use crate::events::types::ActorCreated;

use super::types::Workspace;

/// Derive the actor ID for a workspace: `{workspace_id}/{agent_id}`.
pub(super) fn actor_id_for_workspace(workspace: &Workspace) -> String {
    let agent = workspace.active_agent.as_deref().unwrap_or("default");
    format!("{}/{}", workspace.id, agent)
}

/// Query the current actor for this workspace from the events table.
///
/// Reads from platform.db events for immediate consistency — no dependency
/// on the async ActorProjection. Finds the actor whose last event was not
/// actor.stopped (handles re-creation correctly via MAX(id) grouping).
pub(super) fn current_actor_in_workspace(workspace: &Workspace) -> Result<Option<String>> {
    let mut conn = crate::db::open_db()?;
    let rows: Vec<(String,)> = crate::db::block_on(async {
        sqlx::query_as(
            "SELECT e.actor_id FROM events e \
             INNER JOIN ( \
               SELECT actor_id, MAX(id) AS max_id FROM events \
               WHERE workspace_id = ? AND actor_id IS NOT NULL \
               AND event_type IN ('actor.created','actor.woke','actor.slept', \
                                  'actor.stopped','actor.crashed') \
               GROUP BY actor_id \
             ) latest ON e.actor_id = latest.actor_id AND e.id = latest.max_id \
             WHERE e.event_type != 'actor.stopped' \
             ORDER BY e.id DESC LIMIT 1",
        )
        .bind(&workspace.id)
        .fetch_all(&mut conn)
        .await
    })?;
    Ok(rows.first().map(|r| r.0.clone()))
}

/// Ensure an actor exists for the workspace's current agent.
/// If the agent changed, stop the old actor and create a new one.
pub(super) fn ensure_actor_for_workspace(workspace: &Workspace) -> Result<()> {
    let desired_id = actor_id_for_workspace(workspace);
    let ws_id = Some(workspace.id.as_str());

    match current_actor_in_workspace(workspace)? {
        Some(existing_id) if existing_id == desired_id => {
            // Actor already exists for this agent — nothing to do.
        }
        Some(existing_id) => {
            // Agent changed — stop old actor, create new one.
            emit_actor_stopped(&existing_id, "agent changed", ws_id, None)?;
            emit_actor_created(
                &desired_id,
                &ActorCreated {
                    kind: "workspace".to_string(),
                    environment_type: "local".to_string(),
                    workspace_id: ws_id.map(str::to_string),
                    parent_actor_id: None,
                    restart_count: 0,
                },
                ws_id,
                None,
            )?;
        }
        None => {
            // No actor yet — create one.
            emit_actor_created(
                &desired_id,
                &ActorCreated {
                    kind: "workspace".to_string(),
                    environment_type: "local".to_string(),
                    workspace_id: ws_id.map(str::to_string),
                    parent_actor_id: None,
                    restart_count: 0,
                },
                ws_id,
                None,
            )?;
        }
    }

    Ok(())
}
