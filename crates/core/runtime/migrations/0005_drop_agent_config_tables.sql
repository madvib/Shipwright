-- Drop the god-object agent_runtime_settings singleton and the redundant
-- agent_config table.  Data migrates to kv_state (runtime settings) and
-- is read from .ship/agents/*.jsonc (modes).  The agent_artifact_registry
-- table is intentionally retained.

-- Migrate existing runtime settings into kv_state before dropping.
INSERT OR IGNORE INTO kv_state (namespace, key, value_json, updated_at)
  SELECT 'runtime', 'providers', COALESCE(providers_json, '[]'), COALESCE(updated_at, datetime('now'))
  FROM agent_runtime_settings WHERE id = 1;

INSERT OR IGNORE INTO kv_state (namespace, key, value_json, updated_at)
  SELECT 'runtime', 'active_agent', CASE WHEN active_agent IS NOT NULL THEN json_quote(active_agent) ELSE 'null' END, COALESCE(updated_at, datetime('now'))
  FROM agent_runtime_settings WHERE id = 1;

INSERT OR IGNORE INTO kv_state (namespace, key, value_json, updated_at)
  SELECT 'runtime', 'hooks', COALESCE(hooks_json, '[]'), COALESCE(updated_at, datetime('now'))
  FROM agent_runtime_settings WHERE id = 1;

INSERT OR IGNORE INTO kv_state (namespace, key, value_json, updated_at)
  SELECT 'runtime', 'statuses', COALESCE(statuses_json, '[]'), COALESCE(updated_at, datetime('now'))
  FROM agent_runtime_settings WHERE id = 1;

INSERT OR IGNORE INTO kv_state (namespace, key, value_json, updated_at)
  SELECT 'runtime', 'ai', CASE WHEN ai_json IS NOT NULL THEN ai_json ELSE 'null' END, COALESCE(updated_at, datetime('now'))
  FROM agent_runtime_settings WHERE id = 1;

INSERT OR IGNORE INTO kv_state (namespace, key, value_json, updated_at)
  SELECT 'runtime', 'git', COALESCE(git_json, '{}'), COALESCE(updated_at, datetime('now'))
  FROM agent_runtime_settings WHERE id = 1;

INSERT OR IGNORE INTO kv_state (namespace, key, value_json, updated_at)
  SELECT 'runtime', 'namespaces', COALESCE(namespaces_json, '[]'), COALESCE(updated_at, datetime('now'))
  FROM agent_runtime_settings WHERE id = 1;

-- Migrate agent_config rows into a single kv_state JSON array.
INSERT OR IGNORE INTO kv_state (namespace, key, value_json, updated_at)
  SELECT 'runtime', 'modes',
    COALESCE(
      (SELECT json_group_array(
        json_object(
          'id', id,
          'name', name,
          'description', description,
          'active_tools_json', active_tools_json,
          'mcp_refs_json', mcp_refs_json,
          'skill_refs_json', skill_refs_json,
          'rule_refs_json', rule_refs_json,
          'prompt_id', prompt_id,
          'hooks_json', hooks_json,
          'permissions_json', permissions_json,
          'target_agents_json', target_agents_json
        )
      ) FROM agent_config),
      '[]'
    ),
    datetime('now');

DROP TABLE IF EXISTS agent_runtime_settings;
DROP TABLE IF EXISTS agent_config;
