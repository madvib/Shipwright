//! Codex CLI decompiler — parse `.codex/config.toml` and `AGENTS.md` into a [`ProjectLibrary`].

use std::collections::HashMap;
use std::path::Path;

use serde_json::Value as Json;

use crate::ProjectLibrary;
use crate::types::{McpServerConfig, McpServerType, Rule};

/// Known top-level keys in `.codex/config.toml` that map to structured Ship fields.
const KNOWN_CONFIG_KEYS: &[&str] = &[
    "model",
    "approval_policy",
    "sandbox_mode",
    "model_reasoning_effort",
    "shell_environment_policy",
    "notify",
    "agents",
    "mcp_servers",
];

/// Parse Codex CLI native config files and produce a partial [`ProjectLibrary`].
///
/// Reads (if present):
/// - `.codex/config.toml` → model, codex settings, MCP servers, provider_defaults
/// - `AGENTS.md` → rules
pub fn decompile_codex(project_root: &Path) -> ProjectLibrary {
    let mut library = ProjectLibrary::default();

    // ── .codex/config.toml ───────────────────────────────────────────────────
    let config_path = project_root.join(".codex").join("config.toml");
    if let Ok(content) = std::fs::read_to_string(&config_path)
        && let Ok(table) = content.parse::<toml::Table>()
    {
        parse_codex_config(&mut library, &table);
    }

    // ── AGENTS.md ────────────────────────────────────────────────────────────
    let agents_md = project_root.join("AGENTS.md");
    if let Ok(content) = std::fs::read_to_string(&agents_md) {
        let trimmed = content.trim();
        if !trimmed.is_empty() {
            library.rules.push(Rule {
                file_name: "AGENTS.md".to_string(),
                content: trimmed.to_string(),
                always_apply: true,
                globs: vec![],
                description: None,
            });
        }
    }

    library
}

fn parse_codex_config(library: &mut ProjectLibrary, table: &toml::Table) {
    // ── Model ────────────────────────────────────────────────────────────────
    if let Some(toml::Value::String(m)) = table.get("model") {
        library.model = Some(m.clone());
    }

    // ── Approval policy → codex_approval_policy ──────────────────────────────
    if let Some(toml::Value::String(p)) = table.get("approval_policy") {
        library.codex_approval_policy = Some(reverse_approval_policy(p));
    }

    // ── Sandbox mode → codex_sandbox ─────────────────────────────────────────
    if let Some(toml::Value::String(s)) = table.get("sandbox_mode") {
        library.codex_sandbox = Some(reverse_sandbox(s));
    }

    // ── Reasoning effort ─────────────────────────────────────────────────────
    if let Some(toml::Value::String(e)) = table.get("model_reasoning_effort") {
        library.codex_reasoning_effort = Some(e.clone());
    }

    // ── Shell environment policy ─────────────────────────────────────────────
    if let Some(toml::Value::String(p)) = table.get("shell_environment_policy") {
        library.codex_shell_env_policy = Some(p.clone());
    }

    // ── Notify ───────────────────────────────────────────────────────────────
    if let Some(v) = table.get("notify")
        && let Some(json) = toml_to_json(v)
    {
        library.codex_notify = Some(json);
    }

    // ── [agents] table ───────────────────────────────────────────────────────
    if let Some(toml::Value::Table(agents)) = table.get("agents") {
        if let Some(toml::Value::Integer(v)) = agents.get("max_threads") {
            library.codex_max_threads = Some(*v as u32);
        }
        if let Some(toml::Value::Integer(v)) = agents.get("max_depth") {
            library.codex_max_depth = Some(*v as u32);
        }
        if let Some(toml::Value::Integer(v)) = agents.get("job_max_runtime_seconds") {
            library.codex_job_max_runtime_seconds = Some(*v as u64);
        }
    }

    // ── [mcp_servers.*] tables → MCP servers ─────────────────────────────────
    if let Some(toml::Value::Table(mcp)) = table.get("mcp_servers") {
        for (id, entry) in mcp {
            if let toml::Value::Table(t) = entry
                && let Some(server) = parse_codex_mcp_server(id, t)
            {
                library.mcp_servers.push(server);
            }
        }
    }

    // ── Provider defaults — everything we don't recognize ────────────────────
    let mut extra = serde_json::Map::new();
    for (k, v) in table {
        if !KNOWN_CONFIG_KEYS.contains(&k.as_str())
            && let Some(json) = toml_to_json(v)
        {
            extra.insert(k.clone(), json);
        }
    }
    if !extra.is_empty() {
        library
            .provider_defaults
            .insert("codex".to_string(), Json::Object(extra));
    }
}

fn parse_codex_mcp_server(id: &str, table: &toml::Table) -> Option<McpServerConfig> {
    let command = table
        .get("command")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let url = table.get("url").and_then(|v| v.as_str()).map(String::from);

    let server_type = if url.is_some() && command.is_empty() {
        McpServerType::Sse
    } else {
        McpServerType::Stdio
    };

    let args = table
        .get("args")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let env: HashMap<String, String> = table
        .get("env")
        .and_then(|v| v.as_table())
        .map(|t| {
            t.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default();

    let timeout_secs = table
        .get("startup_timeout_sec")
        .and_then(|v| v.as_integer())
        .map(|t| t as u32);

    let enabled_tools = toml_string_array(table.get("enabled_tools"));
    let disabled_tools = toml_string_array(table.get("disabled_tools"));

    Some(McpServerConfig {
        id: id.to_string(),
        name: id.to_string(),
        command,
        args,
        env,
        scope: "project".to_string(),
        server_type,
        url,
        disabled: false,
        timeout_secs,
        codex_enabled_tools: enabled_tools,
        codex_disabled_tools: disabled_tools,
        gemini_trust: None,
        gemini_include_tools: vec![],
        gemini_exclude_tools: vec![],
        gemini_timeout_ms: None,
        cursor_env_file: None,
    })
}

// ── Reverse translations ─────────────────────────────────────────────────────

fn reverse_sandbox(val: &str) -> String {
    match val {
        "danger-full-internet" => "full",
        "network-disabled" => "network-only",
        "disabled" => "off",
        other => other,
    }
    .to_string()
}

fn reverse_approval_policy(val: &str) -> String {
    match val {
        "suggest" => "default",
        "auto-edit" => "auto_edit",
        "full-auto" => "plan",
        other => other,
    }
    .to_string()
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn toml_string_array(val: Option<&toml::Value>) -> Vec<String> {
    val.and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn toml_to_json(v: &toml::Value) -> Option<Json> {
    match v {
        toml::Value::String(s) => Some(Json::String(s.clone())),
        toml::Value::Integer(i) => Some(Json::Number((*i).into())),
        toml::Value::Float(f) => serde_json::Number::from_f64(*f).map(Json::Number),
        toml::Value::Boolean(b) => Some(Json::Bool(*b)),
        toml::Value::Datetime(d) => Some(Json::String(d.to_string())),
        toml::Value::Array(arr) => {
            let items: Vec<Json> = arr.iter().filter_map(toml_to_json).collect();
            Some(Json::Array(items))
        }
        toml::Value::Table(t) => {
            let mut map = serde_json::Map::new();
            for (k, val) in t {
                if let Some(json) = toml_to_json(val) {
                    map.insert(k.clone(), json);
                }
            }
            Some(Json::Object(map))
        }
    }
}
