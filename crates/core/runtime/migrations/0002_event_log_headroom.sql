-- 0002_event_log_headroom: Add event store columns for v0.2.0 event-sourced runtime.
-- All columns are nullable so existing rows require no backfill.
-- v0.2.0 will begin writing these; v0.1.x ignores them.

ALTER TABLE event_log ADD COLUMN version INTEGER;
ALTER TABLE event_log ADD COLUMN correlation_id TEXT;
ALTER TABLE event_log ADD COLUMN causation_id TEXT;
ALTER TABLE event_log ADD COLUMN synced_at TEXT;
