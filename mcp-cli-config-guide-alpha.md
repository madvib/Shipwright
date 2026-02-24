# Shipwright — AI CLI Config Management: Alpha Build Guide

**Version:** 0.1  
**Scope:** Alpha — Claude Code, Gemini CLI, Codex only  
**Last Updated:** 2026-02-22

---

## Overview

Shipwright manages MCP server configuration for the three major AI CLIs. Define your servers and modes once in `.ship/config.toml`. Shipwright generates the correct config for each tool, keeps them in sync on mode switch, and handles the import of existing setups.

This guide covers everything needed to build this feature for alpha.

---

## The Actual Config Formats

Verified from official documentation. These are the ground truth schemas Shipwright must read and write.

### Claude Code — `.mcp.json`

Project-scoped. Lives at the project root. Checked into version control. Shared with the team.

```json
{
  "mcpServers": {
    "shipwright": {
      "command": "shipwright",
      "args": ["mcp", "start", "--stdio"]
    },
    "github": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "$GITHUB_TOKEN"
      }
    },
    "postgres": {
      "type": "http",
      "url": "http://localhost:5433/mcp"
    }
  }
}
```

**Key facts:**
- File: `.mcp.json` at project root
- Global fallback: `~/.claude.json` (under the project path key)
- Supports stdio (`command` + `args`) and HTTP (`type: "http"`, `url`)
- Env vars support `$VAR` expansion
- Claude Code prompts for approval before using project-scoped servers from `.mcp.json` (security feature — can't be disabled programmatically)
- Restart required after changes

---

### Gemini CLI — `.gemini/settings.json`

Two locations: global (`~/.gemini/settings.json`) and project-local (`.gemini/settings.json`). Project-local takes precedence. `mcpServers` is a top-level key alongside other Gemini settings like `theme` and `selectedAuthType`.

```json
{
  "selectedAuthType": "gemini-api-key",
  "theme": "Dracula",
  "mcpServers": {
    "shipwright": {
      "command": "shipwright",
      "args": ["mcp", "start", "--stdio"]
    },
    "github": {
      "httpUrl": "https://api.githubcopilot.com/mcp/",
      "headers": {
        "Authorization": "Bearer $GITHUB_MCP_PAT"
      },
      "timeout": 5000
    },
    "git": {
      "command": "uvx",
      "args": ["mcp-server-git"]
    }
  }
}
```

**Key facts:**
- File: `.gemini/settings.json` (project) or `~/.gemini/settings.json` (global)
- Supports stdio (`command` + `args`) and HTTP (`httpUrl` + `headers`)
- HTTP servers also support `timeout` (milliseconds)
- `trust` option bypasses confirmation dialogs (use with caution)
- Enablement state tracked separately in `~/.gemini/mcp-server-enablement.json`
- Env vars: must be explicitly declared in `env` property — Gemini does NOT auto-inherit env
- Restart required after changes

**Critical difference from Claude:** Gemini does NOT expand env vars automatically in the `headers` field for HTTP servers. Tokens must be in the `env` property for stdio, or hardcoded for HTTP (which is a security concern). Design the UX to warn users about this.

---

### Codex — `.codex/config.toml`

Project-scoped at `.codex/config.toml` (trusted projects only). Global at `~/.codex/config.toml`. Uses TOML with `[mcp_servers.<name>]` tables. CLI and IDE extension share this file.

```toml
# .codex/config.toml

[mcp_servers.shipwright]
command = "shipwright"
args = ["mcp", "start", "--stdio"]

[mcp_servers.github]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-github"]
env = { GITHUB_PERSONAL_ACCESS_TOKEN = "$GITHUB_TOKEN" }

[mcp_servers.figma]
url = "https://mcp.figma.com/mcp"
bearer_token_env_var = "FIGMA_OAUTH_TOKEN"

[mcp_servers.postgres]
url = "http://localhost:5433/mcp"
enabled = true
enabled_tools = ["query", "describe"]    # allowlist
disabled_tools = ["drop", "truncate"]    # denylist — applied after enabled_tools
startup_timeout_sec = 20
tool_timeout_sec = 45
```

**Key facts:**
- File: `.codex/config.toml` (project) or `~/.codex/config.toml` (global)
- Section name is `mcp_servers` with underscore — NOT `mcp-servers` (silent failure if wrong)
- Supports stdio (`command` + `args`) and HTTP (`url`)
- HTTP auth: `bearer_token_env_var` (env var name, not value) or `http_headers`
- `enabled_tools` / `disabled_tools` for per-server tool allowlists
- `enabled = false` disables without removing
- `startup_timeout_sec` and `tool_timeout_sec` for fine-grained timeout control
- CLI and IDE extension share the same file — a syntax error breaks both simultaneously

---

## Schema Comparison

| Field | Claude (JSON) | Gemini (JSON) | Codex (TOML) |
|-------|--------------|---------------|--------------|
| Stdio command | `command` | `command` | `command` |
| Stdio args | `args` | `args` | `args` |
| Stdio env | `env: {}` | `env: {}` | `env = {}` |
| HTTP url | `type:"http", url` | `httpUrl` | `url` |
| HTTP headers | `headers` (in JSON) | `headers` | `http_headers` |
| HTTP bearer token | in `headers` | in `headers` | `bearer_token_env_var` |
| Timeout | — | `timeout` (ms) | `startup_timeout_sec`, `tool_timeout_sec` |
| Enable/disable | — | enablement file | `enabled = false` |
| Tool allowlist | — | — | `enabled_tools` |
| File format | JSON | JSON | TOML |
| Project scope file | `.mcp.json` | `.gemini/settings.json` | `.codex/config.toml` |
| Global scope file | `~/.claude.json` | `~/.gemini/settings.json` | `~/.codex/config.toml` |

---

## Architecture

### Shipwright's Internal Representation

One canonical schema for all servers. Formatters convert this to each tool's format.

```rust
// crates/runtime/src/mcp_manager/types.rs

/// Canonical server definition — tool-agnostic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerDef {
    pub id: String,
    pub transport: McpTransport,
    pub enabled: bool,
    pub shipwright_managed: bool,   // false = user-defined, don't modify
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum McpTransport {
    Stdio {
        command: String,
        args: Vec<String>,
        env: HashMap<String, String>,
    },
    Http {
        url: String,
        headers: HashMap<String, String>,
        bearer_token_env_var: Option<String>,
        timeout_ms: Option<u32>,
    },
}

impl McpServerDef {
    /// The Shipwright server itself — always injected into every config
    pub fn shipwright() -> Self {
        Self {
            id: "shipwright".to_string(),
            transport: McpTransport::Stdio {
                command: "shipwright".to_string(),
                args: vec!["mcp".to_string(), "start".to_string(), "--stdio".to_string()],
                env: HashMap::new(),
            },
            enabled: true,
            shipwright_managed: true,
        }
    }
}
```

### Formatter Trait

```rust
// crates/runtime/src/mcp_manager/formatter.rs

pub trait McpFormatter: Send + Sync {
    /// Tool identifier — "claude", "gemini", "codex"
    fn id(&self) -> &str;

    /// Human-readable name
    fn name(&self) -> &str;

    /// Where to write the project-scoped config file
    fn project_config_path(&self, project_root: &Path) -> PathBuf;

    /// Where the global config lives (for import only)
    fn global_config_path(&self) -> Option<PathBuf>;

    /// Whether this tool is installed (detect binary in PATH)
    fn is_installed(&self) -> bool;

    /// Read existing config and extract MCP server definitions
    /// Returns (shipwright_servers, user_servers) — separate so we preserve user content
    fn read(&self, path: &Path) -> Result<ReadResult>;

    /// Generate the config file content for the given servers
    /// Merges Shipwright servers into existing user content
    fn format(&self, existing: ReadResult, servers: &[McpServerDef]) -> Result<String>;

    /// Whether the tool needs to be restarted after config changes
    fn requires_restart(&self) -> bool { true }
}

pub struct ReadResult {
    pub shipwright_servers: Vec<McpServerDef>,   // previously written by Shipwright
    pub user_servers: Vec<McpServerDef>,          // user-defined — NEVER modify
    pub raw_other_fields: serde_json::Value,      // non-MCP fields — preserve exactly
}
```

### Formatter Registry

```rust
// crates/runtime/src/mcp_manager/registry.rs

pub fn formatters() -> Vec<Box<dyn McpFormatter>> {
    vec![
        Box::new(ClaudeFormatter::new()),
        Box::new(GeminiFormatter::new()),
        Box::new(CodexFormatter::new()),
    ]
}

pub fn formatter_for(id: &str) -> Option<Box<dyn McpFormatter>> {
    formatters().into_iter().find(|f| f.id() == id)
}
```

---

## Formatter Implementations

### Claude Formatter

```rust
// crates/runtime/src/mcp_manager/formatters/claude.rs

pub struct ClaudeFormatter;

impl McpFormatter for ClaudeFormatter {
    fn id(&self) -> &str { "claude" }
    fn name(&self) -> &str { "Claude Code" }

    fn project_config_path(&self, project_root: &Path) -> PathBuf {
        project_root.join(".mcp.json")
    }

    fn global_config_path(&self) -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".claude.json"))
    }

    fn is_installed(&self) -> bool {
        which::which("claude").is_ok()
    }

    fn read(&self, path: &Path) -> Result<ReadResult> {
        if !path.exists() {
            return Ok(ReadResult::empty());
        }

        let content = fs::read_to_string(path)?;
        let raw: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| Error::ParseError {
                path: path.to_path_buf(),
                message: e.to_string(),
            })?;

        let mcp_servers = raw["mcpServers"].as_object()
            .cloned()
            .unwrap_or_default();

        let mut shipwright_servers = vec![];
        let mut user_servers = vec![];

        for (id, server) in &mcp_servers {
            let def = parse_claude_server(id, server)?;
            if server["_shipwright"]["managed"].as_bool() == Some(true) {
                shipwright_servers.push(def);
            } else {
                user_servers.push(def);
            }
        }

        // Preserve all non-mcpServers fields exactly
        let mut other = raw.clone();
        other.as_object_mut().map(|o| o.remove("mcpServers"));

        Ok(ReadResult {
            shipwright_servers,
            user_servers,
            raw_other_fields: other,
        })
    }

    fn format(&self, existing: ReadResult, servers: &[McpServerDef]) -> Result<String> {
        // Start with preserved non-MCP fields
        let mut root = existing.raw_other_fields.clone();
        if !root.is_object() {
            root = serde_json::json!({});
        }

        let mut mcp_servers = serde_json::Map::new();

        // User servers first — preserved exactly, no modification
        for server in &existing.user_servers {
            mcp_servers.insert(server.id.clone(), format_claude_server_user(server));
        }

        // Shipwright servers — always include self, then mode servers
        for server in servers {
            mcp_servers.insert(server.id.clone(), format_claude_server_managed(server));
        }

        root["mcpServers"] = serde_json::Value::Object(mcp_servers);

        Ok(serde_json::to_string_pretty(&root)?)
    }
}

fn format_claude_server_managed(server: &McpServerDef) -> serde_json::Value {
    let mut obj = format_claude_server_base(server);
    // Shipwright marker — how we identify our servers on next read
    obj["_shipwright"] = serde_json::json!({
        "managed": true,
        "version": env!("CARGO_PKG_VERSION"),
    });
    obj
}

fn format_claude_server_base(server: &McpServerDef) -> serde_json::Value {
    match &server.transport {
        McpTransport::Stdio { command, args, env } => {
            let mut obj = serde_json::json!({
                "command": command,
                "args": args,
            });
            if !env.is_empty() {
                obj["env"] = serde_json::to_value(env).unwrap();
            }
            obj
        }
        McpTransport::Http { url, headers, .. } => {
            let mut obj = serde_json::json!({
                "type": "http",
                "url": url,
            });
            if !headers.is_empty() {
                obj["headers"] = serde_json::to_value(headers).unwrap();
            }
            obj
        }
    }
}
```

### Gemini Formatter

The key difference: `mcpServers` is nested inside a larger settings file alongside other Gemini-specific keys. Must preserve all non-MCP content.

```rust
// crates/runtime/src/mcp_manager/formatters/gemini.rs

pub struct GeminiFormatter;

impl McpFormatter for GeminiFormatter {
    fn id(&self) -> &str { "gemini" }
    fn name(&self) -> &str { "Gemini CLI" }

    fn project_config_path(&self, project_root: &Path) -> PathBuf {
        project_root.join(".gemini/settings.json")
    }

    fn global_config_path(&self) -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".gemini/settings.json"))
    }

    fn is_installed(&self) -> bool {
        which::which("gemini").is_ok()
    }

    fn format(&self, existing: ReadResult, servers: &[McpServerDef]) -> Result<String> {
        // Preserve all existing Gemini settings (theme, auth, etc.)
        let mut root = existing.raw_other_fields.clone();
        if !root.is_object() {
            root = serde_json::json!({});
        }

        let mut mcp_servers = serde_json::Map::new();

        // User servers preserved
        for server in &existing.user_servers {
            mcp_servers.insert(server.id.clone(), format_gemini_server_user(server));
        }

        // Shipwright servers with marker
        for server in servers {
            mcp_servers.insert(server.id.clone(), format_gemini_server_managed(server));
        }

        root["mcpServers"] = serde_json::Value::Object(mcp_servers);

        Ok(serde_json::to_string_pretty(&root)?)
    }
}

fn format_gemini_server_managed(server: &McpServerDef) -> serde_json::Value {
    match &server.transport {
        McpTransport::Stdio { command, args, env } => {
            let mut obj = serde_json::json!({
                "command": command,
                "args": args,
                "_shipwright": { "managed": true }
            });
            if !env.is_empty() {
                // Gemini requires explicit env declaration — auto-inherit is NOT supported
                obj["env"] = serde_json::to_value(env).unwrap();
            }
            obj
        }
        McpTransport::Http { url, headers, timeout_ms, .. } => {
            // Gemini uses "httpUrl" not "url" for HTTP servers
            let mut obj = serde_json::json!({
                "httpUrl": url,
                "_shipwright": { "managed": true }
            });
            if !headers.is_empty() {
                obj["headers"] = serde_json::to_value(headers).unwrap();
            }
            if let Some(ms) = timeout_ms {
                obj["timeout"] = serde_json::json!(ms);
            }
            obj
        }
    }
}
```

### Codex Formatter

TOML output. Must handle the `mcp_servers` (underscore) naming exactly. Must preserve non-MCP TOML content.

```rust
// crates/runtime/src/mcp_manager/formatters/codex.rs

pub struct CodexFormatter;

impl McpFormatter for CodexFormatter {
    fn id(&self) -> &str { "codex" }
    fn name(&self) -> &str { "Codex" }

    fn project_config_path(&self, project_root: &Path) -> PathBuf {
        project_root.join(".codex/config.toml")
    }

    fn global_config_path(&self) -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".codex/config.toml"))
    }

    fn is_installed(&self) -> bool {
        which::which("codex").is_ok()
    }

    fn read(&self, path: &Path) -> Result<ReadResult> {
        if !path.exists() {
            return Ok(ReadResult::empty());
        }

        let content = fs::read_to_string(path)?;
        let raw: toml::Value = toml::from_str(&content)
            .map_err(|e| Error::ParseError {
                path: path.to_path_buf(),
                message: format!(
                    "TOML parse error: {}. Note: Codex uses 'mcp_servers' (underscore), not 'mcp-servers'",
                    e
                ),
            })?;

        let mcp_table = raw.get("mcp_servers")
            .and_then(|v| v.as_table())
            .cloned()
            .unwrap_or_default();

        let mut shipwright_servers = vec![];
        let mut user_servers = vec![];

        for (id, server) in &mcp_table {
            let def = parse_codex_server(id, server)?;
            // Codex has no _shipwright marker in the TOML — use a comment convention
            // We track managed servers via Shipwright's own state, not the file
            if is_shipwright_managed_codex(id, server) {
                shipwright_servers.push(def);
            } else {
                user_servers.push(def);
            }
        }

        // Preserve non-mcp_servers TOML content
        let mut other = raw.clone();
        if let Some(table) = other.as_table_mut() {
            table.remove("mcp_servers");
        }

        Ok(ReadResult {
            shipwright_servers,
            user_servers,
            raw_other_fields: serde_json::to_value(&other)?,  // internal repr
        })
    }

    fn format(&self, existing: ReadResult, servers: &[McpServerDef]) -> Result<String> {
        // Reconstruct TOML — start with non-MCP content
        let mut output = String::new();

        // Write non-MCP TOML fields first (model, sandbox, etc.)
        let other: toml::Value = serde_json::from_value(existing.raw_other_fields)?;
        if let toml::Value::Table(table) = &other {
            for (key, value) in table {
                output.push_str(&format!("{} = {}\n", key, toml::to_string(value)?));
            }
        }

        if !output.is_empty() {
            output.push('\n');
        }

        // User-defined MCP servers — preserved exactly
        for server in &existing.user_servers {
            output.push_str(&format_codex_server_toml(&server.id, server, false));
            output.push('\n');
        }

        // Shipwright-managed servers
        // Comment header marks the section for identification on next read
        if !servers.is_empty() {
            output.push_str("# --- Managed by Shipwright ---\n");
            for server in servers {
                output.push_str(&format_codex_server_toml(&server.id, server, true));
                output.push('\n');
            }
            output.push_str("# --- End Shipwright managed ---\n");
        }

        Ok(output)
    }
}

fn format_codex_server_toml(id: &str, server: &McpServerDef, managed: bool) -> String {
    let mut lines = vec![];
    lines.push(format!("[mcp_servers.{}]", id));

    match &server.transport {
        McpTransport::Stdio { command, args, env } => {
            lines.push(format!("command = {:?}", command));
            if !args.is_empty() {
                let args_toml = args.iter()
                    .map(|a| format!("{:?}", a))
                    .collect::<Vec<_>>()
                    .join(", ");
                lines.push(format!("args = [{}]", args_toml));
            }
            if !env.is_empty() {
                let env_toml = env.iter()
                    .map(|(k, v)| format!("{} = {:?}", k, v))
                    .collect::<Vec<_>>()
                    .join(", ");
                lines.push(format!("env = {{ {} }}", env_toml));
            }
        }
        McpTransport::Http { url, bearer_token_env_var, .. } => {
            lines.push(format!("url = {:?}", url));
            if let Some(var) = bearer_token_env_var {
                lines.push(format!("bearer_token_env_var = {:?}", var));
            }
        }
    }

    lines.join("\n")
}

fn is_shipwright_managed_codex(id: &str, _server: &toml::Value) -> bool {
    // Codex TOML doesn't support arbitrary comment-adjacent metadata
    // Track managed server IDs in Shipwright's own state file instead
    // (.ship/mcp_managed_state.toml — which server IDs in each tool we wrote)
    // This function would query that state
    id == "shipwright" // shipwright itself is always ours
    // In practice: check against stored state
}
```

**The Codex marker problem:** Unlike JSON where we can add `_shipwright: {managed: true}` as a field, TOML tables don't have a clean way to add arbitrary metadata. The solution is a comment convention (`# --- Managed by Shipwright ---`) plus a Shipwright-owned state file that tracks which server IDs we wrote into which tool's config.

```toml
# .ship/mcp_managed_state.toml — Shipwright's own record of what it wrote
[claude]
managed_servers = ["shipwright", "github", "postgres"]
last_written = "2026-02-22T10:00:00Z"
last_mode = "backend"

[gemini]
managed_servers = ["shipwright", "github", "postgres"]
last_written = "2026-02-22T10:00:00Z"
last_mode = "backend"

[codex]
managed_servers = ["shipwright", "github", "postgres"]
last_written = "2026-02-22T10:00:00Z"
last_mode = "backend"
```

---

## The Write Pipeline

Same safe pipeline for all three formatters.

```rust
// crates/runtime/src/mcp_manager/pipeline.rs

pub async fn write_config(
    formatter: &dyn McpFormatter,
    project_root: &Path,
    servers: &[McpServerDef],
) -> Result<WriteResult> {

    let config_path = formatter.project_config_path(project_root);

    // 1. Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // 2. Backup if file exists
    let backup_path = if config_path.exists() {
        let backup = config_path.with_extension(
            format!("{}.shipwright-backup", 
                config_path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("bak")
            )
        );
        fs::copy(&config_path, &backup)?;
        Some(backup)
    } else {
        None
    };

    // 3. Read existing (preserves user content)
    let existing = if config_path.exists() {
        formatter.read(&config_path).unwrap_or_else(|e| {
            // Log parse error but don't fail — write fresh
            eprintln!("Warning: could not parse existing config: {}", e);
            ReadResult::empty()
        })
    } else {
        ReadResult::empty()
    };

    // 4. Always inject Shipwright itself first
    let mut all_servers = vec![McpServerDef::shipwright()];
    all_servers.extend_from_slice(servers);

    // 5. Format
    let content = formatter.format(existing, &all_servers)?;

    // 6. Atomic write (temp file + rename)
    let tmp = config_path.with_extension("tmp");
    fs::write(&tmp, &content)?;
    fs::rename(&tmp, &config_path).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        Error::WriteFailed(e.to_string())
    })?;

    // 7. Verify — read back and confirm servers present
    let verify = formatter.read(&config_path)?;
    let written_ids: HashSet<_> = all_servers.iter().map(|s| &s.id).collect();
    let found_ids: HashSet<_> = verify.shipwright_servers.iter()
        .map(|s| &s.id).collect();
    let missing: Vec<_> = written_ids.difference(&found_ids).collect();

    if !missing.is_empty() {
        // Restore backup
        if let Some(ref backup) = backup_path {
            fs::copy(backup, &config_path)?;
        }
        return Err(Error::VerificationFailed {
            tool: formatter.id().to_string(),
            missing: missing.iter().map(|s| s.to_string()).collect(),
        });
    }

    // 8. Update managed state
    update_managed_state(project_root, formatter.id(), &all_servers)?;

    Ok(WriteResult {
        path: config_path,
        backup_path,
        servers_written: all_servers.iter().map(|s| s.id.clone()).collect(),
        servers_preserved: verify.user_servers.iter().map(|s| s.id.clone()).collect(),
        restart_required: formatter.requires_restart(),
    })
}
```

---

## Import

Import reads existing configs from all three tools and normalizes into Shipwright's schema.

```rust
// crates/runtime/src/mcp_manager/import.rs

pub async fn import_existing(
    formatters: &[Box<dyn McpFormatter>],
    project_root: &Path,
) -> Result<ImportResult> {

    let mut found: HashMap<String, ImportedServer> = HashMap::new();

    for formatter in formatters {
        // Check project-scoped config first, then global
        let paths = [
            Some(formatter.project_config_path(project_root)),
            formatter.global_config_path(),
        ];

        for path in paths.iter().flatten() {
            if !path.exists() { continue; }

            let state = match formatter.read(path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Could not read {} config at {:?}: {}", formatter.name(), path, e);
                    continue;
                }
            };

            for server in state.user_servers {
                match found.get_mut(&server.id) {
                    Some(existing) => {
                        existing.found_in.push(formatter.id().to_string());
                        if existing.def.transport != server.transport {
                            existing.conflict = Some(format!(
                                "Different config in {} vs previous tool",
                                formatter.name()
                            ));
                        }
                    }
                    None => {
                        found.insert(server.id.clone(), ImportedServer {
                            def: server.clone(),
                            found_in: vec![formatter.id().to_string()],
                            suggested_modes: suggest_modes(&server),
                            conflict: None,
                        });
                    }
                }
            }
        }
    }

    // Skip "shipwright" — we always manage that ourselves
    found.remove("shipwright");

    Ok(ImportResult {
        servers: found.into_values().collect(),
    })
}

fn suggest_modes(server: &McpServerDef) -> Vec<String> {
    let id = server.id.to_lowercase();
    let mut modes = vec![];

    if id.contains("github") || id.contains("git") || id.contains("linear") {
        modes.push("execution".to_string());
    }
    if id.contains("figma") || id.contains("design") || id.contains("storybook") {
        modes.push("planning".to_string());
    }
    if id.contains("postgres") || id.contains("mysql") || id.contains("db") || id.contains("database") {
        modes.push("backend".to_string());
    }
    if id.contains("notion") || id.contains("jira") || id.contains("linear") {
        modes.push("planning".to_string());
    }
    if modes.is_empty() {
        modes.push("all".to_string());
    }
    modes.dedup();
    modes
}
```

---

## Mode Switching

On mode switch, write updated configs to all managed tools simultaneously.

```rust
// crates/runtime/src/mcp_manager/manager.rs

pub async fn switch_mode(
    &self,
    mode_id: &str,
    project_root: &Path,
) -> Result<ModeSwitchReport> {

    let mode = self.modes.get(mode_id)
        .ok_or(Error::UnknownMode(mode_id.to_string()))?;

    let servers = self.resolve_servers(mode)?;
    let mut results = vec![];

    for formatter in &self.formatters {
        // Only write to tools that are managed
        if !self.is_managed(formatter.id(), project_root) {
            continue;
        }

        let result = write_config(formatter.as_ref(), project_root, &servers).await;

        results.push(ToolResult {
            tool: formatter.id().to_string(),
            tool_name: formatter.name().to_string(),
            success: result.is_ok(),
            error: result.err().map(|e| e.to_string()),
            restart_required: formatter.requires_restart(),
        });
    }

    // Update runtime MCP server regardless of file writes
    self.mcp_registry.set_active_mode(mode_id).await?;
    self.mcp_registry.broadcast_tools_changed().await;

    Ok(ModeSwitchReport {
        mode_id: mode_id.to_string(),
        mode_name: mode.name.clone(),
        results,
        any_restart_required: results.iter().any(|r| r.restart_required && r.success),
    })
}

fn resolve_servers(&self, mode: &Mode) -> Result<Vec<McpServerDef>> {
    mode.mcp_servers.iter()
        .map(|s| self.resolve_server_ref(s))
        .collect()
}

fn resolve_server_ref(&self, server_ref: &ModeServerRef) -> Result<McpServerDef> {
    // Look up full server definition from project config
    // Expand env vars in command and env values
    let def = self.server_registry.get(&server_ref.id)
        .ok_or(Error::UnknownServer(server_ref.id.clone()))?;

    Ok(McpServerDef {
        id: server_ref.id.clone(),
        transport: expand_env_vars(&def.transport)?,
        enabled: true,
        shipwright_managed: true,
    })
}
```

---

## CLI Commands

```bash
# Status
shipwright tools status                  # Show detected tools, managed state, active mode

# Setup
shipwright tools manage claude           # Start managing Claude Code config
shipwright tools manage gemini           # Start managing Gemini CLI config
shipwright tools manage codex            # Start managing Codex config
shipwright tools manage --all            # Manage all detected tools
shipwright tools unmanage claude         # Stop managing, leave config as-is

# Import
shipwright tools import                  # Interactive import from existing configs
shipwright tools import --dry-run        # Preview what would be imported, no changes

# Mode switching
shipwright mode switch planning          # Switch mode, update all managed tools
shipwright mode switch execution
shipwright mode current                  # Show active mode and active servers

# One-time export (no ongoing management)
shipwright tools export --target claude  # Write .mcp.json from current mode
shipwright tools export --target gemini  # Write .gemini/settings.json
shipwright tools export --target codex   # Write .codex/config.toml
shipwright tools export --target all     # Write all three
shipwright tools export --dry-run        # Preview output without writing

# Recovery
shipwright tools restore claude          # Restore from backup
shipwright tools restore --all
```

---

## Test Matrix

Every formatter must pass every test. Use `tempdir` for all file system operations.

```rust
#[cfg(test)]
mod tests {

    // ── Format round-trip ─────────────────────────────────────────
    // Write servers → read back → verify all present
    #[test] fn claude_round_trip_stdio_server() {}
    #[test] fn claude_round_trip_http_server() {}
    #[test] fn gemini_round_trip_stdio_server() {}
    #[test] fn gemini_round_trip_http_server() {}
    #[test] fn codex_round_trip_stdio_server() {}
    #[test] fn codex_round_trip_http_server() {}

    // ── User content preservation ─────────────────────────────────
    // Non-Shipwright servers and fields must survive a write
    #[test] fn claude_preserves_user_servers() {}
    #[test] fn gemini_preserves_user_servers() {}
    #[test] fn gemini_preserves_theme_and_auth_fields() {}
    #[test] fn codex_preserves_user_servers() {}
    #[test] fn codex_preserves_non_mcp_toml_keys() {}

    // ── Shipwright always present ─────────────────────────────────
    #[test] fn claude_shipwright_always_injected() {}
    #[test] fn gemini_shipwright_always_injected() {}
    #[test] fn codex_shipwright_always_injected() {}

    // ── Empty / new file ─────────────────────────────────────────
    #[test] fn claude_writes_to_nonexistent_file() {}
    #[test] fn gemini_creates_parent_dir_if_missing() {}
    #[test] fn codex_creates_parent_dir_if_missing() {}

    // ── Parse error handling ──────────────────────────────────────
    #[test] fn claude_handles_malformed_json_gracefully() {}
    #[test] fn gemini_handles_malformed_json_gracefully() {}
    #[test] fn codex_handles_malformed_toml_gracefully() {}
    #[test] fn codex_detects_mcp_underscore_vs_hyphen() {}  // common mistake

    // ── Atomic write ─────────────────────────────────────────────
    #[test] fn all_formatters_write_atomically() {}
    #[test] fn backup_created_before_write() {}
    #[test] fn backup_restored_on_verification_failure() {}

    // ── Managed state tracking ───────────────────────────────────
    #[test] fn managed_state_updated_after_write() {}
    #[test] fn managed_state_used_to_identify_codex_servers() {}

    // ── Mode switch ───────────────────────────────────────────────
    #[test] fn mode_switch_updates_all_managed_tools() {}
    #[test] fn mode_switch_skips_unmanaged_tools() {}
    #[test] fn mode_switch_partial_failure_reports_per_tool() {}
    #[test] fn mode_switch_broadcasts_mcp_tools_changed() {}

    // ── Import ───────────────────────────────────────────────────
    #[test] fn import_finds_claude_project_config() {}
    #[test] fn import_finds_gemini_global_config() {}
    #[test] fn import_finds_codex_project_config() {}
    #[test] fn import_deduplicates_across_tools() {}
    #[test] fn import_detects_transport_conflicts() {}
    #[test] fn import_skips_shipwright_managed_servers() {}
    #[test] fn import_suggests_modes_by_server_name() {}

    // ── Env var handling ─────────────────────────────────────────
    #[test] fn env_vars_preserved_in_output_as_references() {} // $VAR not expanded
    #[test] fn gemini_env_vars_in_env_field_not_headers() {}   // Gemini quirk
}
```

---

## UX: Key Warnings to Surface

**Claude Code approval prompt** — Claude Code will prompt the user for approval before using project-scoped servers from `.mcp.json` on first use. This is a Claude Code security feature Shipwright cannot disable. Surface this as an expected step, not an error.

**Gemini env vars** — Unlike Claude and Codex, Gemini does not auto-inherit shell env vars for MCP servers. Every variable must be explicitly declared in the server's `env` property. When writing a Gemini config, warn if a server uses env vars in `headers` for HTTP transport (Gemini doesn't expand them there).

**Codex section name** — `mcp_servers` with underscore. If parsing fails, check for `mcp-servers` (hyphen) in the file and offer to fix it automatically.

**Codex shared config** — The Codex CLI and IDE extension share `~/.codex/config.toml`. A TOML syntax error breaks both. Always validate TOML before writing and show a clear error if the existing file has invalid TOML before Shipwright touches it.

**Restart required** — All three tools require a restart after config changes. Surface this clearly in both CLI and GUI output. Don't let users think the change hasn't applied when it just needs a restart.

---

## Implementation Order

**Day 1-2:** Types + Claude formatter + write pipeline + round-trip test
**Day 3:** Gemini formatter + tests  
**Day 4:** Codex formatter + TOML handling + tests (hardest)  
**Day 5:** Managed state file + import scanner  
**Day 6:** Mode switch coordinator + CLI commands  
**Day 7:** GUI — status view + manage flow + mode switch feedback  
**Day 8:** Import UI + conflict resolution  
**Day 9:** Error messages + edge case polish  
**Day 10:** Full integration test pass

---

## Document History

| Version | Date | Changes |
|---------|------|---------|
| 0.1 | 2026-02-22 | Alpha scope — Claude Code, Gemini CLI, Codex only |
