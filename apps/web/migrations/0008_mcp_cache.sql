-- MCP server registry cache
-- Stores upstream registry data in D1 with a 24-hour TTL.
-- Not a source of truth — purely a cache layer.

CREATE TABLE mcp_servers_cache (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT,
  homepage TEXT,
  tags TEXT,              -- JSON array
  vendor TEXT,
  package_registry TEXT,  -- npm, pypi, etc.
  command TEXT,
  args TEXT,              -- JSON array
  vetted INTEGER NOT NULL DEFAULT 0,
  image_url TEXT,
  cached_at INTEGER NOT NULL
);

CREATE INDEX idx_mcp_cache_vetted ON mcp_servers_cache(vetted);
