use crate::config::{get_config, HookConfig, HookTrigger, McpServerConfig, McpServerType, PermissionConfig};
use crate::prompt::Prompt;
use crate::prompt::get_prompt;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// ─── Managed state ────────────────────────────────────────────────────────────

/// Tracks which server IDs Ship wrote into each tool's config.
/// Stored at `.ship/mcp_managed_state.toml` so we can identify Ship-managed
/// servers on re-read without relying on in-file markers (Codex TOML can't hold them).
#[derive(Serialize, Deserialize, Debug, Default)]
struct ManagedState {
    #[serde(default)]
    claude: ToolState,
    #[serde(default)]
    gemini: ToolState,
    #[serde(default)]
    codex: ToolState,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct ToolState {
    #[serde(default)]
    managed_servers: Vec<String>,
    last_mode: Option<String>,
}

fn managed_state_path(project_dir: &Path) -> PathBuf {
    project_dir.join("mcp_managed_state.toml")
}

fn load_managed_state(project_dir: &Path) -> ManagedState {
    let path = managed_state_path(project_dir);
    if !path.exists() {
        return ManagedState::default();
    }
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_managed_state(project_dir: &Path, state: &ManagedState) -> Result<()> {
    let path = managed_state_path(project_dir);
    crate::fs_util::write_atomic(&path, toml::to_string_pretty(state)?)
}

// ─── Sync payload ─────────────────────────────────────────────────────────────

pub struct SyncPayload {
    pub servers: Vec<McpServerConfig>,
    pub prompt: Option<Prompt>,
    pub hooks: Vec<HookConfig>,
    pub permissions: PermissionConfig,
    pub active_mode_id: Option<String>,
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Export the active mode (or global config) to the specified AI client.
pub fn export_to(project_dir: PathBuf, target: &str) -> Result<()> {
    let payload = build_payload(&project_dir)?;
    match target {
        "claude" => export_claude(&project_dir, &payload),
        "codex"  => export_codex(&project_dir, &payload),
        "gemini" => export_gemini(&project_dir, &payload),
        other    => Err(anyhow!("Unknown target '{}': use claude, codex, or gemini", other)),
    }
}

/// Sync all target agents configured for the active mode.
/// Returns list of synced target names.
pub fn sync_active_mode(project_dir: &Path) -> Result<Vec<String>> {
    let config = get_config(Some(project_dir.to_path_buf()))?;
    let targets: Vec<String> = config.active_mode
        .as_ref()
        .and_then(|id| config.modes.iter().find(|m| &m.id == id))
        .map(|m| {
            if m.target_agents.is_empty() {
                vec!["claude".to_string()]
            } else {
                m.target_agents.clone()
            }
        })
        .unwrap_or_default();

    let mut synced = Vec::new();
    for target in &targets {
        export_to(project_dir.to_path_buf(), target)?;
        synced.push(target.clone());
    }
    Ok(synced)
}

/// Non-destructive import of MCP servers from Claude's global config.
/// Returns count of newly-added servers.
pub fn import_from_claude(project_dir: PathBuf) -> Result<usize> {
    let path = home()?.join(".claude.json");
    if !path.exists() {
        return Ok(0);
    }
    let root: serde_json::Value = serde_json::from_str(&fs::read_to_string(&path)?)?;
    let Some(mcp_obj) = root.get("mcpServers").and_then(|v| v.as_object()) else {
        return Ok(0);
    };
    // Also try .mcp.json at project root (project-scoped)
    let state = load_managed_state(&project_dir);
    let mut config = get_config(Some(project_dir.clone()))?;
    let mut added = 0usize;

    for (id, entry) in mcp_obj {
        // Skip servers Ship manages itself
        if state.claude.managed_servers.contains(id) {
            continue;
        }
        if config.mcp_servers.iter().any(|s| &s.id == id) {
            continue;
        }
        let server_type = match entry.get("type").and_then(|v| v.as_str()) {
            Some("sse")  => McpServerType::Sse,
            Some("http") => McpServerType::Http,
            _            => McpServerType::Stdio,
        };
        let command = entry.get("command")
            .and_then(|v| v.as_str()).unwrap_or("").to_string();
        let args = entry.get("args").and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
            .unwrap_or_default();
        let env = entry.get("env").and_then(|v| v.as_object())
            .map(|o| o.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect::<HashMap<_, _>>())
            .unwrap_or_default();
        let url = entry.get("url").and_then(|v| v.as_str()).map(str::to_string);
        let disabled = entry.get("disabled").and_then(|v| v.as_bool()).unwrap_or(false);

        config.mcp_servers.push(McpServerConfig {
            id: id.clone(),
            name: id.clone(),
            command,
            args,
            env,
            scope: "global".to_string(),
            server_type,
            url,
            disabled,
            timeout_secs: None,
        });
        added += 1;
    }

    if added > 0 {
        crate::config::save_config(&config, Some(project_dir))?;
    }
    Ok(added)
}

// ─── Payload builder ──────────────────────────────────────────────────────────

fn build_payload(project_dir: &Path) -> Result<SyncPayload> {
    let config = get_config(Some(project_dir.to_path_buf()))?;

    if let Some(mode_id) = &config.active_mode {
        if let Some(mode) = config.modes.iter().find(|m| &m.id == mode_id) {
            let servers = if mode.mcp_servers.is_empty() {
                config.mcp_servers.clone()
            } else {
                config.mcp_servers.iter()
                    .filter(|s| mode.mcp_servers.contains(&s.id))
                    .cloned()
                    .collect()
            };
            let prompt = mode.prompt_id.as_ref()
                .and_then(|id| get_prompt(project_dir, id).ok());
            let mut hooks = config.hooks.clone();
            hooks.extend(mode.hooks.clone());
            return Ok(SyncPayload {
                servers,
                prompt,
                hooks,
                permissions: mode.permissions.clone(),
                active_mode_id: Some(mode_id.clone()),
            });
        }
    }

    Ok(SyncPayload {
        servers: config.mcp_servers,
        prompt: None,
        hooks: config.hooks,
        permissions: Default::default(),
        active_mode_id: config.active_mode,
    })
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn home() -> Result<PathBuf> {
    home::home_dir().ok_or_else(|| anyhow!("Cannot determine home directory"))
}

/// Injects Ship's own server entry — always present in every exported config.
fn ship_server_entry() -> (&'static str, serde_json::Value) {
    let entry = serde_json::json!({
        "command": "ship",
        "args": ["mcp"],
        "type": "stdio",
        "_ship": { "managed": true }
    });
    ("ship", entry)
}

// ─── Claude ───────────────────────────────────────────────────────────────────
//
// Project scope: `.mcp.json` at project root (the parent of .ship/)
// Global scope:  `~/.claude.json`
//
// Format: { "mcpServers": { "<id>": { "command", "args", "env", "type" } } }
// Ship marker: "_ship": { "managed": true } in each entry (JSON supports it)

fn export_claude(project_dir: &Path, payload: &SyncPayload) -> Result<()> {
    // project_dir is .ship/ — write .mcp.json one level up at the repo root
    let project_root = project_dir.parent()
        .ok_or_else(|| anyhow!("Cannot determine project root from {:?}", project_dir))?;
    let mcp_json = project_root.join(".mcp.json");

    // Read existing, preserve user servers
    let existing: serde_json::Value = if mcp_json.exists() {
        serde_json::from_str(&fs::read_to_string(&mcp_json)?).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    let mut state = load_managed_state(project_dir);
    let mut mcp_servers = serde_json::Map::new();

    // Preserve user-defined servers (not ship-managed)
    if let Some(existing_mcp) = existing.get("mcpServers").and_then(|v| v.as_object()) {
        for (id, entry) in existing_mcp {
            let is_managed = entry.get("_ship")
                .and_then(|v| v.get("managed"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
                || state.claude.managed_servers.contains(id);
            if !is_managed {
                mcp_servers.insert(id.clone(), entry.clone());
            }
        }
    }

    // Always inject Ship itself
    let (ship_id, ship_entry) = ship_server_entry();
    mcp_servers.insert(ship_id.to_string(), ship_entry);

    // Write mode servers with Ship marker
    let mut written_ids = vec![ship_id.to_string()];
    for s in &payload.servers {
        if s.disabled { continue; }
        let mut entry = claude_mcp_entry(s);
        entry["_ship"] = serde_json::json!({ "managed": true });
        mcp_servers.insert(s.id.clone(), entry);
        written_ids.push(s.id.clone());
    }

    // Rebuild file preserving non-mcpServers fields
    let mut root = existing.clone();
    if !root.is_object() {
        root = serde_json::json!({});
    }
    root["mcpServers"] = serde_json::Value::Object(mcp_servers);

    crate::fs_util::write_atomic(&mcp_json, serde_json::to_string_pretty(&root)?)?;

    // Update managed state
    state.claude.managed_servers = written_ids;
    state.claude.last_mode = payload.active_mode_id.clone();
    save_managed_state(project_dir, &state)?;

    // Also write hooks + permissions to ~/.claude/settings.json
    if !payload.hooks.is_empty() || !payload.permissions.allow.is_empty() || !payload.permissions.deny.is_empty() {
        export_claude_settings(&payload.hooks, &payload.permissions)?;
    }

    // Write prompt to project CLAUDE.md if set
    if let Some(prompt) = &payload.prompt {
        let claude_md = project_root.join("CLAUDE.md");
        let content = format!("<!-- managed by ship — prompt: {} -->\n\n{}\n", prompt.id, prompt.content);
        crate::fs_util::write_atomic(&claude_md, content)?;
    }

    Ok(())
}

fn claude_mcp_entry(s: &McpServerConfig) -> serde_json::Value {
    match s.server_type {
        McpServerType::Stdio => {
            let mut entry = serde_json::json!({ "command": s.command, "type": "stdio" });
            if !s.args.is_empty() {
                entry["args"] = serde_json::json!(s.args);
            }
            if !s.env.is_empty() {
                entry["env"] = serde_json::json!(s.env);
            }
            entry
        }
        McpServerType::Http => serde_json::json!({ "type": "http", "url": s.url }),
        McpServerType::Sse  => serde_json::json!({ "type": "sse",  "url": s.url }),
    }
}

/// ~/.claude/settings.json — hooks and permissions only (not MCP servers)
fn export_claude_settings(hooks: &[HookConfig], permissions: &PermissionConfig) -> Result<()> {
    let path = home()?.join(".claude").join("settings.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut root: serde_json::Value = if path.exists() {
        serde_json::from_str(&fs::read_to_string(&path)?).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    let obj = root.as_object_mut()
        .ok_or_else(|| anyhow!("~/.claude/settings.json is not an object"))?;

    // Permissions
    if !permissions.allow.is_empty() || !permissions.deny.is_empty() {
        let perms = obj.entry("permissions").or_insert(serde_json::json!({}));
        let p = perms.as_object_mut().ok_or_else(|| anyhow!("permissions not an object"))?;
        if !permissions.allow.is_empty() {
            p.insert("allow".to_string(), serde_json::json!(permissions.allow));
        }
        if !permissions.deny.is_empty() {
            p.insert("deny".to_string(), serde_json::json!(permissions.deny));
        }
    }

    // Hooks — grouped by trigger name, each is an array of hook objects
    if !hooks.is_empty() {
        let hooks_val = obj.entry("hooks").or_insert(serde_json::json!({}));
        let hooks_map = hooks_val.as_object_mut()
            .ok_or_else(|| anyhow!("hooks not an object"))?;
        let mut by_trigger: HashMap<&str, Vec<serde_json::Value>> = HashMap::new();
        for hook in hooks {
            let key = match hook.trigger {
                HookTrigger::PreToolUse   => "PreToolUse",
                HookTrigger::PostToolUse  => "PostToolUse",
                HookTrigger::Notification => "Notification",
                HookTrigger::Stop         => "Stop",
                HookTrigger::SubagentStop => "SubagentStop",
                HookTrigger::PreCompact   => "PreCompact",
            };
            let mut entry = serde_json::json!({ "type": "command", "command": hook.command });
            if let Some(m) = &hook.matcher {
                entry["matcher"] = serde_json::json!(m);
            }
            by_trigger.entry(key).or_default().push(entry);
        }
        for (trigger, entries) in by_trigger {
            hooks_map.insert(trigger.to_string(), serde_json::json!(entries));
        }
    }

    crate::fs_util::write_atomic(&path, serde_json::to_string_pretty(&root)?)
}

// ─── Gemini ───────────────────────────────────────────────────────────────────
//
// Project scope: `.gemini/settings.json` at project root
// Global scope:  `~/.gemini/settings.json`
//
// IMPORTANT differences from Claude:
// - HTTP servers use `httpUrl` not `url`
// - Env vars are NOT auto-inherited — must be in `env` property
// - Enablement state tracked in ~/.gemini/mcp-server-enablement.json (read-only for us)

fn export_gemini(project_dir: &Path, payload: &SyncPayload) -> Result<()> {
    let project_root = project_dir.parent()
        .ok_or_else(|| anyhow!("Cannot determine project root from {:?}", project_dir))?;
    let settings_path = project_root.join(".gemini").join("settings.json");
    if let Some(parent) = settings_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let existing: serde_json::Value = if settings_path.exists() {
        serde_json::from_str(&fs::read_to_string(&settings_path)?).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    let mut state = load_managed_state(project_dir);
    let mut mcp_servers = serde_json::Map::new();

    // Preserve user servers
    if let Some(existing_mcp) = existing.get("mcpServers").and_then(|v| v.as_object()) {
        for (id, entry) in existing_mcp {
            let is_managed = entry.get("_ship")
                .and_then(|v| v.get("managed"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
                || state.gemini.managed_servers.contains(id);
            if !is_managed {
                mcp_servers.insert(id.clone(), entry.clone());
            }
        }
    }

    // Ship self
    let (ship_id, mut ship_entry) = ship_server_entry();
    // Gemini: no type field needed for stdio
    ship_entry.as_object_mut().map(|o| o.remove("type"));
    mcp_servers.insert(ship_id.to_string(), ship_entry);

    let mut written_ids = vec![ship_id.to_string()];
    for s in &payload.servers {
        if s.disabled { continue; }
        let mut entry = gemini_mcp_entry(s);
        entry["_ship"] = serde_json::json!({ "managed": true });
        mcp_servers.insert(s.id.clone(), entry);
        written_ids.push(s.id.clone());
    }

    // Rebuild preserving non-mcpServers fields (theme, selectedAuthType, etc.)
    let mut root = existing.clone();
    if !root.is_object() { root = serde_json::json!({}); }
    root["mcpServers"] = serde_json::Value::Object(mcp_servers);

    crate::fs_util::write_atomic(&settings_path, serde_json::to_string_pretty(&root)?)?;

    state.gemini.managed_servers = written_ids;
    state.gemini.last_mode = payload.active_mode_id.clone();
    save_managed_state(project_dir, &state)?;

    // Write GEMINI.md if prompt set
    if let Some(prompt) = &payload.prompt {
        let gemini_md = project_root.join("GEMINI.md");
        let content = format!("<!-- managed by ship — prompt: {} -->\n\n{}\n", prompt.id, prompt.content);
        crate::fs_util::write_atomic(&gemini_md, content)?;
    }

    Ok(())
}

fn gemini_mcp_entry(s: &McpServerConfig) -> serde_json::Value {
    match s.server_type {
        McpServerType::Stdio => {
            let mut entry = serde_json::json!({ "command": s.command });
            if !s.args.is_empty() {
                entry["args"] = serde_json::json!(s.args);
            }
            if !s.env.is_empty() {
                // Gemini requires explicit env — NOT auto-inherited from shell
                entry["env"] = serde_json::json!(s.env);
            }
            entry
        }
        // Gemini uses "httpUrl", NOT "url". HTTP headers don't expand env vars.
        McpServerType::Http | McpServerType::Sse => {
            let mut entry = serde_json::json!({ "httpUrl": s.url });
            if let Some(t) = s.timeout_secs {
                entry["timeout"] = serde_json::json!(t * 1000); // Gemini timeout is ms
            }
            entry
        }
    }
}

// ─── Codex ────────────────────────────────────────────────────────────────────
//
// Project scope: `.codex/config.toml` at project root
// Global scope:  `~/.codex/config.toml`
//
// CRITICAL: section is `mcp_servers` with UNDERSCORE, NOT `mcp-servers`.
// Using `mcp-servers` silently does nothing.

fn export_codex(project_dir: &Path, payload: &SyncPayload) -> Result<()> {
    let project_root = project_dir.parent()
        .ok_or_else(|| anyhow!("Cannot determine project root from {:?}", project_dir))?;
    let config_path = project_root.join(".codex").join("config.toml");
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let raw_existing = if config_path.exists() {
        fs::read_to_string(&config_path)?
    } else {
        String::new()
    };

    // Parse existing, gracefully handle malformed TOML
    let mut doc: toml::Value = if raw_existing.is_empty() {
        toml::Value::Table(Default::default())
    } else {
        toml::from_str(&raw_existing).map_err(|e| {
            anyhow!("Cannot parse {}: {}.\nNote: Codex uses 'mcp_servers' (underscore), check for 'mcp-servers' (hyphen).", config_path.display(), e)
        })?
    };

    let root = match &mut doc {
        toml::Value::Table(t) => t,
        _ => return Err(anyhow!("Codex config root is not a TOML table")),
    };

    let mut state = load_managed_state(project_dir);

    // Preserve user-defined servers
    let existing_mcp: toml::value::Table = root
        .get("mcp_servers")  // UNDERSCORE — not hyphen
        .and_then(|v| v.as_table())
        .cloned()
        .unwrap_or_default();

    let mut new_mcp = toml::value::Table::new();

    // Keep user servers (not ship-managed)
    for (id, entry) in &existing_mcp {
        if !state.codex.managed_servers.contains(id) {
            new_mcp.insert(id.clone(), entry.clone());
        }
    }

    // Ship self
    let mut ship_entry = toml::value::Table::new();
    ship_entry.insert("command".to_string(), toml::Value::String("ship".to_string()));
    ship_entry.insert("args".to_string(), toml::Value::Array(vec![toml::Value::String("mcp".to_string())]));
    new_mcp.insert("ship".to_string(), toml::Value::Table(ship_entry));

    let mut written_ids = vec!["ship".to_string()];

    for s in &payload.servers {
        if s.disabled { continue; }
        new_mcp.insert(s.id.clone(), codex_mcp_entry(s));
        written_ids.push(s.id.clone());
    }

    root.insert("mcp_servers".to_string(), toml::Value::Table(new_mcp)); // UNDERSCORE

    // System prompt → instructions field
    if let Some(prompt) = &payload.prompt {
        root.insert("instructions".to_string(), toml::Value::String(prompt.content.clone()));
    }

    crate::fs_util::write_atomic(&config_path, toml::to_string_pretty(&doc)?)?;

    state.codex.managed_servers = written_ids;
    state.codex.last_mode = payload.active_mode_id.clone();
    save_managed_state(project_dir, &state)?;

    Ok(())
}

fn codex_mcp_entry(s: &McpServerConfig) -> toml::Value {
    let mut entry = toml::value::Table::new();
    match s.server_type {
        McpServerType::Stdio => {
            entry.insert("command".to_string(), toml::Value::String(s.command.clone()));
            if !s.args.is_empty() {
                entry.insert("args".to_string(),
                    toml::Value::Array(s.args.iter().map(|a| toml::Value::String(a.clone())).collect()));
            }
            if !s.env.is_empty() {
                let env_table: toml::value::Table = s.env.iter()
                    .map(|(k, v)| (k.clone(), toml::Value::String(v.clone())))
                    .collect();
                entry.insert("env".to_string(), toml::Value::Table(env_table));
            }
        }
        McpServerType::Http | McpServerType::Sse => {
            if let Some(url) = &s.url {
                entry.insert("url".to_string(), toml::Value::String(url.clone()));
            }
            // bearer_token_env_var could be stored in McpServerConfig.env as a special key
            // For now: if env has exactly one key ending in _TOKEN, treat it as bearer
            for (k, _v) in &s.env {
                if k.ends_with("_TOKEN") || k.ends_with("_KEY") {
                    entry.insert("bearer_token_env_var".to_string(), toml::Value::String(k.clone()));
                    break;
                }
            }
        }
    }
    if let Some(t) = s.timeout_secs {
        entry.insert("startup_timeout_sec".to_string(), toml::Value::Integer(t as i64));
    }
    toml::Value::Table(entry)
}
