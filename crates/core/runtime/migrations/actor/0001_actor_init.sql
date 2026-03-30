-- Per-actor DB schema.
-- Each actor gets its own isolated events.db with this minimal schema.

CREATE TABLE IF NOT EXISTS events (
  id               TEXT PRIMARY KEY NOT NULL,
  event_type       TEXT NOT NULL,
  entity_id        TEXT NOT NULL,
  actor            TEXT NOT NULL DEFAULT 'ship',
  payload_json     TEXT NOT NULL DEFAULT '{}',
  version          INTEGER NOT NULL DEFAULT 1,
  correlation_id   TEXT,
  causation_id     TEXT,
  workspace_id     TEXT,
  session_id       TEXT,
  actor_id         TEXT,
  parent_actor_id  TEXT,
  elevated         INTEGER NOT NULL DEFAULT 0,
  created_at       TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_events_entity    ON events(entity_id, event_type);
CREATE INDEX IF NOT EXISTS idx_events_workspace ON events(workspace_id, created_at);
CREATE INDEX IF NOT EXISTS idx_events_session   ON events(session_id);
CREATE INDEX IF NOT EXISTS idx_events_corr      ON events(correlation_id);
CREATE INDEX IF NOT EXISTS idx_events_actor     ON events(actor_id, elevated);

CREATE TRIGGER IF NOT EXISTS events_immutable
BEFORE UPDATE ON events
BEGIN
  SELECT RAISE(FAIL, 'events table is immutable -- use append only');
END;
