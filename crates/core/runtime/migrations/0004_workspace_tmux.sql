-- Add tmux_session_name to workspace table for Studio terminal integration.
ALTER TABLE workspace ADD COLUMN tmux_session_name TEXT;
