use crate::resolve::ResolvedConfig;
use crate::types::McpServerType;
use serde_json::Value as Json;

// ── Sandbox translation ───────────────────────────────────────────────────────

/// Translate our internal sandbox value to Codex's `sandbox_mode` value.
///
/// Codex uses `sandbox_mode` (not `sandbox`) and different value names:
/// - `"full"` → `"danger-full-internet"`
/// - `"network-only"` → `"network-disabled"`
/// - `"off"` → `"disabled"`
fn translate_sandbox(val: &str) -> &str {
    match val {
        "full" => "danger-full-internet",
        "network-only" => "network-disabled",
        "off" => "disabled",
        // Pass through already-translated or unknown values verbatim.
        other => other,
    }
}

/// Translate our internal approval policy to Codex's `approval_policy` value.
///
/// - `"default"` → `"suggest"`
/// - `"auto_edit"` → `"auto-edit"`
/// - `"plan"` → `"full-auto"`
fn translate_approval_policy(val: &str) -> &str {
    match val {
        "default" => "suggest",
        "auto_edit" => "auto-edit",
        "plan" => "full-auto",
        other => other,
    }
}

// ── Main build function ───────────────────────────────────────────────────────

/// Build the `.codex/config.toml` content from a resolved config.
///
/// Source: https://developers.openai.com/codex/config-basic
/// Codex uses TOML. Key fields: `model`, `approval_policy`, `sandbox_mode`,
/// `model_reasoning_effort`, `shell_environment_policy`, `notify`, `[agents]`, `[mcp_servers.*]`.
/// Returns `None` if serialisation fails (should not happen in practice).
pub(super) fn build_codex_config_patch(resolved: &ResolvedConfig) -> Option<String> {
    let servers = &resolved.mcp_servers;
    let model = resolved.model.as_deref();
    let sandbox = resolved.codex_sandbox.as_deref();

    let mut mcp = toml::Table::new();

    // Ship server always first.
    let mut ship_entry = toml::Table::new();
    ship_entry.insert("command".into(), toml::Value::String("ship".into()));
    ship_entry.insert(
        "args".into(),
        toml::Value::Array(vec![
            toml::Value::String("mcp".into()),
            toml::Value::String("serve".into()),
        ]),
    );
    mcp.insert("ship".into(), toml::Value::Table(ship_entry));

    for s in servers {
        if s.disabled || s.id == "ship" {
            continue;
        }
        let mut entry = toml::Table::new();
        match s.server_type {
            McpServerType::Stdio => {
                entry.insert("command".into(), toml::Value::String(s.command.clone()));
                if !s.args.is_empty() {
                    entry.insert(
                        "args".into(),
                        toml::Value::Array(
                            s.args
                                .iter()
                                .map(|a| toml::Value::String(a.clone()))
                                .collect(),
                        ),
                    );
                }
                if !s.env.is_empty() {
                    let mut env_table = toml::Table::new();
                    for (k, v) in &s.env {
                        env_table.insert(k.clone(), toml::Value::String(v.clone()));
                    }
                    entry.insert("env".into(), toml::Value::Table(env_table));
                }
            }
            McpServerType::Sse | McpServerType::Http => {
                if let Some(url) = &s.url {
                    entry.insert("url".into(), toml::Value::String(url.clone()));
                }
            }
        }
        if let Some(t) = s.timeout_secs {
            entry.insert("startup_timeout_sec".into(), toml::Value::Integer(t as i64));
        }
        // Per-server tool filters.
        if !s.codex_enabled_tools.is_empty() {
            entry.insert(
                "enabled_tools".into(),
                toml::Value::Array(
                    s.codex_enabled_tools
                        .iter()
                        .map(|t| toml::Value::String(t.clone()))
                        .collect(),
                ),
            );
        }
        if !s.codex_disabled_tools.is_empty() {
            entry.insert(
                "disabled_tools".into(),
                toml::Value::Array(
                    s.codex_disabled_tools
                        .iter()
                        .map(|t| toml::Value::String(t.clone()))
                        .collect(),
                ),
            );
        }
        mcp.insert(s.id.clone(), toml::Value::Table(entry));
    }

    let mut root = toml::Table::new();

    // Top-level model.
    if let Some(m) = model {
        root.insert("model".into(), toml::Value::String(m.to_string()));
    }

    // Phase 1B: correct key is `sandbox_mode`, values translated.
    if let Some(s) = sandbox {
        root.insert(
            "sandbox_mode".into(),
            toml::Value::String(translate_sandbox(s).to_string()),
        );
    }

    // Approval policy.
    if let Some(p) = resolved.codex_approval_policy.as_deref() {
        root.insert(
            "approval_policy".into(),
            toml::Value::String(translate_approval_policy(p).to_string()),
        );
    }

    // Reasoning effort.
    if let Some(e) = resolved.codex_reasoning_effort.as_deref() {
        root.insert(
            "model_reasoning_effort".into(),
            toml::Value::String(e.to_string()),
        );
    }

    // Shell environment policy.
    if let Some(p) = resolved.codex_shell_env_policy.as_deref() {
        root.insert(
            "shell_environment_policy".into(),
            toml::Value::String(p.to_string()),
        );
    }

    // Notify (JSON value → TOML).
    if let Some(notify) = &resolved.codex_notify
        && let Some(toml_val) = json_to_toml(notify)
    {
        root.insert("notify".into(), toml_val);
    }

    // [agents] table.
    let has_agents = resolved.codex_max_threads.is_some()
        || resolved.codex_max_depth.is_some()
        || resolved.codex_job_max_runtime_seconds.is_some();
    if has_agents {
        let mut agents = toml::Table::new();
        if let Some(v) = resolved.codex_max_threads {
            agents.insert("max_threads".into(), toml::Value::Integer(v as i64));
        }
        if let Some(v) = resolved.codex_max_depth {
            agents.insert("max_depth".into(), toml::Value::Integer(v as i64));
        }
        if let Some(v) = resolved.codex_job_max_runtime_seconds {
            agents.insert(
                "job_max_runtime_seconds".into(),
                toml::Value::Integer(v as i64),
            );
        }
        root.insert("agents".into(), toml::Value::Table(agents));
    }

    // MCP servers table (always present — ship server).
    root.insert("mcp_servers".into(), toml::Value::Table(mcp));

    // settings_extra: merge verbatim after typed fields.
    if let Some(extra) = &resolved.codex_settings_extra
        && let Some(obj) = extra.as_object()
    {
        for (k, v) in obj {
            if let Some(toml_val) = json_to_toml(v) {
                root.insert(k.clone(), toml_val);
            }
        }
    }

    toml::to_string(&root).ok()
}

