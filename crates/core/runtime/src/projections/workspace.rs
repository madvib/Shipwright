//! Workspace state projection — derived from workspace.* events.
//!
//! This projection maintains the `workspace` table as a read model.
//! It handles every workspace event type and applies the state change.
//! On rebuild, the table is truncated and replayed from the event log.
//!
//! Handler functions live in `workspace_handlers` to stay under the line cap.

use anyhow::Result;
use sqlx::SqliteConnection;

use super::async_projection::AsyncProjection;
use super::registry::Projection;
use super::workspace_handlers::*;
use crate::db::block_on;
use crate::events::types::event_types;
use crate::events::EventEnvelope;

/// Projection that maintains the workspace table from workspace.* events.
pub struct WorkspaceProjection;

impl WorkspaceProjection {
    pub fn new() -> Self {
        Self
    }
}

const HANDLED: &[&str] = &[
    event_types::WORKSPACE_CREATED,
    event_types::WORKSPACE_ACTIVATED,
    event_types::WORKSPACE_COMPILED,
    event_types::WORKSPACE_COMPILE_FAILED,
    event_types::WORKSPACE_ARCHIVED,
    event_types::WORKSPACE_STATUS_CHANGED,
    event_types::WORKSPACE_AGENT_CHANGED,
    event_types::WORKSPACE_RECONCILED,
    event_types::WORKSPACE_DELETED,
];

impl Projection for WorkspaceProjection {
    fn name(&self) -> &str {
        "workspace_state"
    }

    fn event_types(&self) -> &[&str] {
        HANDLED
    }

    fn apply(&self, event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
        let entity = &event.entity_id;
        match event.event_type.as_str() {
            event_types::WORKSPACE_CREATED => apply_created(entity, &event.payload_json, conn),
            event_types::WORKSPACE_ACTIVATED => apply_activated(entity, &event.payload_json, conn),
            event_types::WORKSPACE_COMPILED => apply_compiled(entity, &event.payload_json, &event.created_at, conn),
            event_types::WORKSPACE_COMPILE_FAILED => apply_compile_failed(entity, &event.payload_json, &event.created_at, conn),
            event_types::WORKSPACE_ARCHIVED => apply_status(entity, "archived", conn),
            event_types::WORKSPACE_STATUS_CHANGED => apply_status_changed(entity, &event.payload_json, conn),
            event_types::WORKSPACE_AGENT_CHANGED => apply_agent_changed(entity, &event.payload_json, conn),
            event_types::WORKSPACE_RECONCILED => apply_reconciled(entity, &event.payload_json, conn),
            event_types::WORKSPACE_DELETED => apply_deleted(entity, conn),
            _ => Ok(()),
        }
    }

    fn truncate(&self, conn: &mut SqliteConnection) -> Result<()> {
        block_on(async {
            sqlx::query("DELETE FROM workspace")
                .execute(conn)
                .await?;
            Ok(())
        })
    }
}

impl AsyncProjection for WorkspaceProjection {
    fn name(&self) -> &str {
        Projection::name(self)
    }
    fn event_types(&self) -> &[&str] {
        Projection::event_types(self)
    }
    fn apply(&self, event: &EventEnvelope, conn: &mut sqlx::SqliteConnection) -> anyhow::Result<()> {
        Projection::apply(self, event, conn)
    }
    fn truncate(&self, conn: &mut sqlx::SqliteConnection) -> anyhow::Result<()> {
        Projection::truncate(self, conn)
    }
}
