-- Add target_actor_id for directed delivery.
ALTER TABLE events ADD COLUMN target_actor_id TEXT;