// ── Codex hooks.json ─────────────────────────────────────────────────────────

/// Build the `.codex/hooks.json` content.
///
/// Codex fires hooks for all four lifecycle events. The `SessionStart` hook
/// runs `ship hook session-start` (no stdin). Tool hooks run `ship hook
/// before-tool` / `ship hook after-tool` with a JSON payload on stdin.
/// `Stop` maps to `ship hook session-end`.
pub(super) fn build_codex_hooks_json() -> Json {
    serde_json::json!({
        "hooks": [
            {
                "event": "SessionStart",
                "hooks": [{ "type": "command", "command": "ship hook session-start" }]
            },
            {
                "event": "PreToolUse",
                "hooks": [{ "type": "command", "command": "ship hook before-tool" }]
            },
            {
                "event": "PostToolUse",
                "hooks": [{ "type": "command", "command": "ship hook after-tool" }]
            },
            {
                "event": "Stop",
                "hooks": [{ "type": "command", "command": "ship hook session-end" }]
            }
        ]
    })
}

// ── JSON → TOML value conversion ─────────────────────────────────────────────

fn json_to_toml(v: &serde_json::Value) -> Option<toml::Value> {
    match v {
        serde_json::Value::Null => None,
        serde_json::Value::Bool(b) => Some(toml::Value::Boolean(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Some(toml::Value::Integer(i))
            } else {
                n.as_f64().map(toml::Value::Float)
            }
        }
        serde_json::Value::String(s) => Some(toml::Value::String(s.clone())),
        serde_json::Value::Array(arr) => {
            let items: Vec<toml::Value> = arr.iter().filter_map(json_to_toml).collect();
            Some(toml::Value::Array(items))
        }
        serde_json::Value::Object(obj) => {
            let mut table = toml::Table::new();
            for (k, val) in obj {
                if let Some(toml_val) = json_to_toml(val) {
                    table.insert(k.clone(), toml_val);
                }
            }
            Some(toml::Value::Table(table))
        }
    }
}
