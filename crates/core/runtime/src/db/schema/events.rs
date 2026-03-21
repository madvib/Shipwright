//! Schema DDL for the append-only event log.

/// Event log: append-only state change record. Never update or delete events.
/// Context columns (workspace_id, session_id, job_id) allow scoped queries
/// without joins. Replaces the deprecated job_log for job-scoped logging.
pub const EVENT_LOG: &str = r#"
-- event_log: append-only audit trail. Never update or delete rows.
-- Context columns enable scoped queries without joins.
CREATE TABLE IF NOT EXISTS event_log (
    id            TEXT PRIMARY KEY NOT NULL,
    actor         TEXT NOT NULL DEFAULT 'ship',
    entity_type   TEXT NOT NULL,
    entity_id     TEXT,
    action        TEXT NOT NULL,
    detail        TEXT,
    workspace_id  TEXT,
    session_id    TEXT,
    job_id        TEXT,
    created_at    TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_event_workspace ON event_log(workspace_id);
CREATE INDEX IF NOT EXISTS idx_event_session ON event_log(session_id);
CREATE INDEX IF NOT EXISTS idx_event_job ON event_log(job_id);
"#;
