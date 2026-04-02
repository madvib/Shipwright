-- Session drain lifecycle (MCP auto-sessions)
ALTER TABLE workspace_session ADD COLUMN tool_call_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE workspace_session ADD COLUMN drained_at TEXT;
ALTER TABLE workspace_session ADD COLUMN mcp_provider TEXT;

CREATE INDEX IF NOT EXISTS idx_ws_session_drain
  ON workspace_session(status, workspace_id, agent_id);
