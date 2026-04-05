-- Per-workspace DB schema.
-- Each workspace gets its own events.db with this schema.

CREATE TABLE IF NOT EXISTS events (
  id               TEXT PRIMARY KEY NOT NULL,
  event_type       TEXT NOT NULL,
  entity_id        TEXT NOT NULL,
  actor            TEXT NOT NULL DEFAULT 'ship',
  payload_json     TEXT NOT NULL DEFAULT '{}',
  version          INTEGER NOT NULL DEFAULT 1,
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
CREATE INDEX IF NOT EXISTS idx_events_actor     ON events(actor_id, elevated);

CREATE TRIGGER IF NOT EXISTS events_immutable
BEFORE UPDATE ON events
BEGIN
  SELECT RAISE(FAIL, 'events table is immutable -- use append only');
END;

CREATE TABLE IF NOT EXISTS actors (
  id               TEXT PRIMARY KEY NOT NULL,
  kind             TEXT NOT NULL,
  environment_type TEXT NOT NULL DEFAULT 'local',
  status           TEXT NOT NULL DEFAULT 'created',
  workspace_id     TEXT,
  parent_actor_id  TEXT,
  restart_count    INTEGER NOT NULL DEFAULT 0,
  created_at       TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at       TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_actors_workspace ON actors(workspace_id);
CREATE INDEX IF NOT EXISTS idx_actors_status    ON actors(status);

CREATE TABLE IF NOT EXISTS workspace_session (
  id                          TEXT PRIMARY KEY,
  workspace_id                TEXT NOT NULL,
  workspace_branch            TEXT NOT NULL,
  status                      TEXT NOT NULL DEFAULT 'active',
  started_at                  TEXT NOT NULL,
  ended_at                    TEXT,
  agent_id                    TEXT,
  preset_id                   TEXT,
  primary_provider            TEXT,
  goal                        TEXT,
  summary                     TEXT,
  updated_workspace_ids_json  TEXT NOT NULL DEFAULT '[]',
  compiled_at                 TEXT,
  compile_error               TEXT,
  config_generation_at_start  INTEGER,
  created_at                  TEXT NOT NULL,
  updated_at                  TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS workspace_session_workspace_idx
  ON workspace_session(workspace_id, started_at DESC);
CREATE INDEX IF NOT EXISTS workspace_session_status_idx
  ON workspace_session(status, started_at DESC);
