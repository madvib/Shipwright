use std::collections::HashMap;

use serde_json::Value as Json;
use toml;

use crate::types::{HookConfig, HookTrigger, McpServerConfig, McpServerType, Permissions, Rule, Skill};
use crate::resolve::ResolvedConfig;

// ─── Provider registry ────────────────────────────────────────────────────────
// Authoritative path reference, support matrix, and compatibility dates:
// → crates/core/compiler/PROVIDERS.md
// Update that file before changing any ProviderDescriptor field.

/// How MCP servers are keyed in the target config file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpKey {
    /// `"mcpServers"` — Claude, Gemini
    McpServers,
    /// `"mcp_servers"` — Codex/OpenAI
    McpServersUnderscored,
}

impl McpKey {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::McpServers => "mcpServers",
            Self::McpServersUnderscored => "mcp_servers",
        }
    }
}

/// Where the context / system-instructions file is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextFile {
    /// `CLAUDE.md` — Claude Code
    ClaudeMd,
    /// `GEMINI.md` — Gemini CLI
    GeminiMd,
    /// `AGENTS.md` — Codex, Roo, Amp, Goose
    AgentsMd,
    /// Provider does not use a context file
    None,
}

impl ContextFile {
    pub fn file_name(self) -> Option<&'static str> {
        match self {
            Self::ClaudeMd => Some("CLAUDE.md"),
            Self::GeminiMd => Some("GEMINI.md"),
            Self::AgentsMd => Some("AGENTS.md"),
            Self::None => None,
        }
    }
}

/// Where native skill files are written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillsDir {
    /// `.claude/skills/<id>/SKILL.md`
    Claude,
    /// `.agents/skills/<id>/SKILL.md` (Gemini CLI, Cursor fallback, universal)
    Gemini,
    /// `.agents/skills/<id>/SKILL.md`
    Agents,
    /// `.cursor/skills/<id>/SKILL.md`
    Cursor,
    None,
}

impl SkillsDir {
    pub fn base_path(self) -> Option<&'static str> {
        match self {
            Self::Claude => Some(".claude/skills"),
            Self::Gemini => Some(".agents/skills"),
            Self::Agents => Some(".agents/skills"),
            Self::Cursor => Some(".cursor/skills"),
            Self::None => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProviderDescriptor {
    pub id: &'static str,
    pub name: &'static str,
    pub mcp_key: McpKey,
    pub context_file: ContextFile,
    pub skills_dir: SkillsDir,
    /// Whether to emit `"type"` field in MCP server entries.
    /// Claude and Cursor: false (no type field).
    /// Gemini and Codex: false — transport is inferred from field presence
    ///   (command → stdio, url → SSE, httpUrl → HTTP).
    pub emit_type_field: bool,
    /// Field name used for SSE transport URL entries ("url" for most providers).
    pub sse_url_field: &'static str,
    /// Field name used for streamable HTTP transport URL entries.
    /// Gemini uses "httpUrl"; others use "url".
    pub http_url_field: &'static str,
    /// Project-relative path where the MCP config file is written.
    /// `None` when the MCP config is merged into a larger settings file.
    pub mcp_config_path: Option<&'static str>,
}

static PROVIDERS: &[ProviderDescriptor] = &[
    ProviderDescriptor {
        id: "claude",
        name: "Claude Code",
        mcp_key: McpKey::McpServers,
        context_file: ContextFile::ClaudeMd,
        skills_dir: SkillsDir::Claude,
        emit_type_field: false,
        sse_url_field: "url",
        http_url_field: "url",
        // Project-level MCP for Claude Code lives in .mcp.json
        mcp_config_path: Some(".mcp.json"),
    },
    ProviderDescriptor {
        id: "gemini",
        name: "Gemini CLI",
        mcp_key: McpKey::McpServers,
        context_file: ContextFile::GeminiMd,
        skills_dir: SkillsDir::Gemini,
        // Source: https://geminicli.com/docs/tools/mcp-server/
        // No "type" field — transport inferred from field presence.
        emit_type_field: false,
        // SSE → "url", streamable HTTP → "httpUrl"
        sse_url_field: "url",
        http_url_field: "httpUrl",
        // MCP is nested under mcpServers inside settings.json (not a separate file)
        mcp_config_path: Some(".gemini/settings.json"),
    },
    ProviderDescriptor {
        id: "codex",
        name: "OpenAI Codex",
        // Source: https://developers.openai.com/codex/mcp
        // Codex MCP config is TOML ([mcp_servers.<name>] tables in ~/.codex/config.toml).
        // The mcp_key here reflects the TOML table key; the mcp_servers JSON output
        // is a known limitation — proper TOML serialisation is tracked as future work.
        mcp_key: McpKey::McpServersUnderscored,
        context_file: ContextFile::AgentsMd,
        skills_dir: SkillsDir::Agents,
        // No "type" field in Codex TOML MCP entries either.
        emit_type_field: false,
        sse_url_field: "url",
        http_url_field: "url",
        mcp_config_path: Some(".codex/config.toml"),
    },
    ProviderDescriptor {
        id: "cursor",
        name: "Cursor",
        // Source: https://cursor.com/docs/context/skills
        mcp_key: McpKey::McpServers,
        // Cursor uses per-file .mdc rules in .cursor/rules/ — not a single context file
        context_file: ContextFile::None,
        skills_dir: SkillsDir::Cursor,
        emit_type_field: false,
        sse_url_field: "url",
        http_url_field: "url",
        mcp_config_path: Some(".cursor/mcp.json"),
    },
];

pub fn get_provider(id: &str) -> Option<&'static ProviderDescriptor> {
    PROVIDERS.iter().find(|p| p.id == id)
}

pub fn list_providers() -> &'static [ProviderDescriptor] {
    PROVIDERS
}

// ─── Output ───────────────────────────────────────────────────────────────────

/// The content generated by the compiler for a given provider.
/// All values are strings ready to write; no filesystem access required.
#[derive(Debug, Clone, Default)]
pub struct CompileOutput {
    /// MCP server entries to inject into the target config file.
    /// Key = provider's `mcp_key` value, value = JSON object mapping server ID → entry.
    pub mcp_servers: Json,

    /// Project-relative path where the MCP config file should be written,
    /// e.g. `".mcp.json"` for Claude or `".cursor/mcp.json"` for Cursor.
    /// `None` when the provider merges MCP into a larger settings file.
    pub mcp_config_path: Option<&'static str>,

    /// Context file content (CLAUDE.md / GEMINI.md / AGENTS.md).
    /// `None` if the provider has no context file or there is no content to emit.
    pub context_content: Option<String>,

    /// Skill files: path relative to project root → file content.
    /// e.g. `".claude/skills/my-skill/SKILL.md" → "---\nname: my-skill\n..."`.
    pub skill_files: HashMap<String, String>,

    /// Claude settings patch: `permissions`, `hooks`, agent limits.
    /// Only populated for the `claude` provider.
    pub claude_settings_patch: Option<Json>,

    /// Codex config patch: `[mcp_servers.<id>]` TOML tables.
    /// Only populated for the `codex` provider.
    /// Merge into `.codex/config.toml`.
    pub codex_config_patch: Option<String>,

    /// Gemini settings patch: `hooks` section for `.gemini/settings.json`.
    /// Only populated for the `gemini` provider when hooks are present.
    /// Merge alongside `mcpServers` in `.gemini/settings.json`.
    pub gemini_settings_patch: Option<Json>,

    /// Gemini policy file content for `.gemini/policies/ship.toml`.
    /// Only populated for the `gemini` provider when permissions are non-default.
    /// Decisions: `allow`, `deny`, `ask_user`. Tool names: `shell`, `mcp`,
    /// `file_read`, `file_write`, `web_fetch`.
    pub gemini_policy_patch: Option<String>,

    /// Cursor hooks file: full `.cursor/hooks.json` content.
    /// Only populated for the `cursor` provider when hooks are present.
    pub cursor_hooks_patch: Option<Json>,

    /// Cursor permissions: `{ "version": 1, "permissions": { "allow": [...], "deny": [...] } }`
    /// Written to `.cursor/cli.json` (project) or `~/.cursor/cli-config.json` (global).
    /// Applies to both Cursor IDE agents and the Cursor CLI.
    /// Only populated when permissions are non-default. Never emits a permissive allow-all
    /// unless the user explicitly supplies the typed wildcard patterns.
    pub cursor_cli_permissions: Option<Json>,

    /// Per-file rule output for providers that use individual rule files.
    /// Key = relative path (e.g. `.cursor/rules/style.mdc`), value = file content.
    /// Populated for Cursor (per-file .mdc). Other providers use `context_content`.
    pub rule_files: HashMap<String, String>,
}

// ─── Main entry point ─────────────────────────────────────────────────────────

/// Compile a resolved agent config into provider-ready content.
/// Returns `None` if the provider ID is not recognised.
pub fn compile(resolved: &ResolvedConfig, provider_id: &str) -> Option<CompileOutput> {
    let desc = get_provider(provider_id)?;
    let mut out = CompileOutput::default();

    out.mcp_servers = build_mcp_servers(desc, &resolved.mcp_servers);
    out.mcp_config_path = desc.mcp_config_path;
    out.context_content = build_context_content(desc, resolved);
    out.skill_files = build_skill_files(desc, &resolved.skills);

    if provider_id == "claude" {
        out.claude_settings_patch =
            build_claude_settings_patch(&resolved.permissions, &resolved.hooks, resolved.model.as_deref());
    }

    if provider_id == "codex" {
        out.codex_config_patch = build_codex_config_patch(&resolved.mcp_servers);
    }

    if provider_id == "gemini" {
        out.gemini_settings_patch = build_gemini_settings_patch(&resolved.hooks);
        out.gemini_policy_patch = build_gemini_policy_patch(&resolved.permissions);
    }

    if provider_id == "cursor" {
        out.rule_files = build_cursor_rule_files(&resolved.rules);
        out.cursor_hooks_patch = build_cursor_hooks_patch(&resolved.hooks);
        out.cursor_cli_permissions = build_cursor_cli_permissions(&resolved.permissions);
    }

    Some(out)
}

// ─── MCP server serialisation ─────────────────────────────────────────────────

fn build_mcp_servers(desc: &ProviderDescriptor, servers: &[McpServerConfig]) -> Json {
    let mut map = serde_json::Map::new();

    // Ship's own self-hosted MCP server is always injected first.
    map.insert(
        "ship".to_string(),
        ship_server_entry(desc.emit_type_field),
    );

    for s in servers {
        if s.disabled {
            continue;
        }
        map.insert(s.id.clone(), server_entry(desc, s));
    }

    Json::Object(map)
}

fn server_entry(desc: &ProviderDescriptor, s: &McpServerConfig) -> Json {
    let mut entry = serde_json::json!({});
    match s.server_type {
        McpServerType::Stdio => {
            entry["command"] = Json::String(s.command.clone());
            if !s.args.is_empty() {
                entry["args"] = serde_json::json!(s.args);
            }
            if !s.env.is_empty() {
                entry["env"] = serde_json::json!(s.env);
            }
            if desc.emit_type_field {
                entry["type"] = Json::String("stdio".to_string());
            }
        }
        McpServerType::Sse => {
            if let Some(url) = &s.url {
                entry[desc.sse_url_field] = Json::String(url.clone());
            }
            if desc.emit_type_field {
                entry["type"] = Json::String("sse".to_string());
            }
        }
        McpServerType::Http => {
            if let Some(url) = &s.url {
                entry[desc.http_url_field] = Json::String(url.clone());
            }
            if desc.emit_type_field {
                entry["type"] = Json::String("http".to_string());
            }
        }
    }
    if let Some(t) = s.timeout_secs {
        entry["startup_timeout_sec"] = serde_json::json!(t);
    }
    entry
}

fn ship_server_entry(emit_type: bool) -> Json {
    let mut e = serde_json::json!({
        "command": "ship",
        "args": ["mcp", "serve"]
    });
    if emit_type {
        e["type"] = Json::String("stdio".to_string());
    }
    e
}

// ─── Context file content ─────────────────────────────────────────────────────

fn build_context_content(desc: &ProviderDescriptor, resolved: &ResolvedConfig) -> Option<String> {
    if desc.context_file == ContextFile::None {
        return None;
    }

    let mut parts: Vec<String> = Vec::new();

    // Rules — skip blank content so all-empty rules don't produce a context file
    for rule in &resolved.rules {
        let trimmed = rule.content.trim().to_string();
        if !trimmed.is_empty() {
            parts.push(trimmed);
        }
    }

    // Mode notice
    if let Some(mode) = &resolved.active_mode {
        parts.push(format!("<!-- ship: active mode = {} -->", mode));
    }

    if parts.is_empty() {
        return None;
    }

    Some(parts.join("\n\n"))
}

// ─── Skill files ──────────────────────────────────────────────────────────────

fn build_skill_files(desc: &ProviderDescriptor, skills: &[Skill]) -> HashMap<String, String> {
    let Some(base) = desc.skills_dir.base_path() else {
        return HashMap::new();
    };
    skills
        .iter()
        .filter(|skill| !skill.content.trim().is_empty())
        .map(|skill| {
            let path = format!("{}/{}/SKILL.md", base, skill.id);
            let content = format_skill_file(skill);
            (path, content)
        })
        .collect()
}

fn format_skill_file(skill: &Skill) -> String {
    let description = skill
        .description
        .as_deref()
        .unwrap_or("No description provided.");
    format!(
        "---\nname: {}\ndescription: {}\n---\n\n{}",
        skill.id, description, skill.content
    )
}

// ─── Claude settings patch ────────────────────────────────────────────────────

/// Build the `.claude/settings.json` patch from permissions, hooks, and model.
///
/// Returns `None` when nothing needs to be written — i.e. when all permissions
/// are at their safe defaults and there are no hooks. This is intentional: Ship
/// must never write a settings file that silently restricts what Claude can do.
/// If no overrides are present, Claude Code's own defaults govern — which is the
/// safest and least surprising behaviour.
pub fn build_claude_settings_patch(
    permissions: &Permissions,
    hooks: &[HookConfig],
    model: Option<&str>,
) -> Option<Json> {
    let has_perms = has_permission_overrides(permissions);
    let has_hooks = !hooks.is_empty();
    let has_agent_limits = permissions.agent.max_cost_per_session.is_some()
        || permissions.agent.max_turns.is_some();
    let has_model = model.is_some();

    if !has_perms && !has_hooks && !has_agent_limits && !has_model {
        return None;
    }

    let mut patch = serde_json::json!({});

    // Tool permissions — only emit when the user has deliberately configured them.
    // Claude Code interprets an explicit `allow` list as a strict allowlist, so we
    // only write it when the user has moved away from the "allow all" default.
    if has_perms {
        let mut perms = serde_json::json!({});
        // Only include allow if the user has a non-default allowlist.
        // Default ("*" or empty) → omit → Claude Code uses its own defaults.
        let non_default_allow = !permissions.tools.allow.is_empty()
            && !(permissions.tools.allow.len() == 1 && permissions.tools.allow[0] == "*");
        if non_default_allow {
            perms["allow"] = serde_json::json!(permissions.tools.allow);
        }
        if !permissions.tools.ask.is_empty() {
            perms["ask"] = serde_json::json!(permissions.tools.ask);
        }
        if !permissions.tools.deny.is_empty() {
            perms["deny"] = serde_json::json!(permissions.tools.deny);
        }
        if let Some(ref mode) = permissions.default_mode {
            perms["defaultMode"] = serde_json::json!(mode);
        }
        if !permissions.additional_directories.is_empty() {
            perms["additionalDirectories"] = serde_json::json!(permissions.additional_directories);
        }
        patch["permissions"] = perms;
    } else {
        // Even without tool-level overrides, emit permissions block if defaultMode or additionalDirectories set
        let mut perms = serde_json::json!({});
        let mut has_extra = false;
        if let Some(ref mode) = permissions.default_mode {
            perms["defaultMode"] = serde_json::json!(mode);
            has_extra = true;
        }
        if !permissions.additional_directories.is_empty() {
            perms["additionalDirectories"] = serde_json::json!(permissions.additional_directories);
            has_extra = true;
        }
        if has_extra {
            patch["permissions"] = perms;
        }
    }

    // Hooks — grouped by trigger type, matching Claude Code's expected structure.
    if has_hooks {
        let mut by_trigger: HashMap<&str, Vec<Json>> = HashMap::new();
        for hook in hooks {
            let key = match hook.trigger {
                HookTrigger::PreToolUse => "PreToolUse",
                HookTrigger::PostToolUse => "PostToolUse",
                HookTrigger::Notification => "Notification",
                HookTrigger::Stop => "Stop",
                HookTrigger::SubagentStop => "SubagentStop",
                HookTrigger::PreCompact => "PreCompact",
            };
            let mut entry = serde_json::json!({ "type": "command", "command": hook.command });
            if let Some(m) = &hook.matcher {
                entry["matcher"] = Json::String(m.clone());
            }
            by_trigger.entry(key).or_default().push(entry);
        }
        patch["hooks"] = serde_json::json!(by_trigger);
    }

    // Agent limits.
    if let Some(cost) = permissions.agent.max_cost_per_session {
        patch["maxCostPerSession"] = serde_json::json!(cost);
    }
    if let Some(turns) = permissions.agent.max_turns {
        patch["maxTurns"] = serde_json::json!(turns);
    }

    // Model override.
    if let Some(m) = model {
        patch["model"] = serde_json::json!(m);
    }

    Some(patch)
}

/// Returns `true` when the permissions object contains any tool-level overrides
/// that deviate from "allow everything" defaults. Filesystem, command, network,
/// and agent limits are checked separately in the caller.
fn has_permission_overrides(p: &Permissions) -> bool {
    let allow_is_default = p.tools.allow.is_empty()
        || (p.tools.allow.len() == 1 && p.tools.allow[0] == "*");
    !allow_is_default
        || !p.tools.ask.is_empty()
        || !p.tools.deny.is_empty()
        || p.default_mode.is_some()
        || !p.additional_directories.is_empty()
}

// ─── Gemini settings patch ────────────────────────────────────────────────────

/// Map our internal `HookTrigger` to Gemini's event name.
/// Source: https://geminicli.com/docs/hooks
fn gemini_trigger(t: &HookTrigger) -> Option<&'static str> {
    match t {
        HookTrigger::PreToolUse  => Some("BeforeTool"),
        HookTrigger::PostToolUse => Some("AfterTool"),
        HookTrigger::Notification => Some("Notification"),
        HookTrigger::Stop        => Some("SessionEnd"),
        HookTrigger::PreCompact  => Some("PreCompress"),
        // No Gemini equivalent
        HookTrigger::SubagentStop => None,
    }
}

/// Build the `hooks` section for `.gemini/settings.json`.
/// Returns `None` when there are no hooks to emit.
fn build_gemini_settings_patch(hooks: &[HookConfig]) -> Option<Json> {
    let mut by_trigger: std::collections::BTreeMap<&str, Vec<Json>> = std::collections::BTreeMap::new();

    for h in hooks {
        let Some(event) = gemini_trigger(&h.trigger) else { continue };
        let mut hook_obj = serde_json::json!({
            "type": "command",
            "command": h.command,
        });
        if let Some(m) = &h.matcher {
            hook_obj["matcher"] = Json::String(m.clone());
        }
        // Gemini schema: { "matcher": "...", "hooks": [{ "type": "command", "command": "..." }] }
        let entry_with_matcher = if let Some(m) = &h.matcher {
            serde_json::json!({ "matcher": m, "hooks": [hook_obj] })
        } else {
            serde_json::json!({ "hooks": [hook_obj] })
        };
        by_trigger.entry(event).or_default().push(entry_with_matcher);
    }

    if by_trigger.is_empty() {
        return None;
    }

    let hooks_obj: serde_json::Map<String, Json> = by_trigger
        .into_iter()
        .map(|(k, v)| (k.to_string(), Json::Array(v)))
        .collect();

    Some(serde_json::json!({ "hooks": hooks_obj }))
}

// ─── Gemini policy patch ──────────────────────────────────────────────────────

/// Translate a permission pattern to a Gemini policy `(tool, pattern)` pair.
///
/// Source: https://geminicli.com/docs/reference/policy-engine/
/// Tool names: `shell`, `mcp`, `file_read`, `file_write`, `web_fetch`
/// `pattern` is a regex string; omit (None) for any-match on that tool.
fn translate_to_gemini_policy(pattern: &str) -> Option<(&'static str, Option<String>)> {
    if pattern == "*" {
        return None; // wildcard = no restriction, skip
    }
    // Bash(cmd) → shell tool, cmd as pattern (glob → anchored regex prefix)
    if let Some(inner) = pattern.strip_prefix("Bash(").and_then(|s| s.strip_suffix(')')) {
        let re = glob_to_regex_prefix(inner);
        return Some(("shell", Some(re)));
    }
    if pattern == "Bash" {
        return Some(("shell", None));
    }
    // mcp__server__tool → mcp tool, server/tool as pattern
    if let Some(rest) = pattern.strip_prefix("mcp__") {
        if let Some(idx) = rest.find("__") {
            let server = &rest[..idx];
            let tool = &rest[idx + 2..];
            let re = format!("{}/{}", glob_to_regex(server), glob_to_regex(tool));
            return Some(("mcp", Some(re)));
        }
        return None;
    }
    // Read/Glob → file_read
    if pattern == "Read" || pattern == "Glob" || pattern == "LS" {
        return Some(("file_read", None));
    }
    if let Some(inner) = pattern.strip_prefix("Read(").and_then(|s| s.strip_suffix(')')) {
        return Some(("file_read", Some(glob_to_regex(inner))));
    }
    // Write/Edit/MultiEdit → file_write
    if matches!(pattern, "Write" | "Edit" | "MultiEdit") {
        return Some(("file_write", None));
    }
    if let Some(inner) = pattern.strip_prefix("Write(").and_then(|s| s.strip_suffix(')'))
        .or_else(|| pattern.strip_prefix("Edit(").and_then(|s| s.strip_suffix(')')))
    {
        return Some(("file_write", Some(glob_to_regex(inner))));
    }
    // WebFetch → web_fetch
    if pattern == "WebFetch" {
        return Some(("web_fetch", None));
    }
    if let Some(inner) = pattern.strip_prefix("WebFetch(").and_then(|s| s.strip_suffix(')')) {
        return Some(("web_fetch", Some(glob_to_regex(inner))));
    }
    None
}

/// Convert a simple glob pattern to a regex prefix for Gemini.
/// Handles `*` → `.*` and escapes regex metacharacters.
fn glob_to_regex(glob: &str) -> String {
    glob_to_regex_prefix(glob)
}

fn glob_to_regex_prefix(glob: &str) -> String {
    let mut out = String::new();
    for ch in glob.chars() {
        match ch {
            '*' => out.push_str(".*"),
            '.' | '+' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '^' | '$' | '|' | '\\' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out
}

/// Build the `.gemini/policies/ship.toml` content.
///
/// Source: https://geminicli.com/docs/reference/policy-engine/
/// Returns `None` when there are no non-default permissions to emit.
fn build_gemini_policy_patch(permissions: &Permissions) -> Option<String> {
    let mut entries: Vec<(String, Option<String>, &str)> = Vec::new(); // (tool, pattern, decision)

    // deny → "deny"
    for p in &permissions.tools.deny {
        if let Some((tool, pattern)) = translate_to_gemini_policy(p) {
            entries.push((tool.to_string(), pattern, "deny"));
        }
    }
    // ask → "ask_user"
    for p in &permissions.tools.ask {
        if let Some((tool, pattern)) = translate_to_gemini_policy(p) {
            entries.push((tool.to_string(), pattern, "ask_user"));
        }
    }
    // non-default allow → "allow"
    let non_default_allow = !permissions.tools.allow.is_empty()
        && !(permissions.tools.allow.len() == 1 && permissions.tools.allow[0] == "*");
    if non_default_allow {
        for p in &permissions.tools.allow {
            if let Some((tool, pattern)) = translate_to_gemini_policy(p) {
                entries.push((tool.to_string(), pattern, "allow"));
            }
        }
    }

    if entries.is_empty() {
        return None;
    }

    let mut toml = String::new();
    toml.push_str("# Generated by Ship. Do not edit manually — run `ship sync` to regenerate.\n\n");
    for (tool, pattern, decision) in entries {
        toml.push_str("[[tool_policies]]\n");
        toml.push_str(&format!("tool = {:?}\n", tool));
        if let Some(p) = pattern {
            toml.push_str(&format!("pattern = {:?}\n", p));
        }
        toml.push_str(&format!("decision = {:?}\n", decision));
        toml.push('\n');
    }

    Some(toml)
}

// ─── Cursor hooks patch ───────────────────────────────────────────────────────

/// Map our internal `HookTrigger` to Cursor's event name(s).
/// Source: https://cursor.com/docs/agent/hooks
/// Cursor splits PreToolUse into beforeMCPExecution / beforeShellExecution;
/// we emit both so hooks fire regardless of tool type.
fn cursor_triggers(t: &HookTrigger) -> &'static [&'static str] {
    match t {
        HookTrigger::PreToolUse  => &["beforeMCPExecution", "beforeShellExecution"],
        HookTrigger::PostToolUse => &["afterMCPExecution", "afterShellExecution"],
        HookTrigger::Stop        => &["sessionEnd"],
        // No Cursor equivalents
        HookTrigger::Notification | HookTrigger::SubagentStop | HookTrigger::PreCompact => &[],
    }
}

/// Build the full `.cursor/hooks.json` content.
/// Returns `None` when there are no mappable hooks.
fn build_cursor_hooks_patch(hooks: &[HookConfig]) -> Option<Json> {
    let mut by_event: std::collections::BTreeMap<&str, Vec<Json>> = std::collections::BTreeMap::new();

    for h in hooks {
        for &event in cursor_triggers(&h.trigger) {
            let mut entry = serde_json::json!({ "command": h.command });
            if let Some(m) = &h.matcher {
                entry["matcher"] = Json::String(m.clone());
            }
            by_event.entry(event).or_default().push(entry);
        }
    }

    if by_event.is_empty() {
        return None;
    }

    let obj: serde_json::Map<String, Json> = by_event
        .into_iter()
        .map(|(k, v)| (k.to_string(), Json::Array(v)))
        .collect();

    Some(Json::Object(obj))
}

// ─── Cursor permissions ───────────────────────────────────────────────────────

/// Translate a pattern to Cursor's typed permission format.
///
/// Source: https://cursor.com/docs/cli/reference/permissions
/// Cursor uses: `Shell(cmd)`, `Read(glob)`, `Write(glob)`, `WebFetch(domain)`, `Mcp(server:tool)`.
/// These apply to both Cursor IDE agents and the Cursor CLI.
///
/// The bare wildcard `"*"` is intentionally NOT translated to the permissive set
/// `[Shell(*), Read(*), Write(*), WebFetch(*), Mcp(*:*)]`. Cursor's default without
/// a config is interactive/prompt — `"*"` must be an explicit UI choice, not a silent default.
///
/// Returns `None` for patterns with no Cursor equivalent (they are silently dropped).
fn translate_to_cursor_permission(pattern: &str) -> Option<String> {
    // Bare wildcard — must not be auto-expanded. The caller handles the permissive preset.
    if pattern == "*" {
        return None;
    }
    // mcp__server__tool → Mcp(server:tool)
    // e.g. mcp__ship__read_notes → Mcp(ship:read_notes)
    //      mcp__*__delete* → Mcp(*:delete*)
    if let Some(rest) = pattern.strip_prefix("mcp__") {
        if let Some(idx) = rest.find("__") {
            let server = &rest[..idx];
            let tool = &rest[idx + 2..];
            return Some(format!("Mcp({server}:{tool})"));
        }
        return None; // malformed — skip
    }
    // Bash(cmd) → Shell(cmd)
    if let Some(inner) = pattern.strip_prefix("Bash(").and_then(|s| s.strip_suffix(')')) {
        return Some(format!("Shell({inner})"));
    }
    if pattern == "Bash" {
        return Some("Shell(*)".to_string());
    }
    // Read/Glob/LS → Read(*)
    if matches!(pattern, "Read" | "Glob" | "LS") {
        return Some("Read(*)".to_string());
    }
    // Write/Edit/MultiEdit → Write(*)
    if matches!(pattern, "Write" | "Edit" | "MultiEdit") {
        return Some("Write(*)".to_string());
    }
    // WebFetch(domain) — pass through; bare → wildcard
    if pattern == "WebFetch" {
        return Some("WebFetch(*)".to_string());
    }
    if pattern.starts_with("WebFetch(") {
        return Some(pattern.to_string());
    }
    // Unknown tool pattern — drop rather than emit garbage
    None
}

/// The explicit permissive allow list for Cursor.
/// Cursor has no implicit allow-all; this must be emitted only when the user
/// has deliberately chosen "permissive" mode (with appropriate warnings in the UI).
pub const CURSOR_PERMISSIVE_ALLOW: &[&str] =
    &["Shell(*)", "Read(*)", "Write(*)", "WebFetch(*)", "Mcp(*:*)"];

/// Build the `.cursor/cli.json` permissions content.
///
/// Returns `None` when there are no non-default permissions to emit.
/// IMPORTANT: `allow = ["*"]` (our internal allow-all) does NOT automatically expand
/// to the permissive Cursor set — that must be an explicit user action in the UI.
fn build_cursor_cli_permissions(permissions: &Permissions) -> Option<Json> {
    let non_default_allow = !permissions.tools.allow.is_empty()
        && !(permissions.tools.allow.len() == 1 && permissions.tools.allow[0] == "*");

    let allow_patterns: Vec<String> = if non_default_allow {
        permissions.tools.allow
            .iter()
            .filter_map(|p| translate_to_cursor_permission(p))
            .collect()
    } else {
        vec![]
    };

    let deny_patterns: Vec<String> = permissions.tools.deny
        .iter()
        .filter_map(|p| translate_to_cursor_permission(p))
        .collect();

    if allow_patterns.is_empty() && deny_patterns.is_empty() {
        return None;
    }

    let mut perms = serde_json::json!({});
    if !allow_patterns.is_empty() {
        perms["allow"] = serde_json::json!(allow_patterns);
    }
    if !deny_patterns.is_empty() {
        perms["deny"] = serde_json::json!(deny_patterns);
    }

    Some(serde_json::json!({ "version": 1, "permissions": perms }))
}

// ─── Cursor rule files (.cursor/rules/*.mdc) ──────────────────────────────────

/// Build per-file `.cursor/rules/<name>.mdc` entries with YAML frontmatter.
fn build_cursor_rule_files(rules: &[Rule]) -> HashMap<String, String> {
    rules
        .iter()
        .filter(|r| !r.content.trim().is_empty())
        .map(|rule| {
            let stem = rule
                .file_name
                .trim_end_matches(".md")
                .trim_end_matches(".mdc");
            let path = format!(".cursor/rules/{}.mdc", stem);
            let content = format_cursor_rule(rule);
            (path, content)
        })
        .collect()
}

fn format_cursor_rule(rule: &Rule) -> String {
    let mut fm = String::new();
    fm.push_str("---\n");
    if let Some(desc) = &rule.description {
        fm.push_str(&format!("description: {:?}\n", desc));
    }
    if !rule.globs.is_empty() {
        fm.push_str("globs:\n");
        for g in &rule.globs {
            fm.push_str(&format!("  - {}\n", g));
        }
    }
    fm.push_str(&format!("alwaysApply: {}\n", rule.always_apply));
    fm.push_str("---\n\n");
    fm.push_str(rule.content.trim());
    fm
}

// ─── Codex TOML config ────────────────────────────────────────────────────────

/// Build the `[mcp_servers.*]` TOML tables for `.codex/config.toml`.
///
/// Source: https://developers.openai.com/codex/mcp
/// Codex uses TOML, not JSON. Each server is a `[mcp_servers.<id>]` table.
/// Returns `None` if there are no enabled servers to write.
fn build_codex_config_patch(servers: &[McpServerConfig]) -> Option<String> {
    let mut mcp = toml::Table::new();

    // Ship server always first
    let mut ship_entry = toml::Table::new();
    ship_entry.insert("command".into(), toml::Value::String("ship-mcp".into()));
    ship_entry.insert("args".into(), toml::Value::Array(vec![]));
    mcp.insert("ship".into(), toml::Value::Table(ship_entry));

    for s in servers {
        if s.disabled {
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
                            s.args.iter().map(|a| toml::Value::String(a.clone())).collect(),
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
        mcp.insert(s.id.clone(), toml::Value::Table(entry));
    }

    let mut root = toml::Table::new();
    root.insert("mcp_servers".into(), toml::Value::Table(mcp));
    toml::to_string(&root).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolve::ResolvedConfig;
    use crate::types::{
        AgentLimits, HookConfig, HookTrigger, McpServerConfig, McpServerType, Permissions,
        Rule, Skill, ToolPermissions,
    };

    // ── Fixtures ──────────────────────────────────────────────────────────────

    fn make_server(id: &str) -> McpServerConfig {
        McpServerConfig {
            id: id.to_string(),
            name: id.to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), format!("@mcp/{}", id)],
            env: Default::default(),
            scope: "project".to_string(),
            server_type: McpServerType::Stdio,
            url: None,
            disabled: false,
            timeout_secs: None,
        }
    }

    fn make_skill(id: &str) -> Skill {
        Skill {
            id: id.to_string(),
            name: id.to_string(),
            description: Some(format!("{} skill", id)),
            version: None,
            author: None,
            content: format!("# {}\n\nDo the thing.", id),
            source: Default::default(),
        }
    }

    fn make_hook(trigger: HookTrigger, command: &str, matcher: Option<&str>) -> HookConfig {
        HookConfig {
            id: "test-hook".to_string(),
            trigger,
            matcher: matcher.map(str::to_string),
            command: command.to_string(),
        }
    }

    fn make_rule(file_name: &str, content: &str) -> Rule {
        Rule {
            file_name: file_name.to_string(),
            content: content.to_string(),
            always_apply: true,
            globs: vec![],
            description: None,
        }
    }

    fn resolved(servers: Vec<McpServerConfig>) -> ResolvedConfig {
        ResolvedConfig {
            providers: vec!["claude".to_string()],
            model: None,
            max_cost_per_session: None,
            max_turns: None,
            mcp_servers: servers,
            skills: vec![],
            rules: vec![],
            permissions: Permissions::default(),
            hooks: vec![],
            active_mode: None,
        }
    }

    // ── Safety: permissions must not silently block tools ─────────────────────

    /// The most important test. Default config → no settings patch written.
    /// Claude Code's own defaults govern — Ship must not interfere.
    #[test]
    fn default_permissions_emit_no_settings_patch() {
        let out = compile(&resolved(vec![]), "claude").unwrap();
        assert!(
            out.claude_settings_patch.is_none(),
            "Default permissions must not emit a settings patch — \
             any patch with an empty allow list could silently block tools"
        );
    }

    /// Explicit deny-only is safe: it only restricts what the user asked to restrict.
    #[test]
    fn deny_only_emits_patch_with_no_allow_field() {
        let r = ResolvedConfig {
            permissions: Permissions {
                tools: ToolPermissions {
                    allow: vec!["*".to_string()], // default
                    deny: vec!["Bash(rm -rf *)".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..resolved(vec![])
        };
        let out = compile(&r, "claude").unwrap();
        let patch = out.claude_settings_patch.expect("deny-only must emit a patch");
        let perms = &patch["permissions"];

        // deny must be present
        let deny = perms["deny"].as_array().unwrap();
        assert_eq!(deny.len(), 1);
        assert_eq!(deny[0], "Bash(rm -rf *)");

        // allow must NOT be emitted — default allow means Claude uses its own defaults
        assert!(
            perms.get("allow").is_none() || perms["allow"].as_array().map(|a| a.is_empty()).unwrap_or(false),
            "allow field must be absent or empty when the user has not restricted the allowlist"
        );
    }

    /// The "guarded" preset uses allow=["*"] + scoped deny patterns.
    /// This must NOT emit an allow field — only the deny patterns.
    /// If allow=["*"] were treated as an explicit allowlist it would silently block
    /// every tool not listed, which is the exact trust failure we are guarding against.
    #[test]
    fn guarded_preset_never_emits_allow_field() {
        let r = ResolvedConfig {
            permissions: Permissions {
                tools: ToolPermissions {
                    allow: vec!["*".to_string()],
                    deny: vec![
                        "mcp__*__delete*".to_string(),
                        "mcp__*__drop*".to_string(),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..resolved(vec![])
        };
        let out = compile(&r, "claude").unwrap();
        let patch = out.claude_settings_patch.expect("deny patterns must emit a patch");
        let perms = &patch["permissions"];
        assert!(
            perms.get("allow").is_none() || perms["allow"].as_array().map(|a| a.is_empty()).unwrap_or(false),
            "guarded preset (allow=[*] + deny) must not emit an allow field — it would become a strict allowlist"
        );
        let deny = perms["deny"].as_array().unwrap();
        assert_eq!(deny.len(), 2);
    }

    /// "allow *" with no deny is the same as default — no patch needed.
    #[test]
    fn allow_star_with_no_deny_emits_no_patch() {
        let r = ResolvedConfig {
            permissions: Permissions {
                tools: ToolPermissions {
                    allow: vec!["*".to_string()],
                    deny: vec![],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..resolved(vec![])
        };
        let out = compile(&r, "claude").unwrap();
        assert!(
            out.claude_settings_patch.is_none(),
            "allow=[*] deny=[] is the identity case — must not write a patch"
        );
    }

    /// An explicit non-wildcard allow list is a deliberate restriction and must compile.
    #[test]
    fn explicit_allow_list_compiles_correctly() {
        let r = ResolvedConfig {
            permissions: Permissions {
                tools: ToolPermissions {
                    allow: vec!["Read".to_string(), "Glob".to_string()],
                    deny: vec![],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..resolved(vec![])
        };
        let out = compile(&r, "claude").unwrap();
        let patch = out.claude_settings_patch.expect("explicit allow must emit a patch");
        let allow = patch["permissions"]["allow"].as_array().unwrap();
        assert_eq!(allow.len(), 2);
        assert!(allow.iter().any(|v| v == "Read"));
        assert!(allow.iter().any(|v| v == "Glob"));
    }

    /// Permissions patch is only generated for Claude — not Gemini or Codex.
    /// Other providers have their own settings formats; silently writing Claude
    /// permissions to them would be wrong.
    #[test]
    fn settings_patch_only_for_claude() {
        let r = ResolvedConfig {
            permissions: Permissions {
                tools: ToolPermissions {
                    allow: vec!["*".to_string()],
                    deny: vec!["Write".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..resolved(vec![])
        };
        let gemini = compile(&r, "gemini").unwrap();
        assert!(
            gemini.claude_settings_patch.is_none(),
            "Gemini must not receive a Claude settings patch"
        );
        let codex = compile(&r, "codex").unwrap();
        assert!(
            codex.claude_settings_patch.is_none(),
            "Codex must not receive a Claude settings patch"
        );
    }

    // ── Hooks ─────────────────────────────────────────────────────────────────

    /// Hooks must thread from ResolvedConfig all the way into the settings patch.
    #[test]
    fn hooks_compile_into_settings_patch() {
        let r = ResolvedConfig {
            hooks: vec![make_hook(
                HookTrigger::PreToolUse,
                "ship hooks check",
                Some("Bash"),
            )],
            ..resolved(vec![])
        };
        let out = compile(&r, "claude").unwrap();
        let patch = out.claude_settings_patch.expect("hooks must emit a patch");
        let hooks = patch["hooks"]["PreToolUse"].as_array().unwrap();
        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks[0]["command"], "ship hooks check");
        assert_eq!(hooks[0]["matcher"], "Bash");
    }

    /// Multiple hooks on the same trigger type → all emitted in order.
    #[test]
    fn multiple_hooks_same_trigger_all_emitted() {
        let r = ResolvedConfig {
            hooks: vec![
                make_hook(HookTrigger::PostToolUse, "ship log tool", Some("*")),
                make_hook(HookTrigger::PostToolUse, "ship analytics flush", None),
            ],
            ..resolved(vec![])
        };
        let out = compile(&r, "claude").unwrap();
        let patch = out.claude_settings_patch.unwrap();
        let hooks = patch["hooks"]["PostToolUse"].as_array().unwrap();
        assert_eq!(hooks.len(), 2);
    }

    /// Hooks on different trigger types → correctly separated in the output.
    #[test]
    fn hooks_grouped_by_trigger_type() {
        let r = ResolvedConfig {
            hooks: vec![
                make_hook(HookTrigger::PreToolUse, "before", Some("Bash")),
                make_hook(HookTrigger::PostToolUse, "after", None),
                make_hook(HookTrigger::Stop, "on-stop", None),
            ],
            ..resolved(vec![])
        };
        let out = compile(&r, "claude").unwrap();
        let patch = out.claude_settings_patch.unwrap();
        assert_eq!(patch["hooks"]["PreToolUse"].as_array().unwrap().len(), 1);
        assert_eq!(patch["hooks"]["PostToolUse"].as_array().unwrap().len(), 1);
        assert_eq!(patch["hooks"]["Stop"].as_array().unwrap().len(), 1);
        assert!(patch["hooks"].get("PreCompact").is_none());
    }

    /// A hook without a matcher must not emit an empty "matcher" field.
    #[test]
    fn hook_without_matcher_omits_matcher_field() {
        let r = ResolvedConfig {
            hooks: vec![make_hook(HookTrigger::Stop, "ship notify", None)],
            ..resolved(vec![])
        };
        let out = compile(&r, "claude").unwrap();
        let patch = out.claude_settings_patch.unwrap();
        let hook = &patch["hooks"]["Stop"][0];
        assert!(
            hook.get("matcher").is_none(),
            "matcher field must be absent when not set, not null/empty"
        );
    }

    // ── Agent limits ─────────────────────────────────────────────────────────

    #[test]
    fn max_cost_per_session_emits_to_settings() {
        let r = ResolvedConfig {
            permissions: Permissions {
                agent: AgentLimits {
                    max_cost_per_session: Some(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..resolved(vec![])
        };
        let out = compile(&r, "claude").unwrap();
        let patch = out.claude_settings_patch.expect("cost limit must emit a patch");
        assert_eq!(patch["maxCostPerSession"], 5.0);
    }

    #[test]
    fn max_turns_emits_to_settings() {
        let r = ResolvedConfig {
            permissions: Permissions {
                agent: AgentLimits {
                    max_turns: Some(20),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..resolved(vec![])
        };
        let out = compile(&r, "claude").unwrap();
        let patch = out.claude_settings_patch.expect("turn limit must emit a patch");
        assert_eq!(patch["maxTurns"], 20);
    }

    // ── MCP server output correctness ─────────────────────────────────────────

    #[test]
    fn ship_server_always_first() {
        let r = resolved(vec![make_server("github"), make_server("linear")]);
        let out = compile(&r, "claude").unwrap();
        let keys: Vec<&str> = out.mcp_servers.as_object().unwrap().keys().map(|k| k.as_str()).collect();
        assert_eq!(keys[0], "ship", "ship server must be first in MCP output");
    }

    #[test]
    fn disabled_server_excluded() {
        let mut s = make_server("github");
        s.disabled = true;
        let out = compile(&resolved(vec![s]), "claude").unwrap();
        assert!(!out.mcp_servers.as_object().unwrap().contains_key("github"));
    }

    #[test]
    fn claude_uses_mcpservers_key() {
        let out = compile(&resolved(vec![make_server("x")]), "claude").unwrap();
        // mcp_servers is the root object keyed by the server ids — check claude doesn't use underscored key
        // (The key check is implicit: the output IS the object, not wrapped further)
        assert!(out.mcp_servers.is_object());
        assert!(out.mcp_servers.as_object().unwrap().contains_key("x"));
    }

    #[test]
    fn codex_uses_mcp_servers_key() {
        let desc = get_provider("codex").unwrap();
        assert_eq!(desc.mcp_key.as_str(), "mcp_servers");
    }

    // ── Codex TOML output ─────────────────────────────────────────────────────

    /// Source: https://developers.openai.com/codex/mcp
    /// Codex config is TOML — codex_config_patch must be a valid TOML string.
    #[test]
    fn codex_produces_toml_patch_not_json() {
        let r = resolved(vec![make_server("github")]);
        let out = compile(&r, "codex").unwrap();
        let toml_str = out.codex_config_patch.expect("codex must emit a TOML config patch");
        // Must parse as valid TOML
        let parsed: toml::Table = toml::from_str(&toml_str).expect("must be valid TOML");
        assert!(parsed.contains_key("mcp_servers"), "must have mcp_servers table");
    }

    #[test]
    fn codex_toml_ship_server_first() {
        let r = resolved(vec![make_server("github")]);
        let out = compile(&r, "codex").unwrap();
        let toml_str = out.codex_config_patch.unwrap();
        let parsed: toml::Table = toml::from_str(&toml_str).unwrap();
        let mcp = parsed["mcp_servers"].as_table().unwrap();
        let keys: Vec<&str> = mcp.keys().map(|k| k.as_str()).collect();
        assert_eq!(keys[0], "ship", "ship server must be first in Codex TOML output");
    }

    #[test]
    fn codex_toml_stdio_entry_shape() {
        let mut s = make_server("context7");
        s.args = vec!["-y".into(), "@upstash/context7-mcp".into()];
        let r = resolved(vec![s]);
        let out = compile(&r, "codex").unwrap();
        let toml_str = out.codex_config_patch.unwrap();
        let parsed: toml::Table = toml::from_str(&toml_str).unwrap();
        let entry = parsed["mcp_servers"]["context7"].as_table().unwrap();
        assert_eq!(entry["command"].as_str().unwrap(), "npx");
        assert_eq!(entry["args"].as_array().unwrap().len(), 2);
        assert!(entry.get("type").is_none(), "no type field in Codex TOML");
    }

    #[test]
    fn codex_toml_disabled_server_excluded() {
        let mut s = make_server("disabled");
        s.disabled = true;
        let r = resolved(vec![s]);
        let out = compile(&r, "codex").unwrap();
        let toml_str = out.codex_config_patch.unwrap();
        let parsed: toml::Table = toml::from_str(&toml_str).unwrap();
        let mcp = parsed["mcp_servers"].as_table().unwrap();
        assert!(!mcp.contains_key("disabled"));
    }

    #[test]
    fn codex_toml_timeout_uses_startup_timeout_sec() {
        let mut s = make_server("slow");
        s.timeout_secs = Some(30);
        let r = resolved(vec![s]);
        let out = compile(&r, "codex").unwrap();
        let toml_str = out.codex_config_patch.unwrap();
        let parsed: toml::Table = toml::from_str(&toml_str).unwrap();
        assert_eq!(
            parsed["mcp_servers"]["slow"]["startup_timeout_sec"].as_integer().unwrap(),
            30
        );
    }

    #[test]
    fn codex_no_json_mcp_key_confusion() {
        // The JSON mcp_servers output uses "mcp_servers" key (McpServersUnderscored)
        // but the real Codex config is TOML — verify codex_config_patch is populated
        // and claude/cursor don't accidentally get a codex patch.
        let r = resolved(vec![make_server("x")]);
        assert!(compile(&r, "codex").unwrap().codex_config_patch.is_some());
        assert!(compile(&r, "claude").unwrap().codex_config_patch.is_none());
        assert!(compile(&r, "cursor").unwrap().codex_config_patch.is_none());
        assert!(compile(&r, "gemini").unwrap().codex_config_patch.is_none());
    }

    /// Source: https://geminicli.com/docs/tools/mcp-server
    /// Gemini has NO "type" field — transport is inferred from which property is present.
    /// Claude, Cursor: also no "type" field. Only verified if a provider sets emit_type_field.
    #[test]
    fn no_provider_emits_type_field_for_stdio() {
        let r = resolved(vec![make_server("github")]);
        for provider_id in &["claude", "gemini", "codex", "cursor"] {
            let out = compile(&r, provider_id).unwrap();
            assert!(
                out.mcp_servers["github"].get("type").is_none(),
                "{provider_id}: must not emit 'type' field for stdio servers"
            );
        }
    }

    #[test]
    fn http_server_url_field_per_provider() {
        let mut s = make_server("remote");
        s.server_type = McpServerType::Http;
        s.url = Some("https://api.example.com/mcp".to_string());
        s.command = String::new();
        s.args = vec![];

        let r = resolved(vec![s]);

        let claude_out = compile(&r, "claude").unwrap();
        assert!(claude_out.mcp_servers["remote"].get("url").is_some(), "Claude uses 'url'");
        assert!(claude_out.mcp_servers["remote"].get("httpUrl").is_none());

        let gemini_out = compile(&r, "gemini").unwrap();
        assert!(gemini_out.mcp_servers["remote"].get("httpUrl").is_some(), "Gemini uses 'httpUrl'");
        assert!(gemini_out.mcp_servers["remote"].get("url").is_none());
    }

    /// Source: https://geminicli.com/docs/tools/mcp-server
    /// Gemini SSE uses "url" field (not "httpUrl" — that's only for streamable HTTP).
    #[test]
    fn gemini_sse_uses_url_field_not_httpurl() {
        let mut s = make_server("sse-server");
        s.server_type = McpServerType::Sse;
        s.url = Some("https://sse.example.com/mcp".to_string());
        s.command = String::new();
        s.args = vec![];

        let r = resolved(vec![s]);
        let out = compile(&r, "gemini").unwrap();
        assert!(out.mcp_servers["sse-server"].get("url").is_some(), "Gemini SSE must use 'url' field");
        assert!(out.mcp_servers["sse-server"].get("httpUrl").is_none(), "Gemini SSE must not use 'httpUrl'");
    }

    #[test]
    fn timeout_secs_maps_to_startup_timeout_sec() {
        let mut s = make_server("slow");
        s.timeout_secs = Some(30);
        let out = compile(&resolved(vec![s]), "claude").unwrap();
        assert_eq!(out.mcp_servers["slow"]["startup_timeout_sec"], 30);
    }

    // ── Context file (CLAUDE.md / AGENTS.md / GEMINI.md) ─────────────────────

    #[test]
    fn provider_context_file_names() {
        let desc_claude = get_provider("claude").unwrap();
        let desc_gemini = get_provider("gemini").unwrap();
        let desc_codex = get_provider("codex").unwrap();
        assert_eq!(desc_claude.context_file.file_name(), Some("CLAUDE.md"));
        assert_eq!(desc_gemini.context_file.file_name(), Some("GEMINI.md"));
        assert_eq!(desc_codex.context_file.file_name(), Some("AGENTS.md"));
    }

    #[test]
    fn rules_concatenated_into_context_file() {
        let r = ResolvedConfig {
            rules: vec![
                Rule { file_name: "style.md".into(), content: "Use explicit types.".into(), always_apply: true, globs: vec![], description: None },
                Rule { file_name: "workflow.md".into(), content: "Run tests before committing.".into(), always_apply: true, globs: vec![], description: None },
            ],
            ..resolved(vec![])
        };
        let out = compile(&r, "claude").unwrap();
        let content = out.context_content.expect("rules must produce a context file");
        assert!(content.contains("Use explicit types."));
        assert!(content.contains("Run tests before committing."));
    }

    #[test]
    fn no_rules_no_context_file() {
        let out = compile(&resolved(vec![]), "claude").unwrap();
        assert!(out.context_content.is_none(), "empty rules must not produce a context file");
    }

    // ── Skill files ───────────────────────────────────────────────────────────

    /// Skills with empty content (stubs) must be silently skipped — not emitted
    /// as empty SKILL.md files that confuse agents.
    #[test]
    fn empty_skill_content_is_filtered() {
        let mut stub = make_skill("git-commit");
        stub.content = String::new();
        let mut real = make_skill("code-review");
        real.content = "## Instructions\nReview the diff carefully.".to_string();
        let r = ResolvedConfig {
            skills: vec![stub, real],
            ..resolved(vec![])
        };
        let out = compile(&r, "claude").unwrap();
        assert!(!out.skill_files.contains_key(".claude/skills/git-commit/SKILL.md"),
            "stub skill with empty content must not be emitted");
        assert!(out.skill_files.contains_key(".claude/skills/code-review/SKILL.md"),
            "skill with content must be emitted");
    }

    #[test]
    fn skill_files_provider_directories() {
        let r = ResolvedConfig {
            skills: vec![make_skill("rust-expert")],
            ..resolved(vec![])
        };
        assert!(compile(&r, "claude").unwrap().skill_files.contains_key(".claude/skills/rust-expert/SKILL.md"));
        assert!(compile(&r, "gemini").unwrap().skill_files.contains_key(".agents/skills/rust-expert/SKILL.md"));
        assert!(compile(&r, "codex").unwrap().skill_files.contains_key(".agents/skills/rust-expert/SKILL.md"));
    }

    #[test]
    fn skill_file_has_yaml_frontmatter_and_content() {
        let r = ResolvedConfig {
            skills: vec![make_skill("my-skill")],
            ..resolved(vec![])
        };
        let out = compile(&r, "claude").unwrap();
        let content = &out.skill_files[".claude/skills/my-skill/SKILL.md"];
        assert!(content.starts_with("---\n"), "skill file must start with YAML frontmatter");
        assert!(content.contains("name: my-skill"));
        assert!(content.contains("# my-skill"), "skill file must contain markdown content");
    }

    // ── Idempotency ───────────────────────────────────────────────────────────

    /// Compiling the same input twice must produce identical output.
    /// This is a prerequisite for reliable file writes — no random ordering,
    /// no non-deterministic IDs, no timestamp injection.
    #[test]
    fn compile_is_idempotent() {
        let r = ResolvedConfig {
            mcp_servers: vec![make_server("github"), make_server("linear")],
            skills: vec![make_skill("rust-expert")],
            rules: vec![Rule { file_name: "style.md".into(), content: "Use explicit types.".into(), always_apply: true, globs: vec![], description: None }],
            permissions: Permissions {
                tools: ToolPermissions {
                    allow: vec!["*".to_string()],
                    deny: vec!["Bash(rm -rf *)".to_string()],
                    ..Default::default()
                },
                agent: AgentLimits {
                    max_cost_per_session: Some(2.50),
                    ..Default::default()
                },
                ..Default::default()
            },
            hooks: vec![make_hook(HookTrigger::PostToolUse, "ship log", Some("*"))],
            ..resolved(vec![])
        };

        let first = compile(&r, "claude").unwrap();
        let second = compile(&r, "claude").unwrap();

        assert_eq!(
            serde_json::to_string(&first.mcp_servers).unwrap(),
            serde_json::to_string(&second.mcp_servers).unwrap(),
            "MCP output must be identical across compilations"
        );
        assert_eq!(first.context_content, second.context_content);
        assert_eq!(first.skill_files, second.skill_files);
        assert_eq!(
            serde_json::to_string(&first.claude_settings_patch).unwrap(),
            serde_json::to_string(&second.claude_settings_patch).unwrap(),
            "Settings patch must be identical across compilations"
        );
    }

    /// compile_library_all via ProjectLibrary round-trip — ensure WASM entrypoint
    /// threads everything correctly end to end.
    #[test]
    fn library_round_trip_via_resolve() {
        use crate::resolve::{ProjectLibrary, resolve_library};

        let library = ProjectLibrary {
            mcp_servers: vec![make_server("github")],
            skills: vec![make_skill("workflow")],
            rules: vec![Rule { file_name: "style.md".into(), content: "Keep it clean.".into(), always_apply: true, globs: vec![], description: None }],
            permissions: Permissions {
                tools: ToolPermissions {
                    allow: vec!["*".to_string()],
                    deny: vec!["Bash(sudo *)".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
            hooks: vec![make_hook(HookTrigger::PreToolUse, "ship check", Some("Bash"))],
            ..Default::default()
        };

        let resolved = resolve_library(&library, None, None);

        // Hooks must have threaded through
        assert_eq!(resolved.hooks.len(), 1, "hooks must survive resolve_library");

        let out = compile(&resolved, "claude").unwrap();

        // MCP
        assert!(out.mcp_servers["github"].is_object());

        // Context
        assert!(out.context_content.unwrap().contains("Keep it clean."));

        // Skills
        assert!(out.skill_files.contains_key(".claude/skills/workflow/SKILL.md"));

        // Settings patch: deny present, hooks present
        let patch = out.claude_settings_patch.unwrap();
        assert_eq!(patch["permissions"]["deny"][0], "Bash(sudo *)");
        assert!(patch["hooks"]["PreToolUse"].is_array());
        assert_eq!(patch["hooks"]["PreToolUse"][0]["matcher"], "Bash");

        // mcp_config_path exposed
        assert_eq!(out.mcp_config_path, Some(".mcp.json"));
    }

    // ── Cursor provider ───────────────────────────────────────────────────────

    #[test]
    fn cursor_provider_exists() {
        let desc = get_provider("cursor").expect("cursor provider must be registered");
        assert_eq!(desc.name, "Cursor");
        assert_eq!(desc.mcp_key.as_str(), "mcpServers");
        assert!(!desc.emit_type_field, "Cursor does not emit type field for stdio");
        assert_eq!(desc.http_url_field, "url");
        assert_eq!(desc.mcp_config_path, Some(".cursor/mcp.json"));
    }

    #[test]
    fn cursor_mcp_matches_claude_format() {
        // Cursor uses the same mcpServers shape as Claude — no "type" field on stdio servers.
        let r = resolved(vec![make_server("github")]);
        let claude = compile(&r, "claude").unwrap();
        let cursor = compile(&r, "cursor").unwrap();

        // Both have the ship server first
        let claude_keys: Vec<&str> = claude.mcp_servers.as_object().unwrap().keys().map(|k| k.as_str()).collect();
        let cursor_keys: Vec<&str> = cursor.mcp_servers.as_object().unwrap().keys().map(|k| k.as_str()).collect();
        assert_eq!(claude_keys[0], "ship");
        assert_eq!(cursor_keys[0], "ship");

        // Neither emits a "type" field for stdio
        assert!(cursor.mcp_servers["github"].get("type").is_none());
        assert!(claude.mcp_servers["github"].get("type").is_none());
    }

    #[test]
    fn cursor_skill_files_in_rules_dir() {
        let r = ResolvedConfig {
            skills: vec![make_skill("refactor")],
            ..resolved(vec![])
        };
        let out = compile(&r, "cursor").unwrap();
        assert!(
            out.skill_files.contains_key(".cursor/skills/refactor/SKILL.md"),
            "Cursor skills must go in .cursor/skills/"
        );
    }

    #[test]
    fn cursor_context_file_is_none() {
        // Cursor uses per-file .mdc rules instead of a single context file.
        let desc = get_provider("cursor").unwrap();
        assert_eq!(desc.context_file, ContextFile::None);
        assert_eq!(desc.context_file.file_name(), None);
    }

    #[test]
    fn cursor_no_settings_patch() {
        // Cursor has no structured permissions config — must never produce a Claude settings patch.
        let r = ResolvedConfig {
            permissions: Permissions {
                tools: ToolPermissions {
                    allow: vec!["*".to_string()],
                    deny: vec!["Bash(rm -rf *)".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..resolved(vec![])
        };
        let out = compile(&r, "cursor").unwrap();
        assert!(
            out.claude_settings_patch.is_none(),
            "Cursor must never receive a Claude settings patch"
        );
    }

    #[test]
    fn cursor_is_valid_provider_in_normalize() {
        use crate::resolve::{FeatureOverrides, resolve};
        use crate::types::ProjectConfig;
        let feature = FeatureOverrides {
            providers: vec!["cursor".to_string()],
            ..Default::default()
        };
        let resolved = resolve(&ProjectConfig::default(), &[], &[], &Permissions::default(), &[], Some(&feature), None);
        assert_eq!(resolved.providers, vec!["cursor"]);
    }

    #[test]
    fn cursor_mcp_config_path_in_output() {
        let r = resolved(vec![]);
        let out = compile(&r, "cursor").unwrap();
        assert_eq!(out.mcp_config_path, Some(".cursor/mcp.json"));
    }

    #[test]
    fn claude_mcp_config_path_in_output() {
        let r = resolved(vec![]);
        let out = compile(&r, "claude").unwrap();
        assert_eq!(out.mcp_config_path, Some(".mcp.json"));
    }

    // ── Gemini hooks ──────────────────────────────────────────────────────────

    #[test]
    fn gemini_hooks_pre_tool_maps_to_before_tool() {
        let r = ResolvedConfig {
            hooks: vec![make_hook(HookTrigger::PreToolUse, "ship check", Some("Bash"))],
            ..resolved(vec![])
        };
        let out = compile(&r, "gemini").unwrap();
        let patch = out.gemini_settings_patch.expect("gemini must emit hooks patch");
        assert!(patch["hooks"]["BeforeTool"].is_array());
        assert_eq!(patch["hooks"]["BeforeTool"][0]["matcher"], "Bash");
        assert_eq!(patch["hooks"]["BeforeTool"][0]["hooks"][0]["command"], "ship check");
    }

    #[test]
    fn gemini_hooks_trigger_mapping() {
        let hooks = vec![
            make_hook(HookTrigger::PreToolUse,  "cmd-pre",    None),
            make_hook(HookTrigger::PostToolUse, "cmd-post",   None),
            make_hook(HookTrigger::Stop,        "cmd-stop",   None),
            make_hook(HookTrigger::PreCompact,  "cmd-compact",None),
            make_hook(HookTrigger::Notification,"cmd-notify", None),
        ];
        let r = ResolvedConfig { hooks, ..resolved(vec![]) };
        let out = compile(&r, "gemini").unwrap();
        let patch = out.gemini_settings_patch.unwrap();
        let h = &patch["hooks"];
        assert!(h["BeforeTool"].is_array(),   "PreToolUse → BeforeTool");
        assert!(h["AfterTool"].is_array(),    "PostToolUse → AfterTool");
        assert!(h["SessionEnd"].is_array(),   "Stop → SessionEnd");
        assert!(h["PreCompress"].is_array(),  "PreCompact → PreCompress");
        assert!(h["Notification"].is_array(), "Notification → Notification");
        // SubagentStop has no Gemini equivalent — must not appear
        assert!(h.get("SubagentStop").is_none());
    }

    #[test]
    fn gemini_hooks_subagent_stop_dropped() {
        let r = ResolvedConfig {
            hooks: vec![make_hook(HookTrigger::SubagentStop, "cmd", None)],
            ..resolved(vec![])
        };
        let out = compile(&r, "gemini").unwrap();
        assert!(out.gemini_settings_patch.is_none(),
            "SubagentStop has no Gemini equivalent — patch must be None");
    }

    #[test]
    fn gemini_no_hooks_no_patch() {
        let out = compile(&resolved(vec![]), "gemini").unwrap();
        assert!(out.gemini_settings_patch.is_none());
    }

    #[test]
    fn gemini_hooks_not_emitted_for_other_providers() {
        let r = ResolvedConfig {
            hooks: vec![make_hook(HookTrigger::PreToolUse, "cmd", None)],
            ..resolved(vec![])
        };
        assert!(compile(&r, "claude").unwrap().gemini_settings_patch.is_none());
        assert!(compile(&r, "codex").unwrap().gemini_settings_patch.is_none());
        assert!(compile(&r, "cursor").unwrap().gemini_settings_patch.is_none());
    }

    // ── Cursor hooks ──────────────────────────────────────────────────────────

    #[test]
    fn cursor_pre_tool_emits_both_mcp_and_shell_events() {
        let r = ResolvedConfig {
            hooks: vec![make_hook(HookTrigger::PreToolUse, "ship check", Some("Bash"))],
            ..resolved(vec![])
        };
        let out = compile(&r, "cursor").unwrap();
        let patch = out.cursor_hooks_patch.expect("cursor must emit hooks patch");
        assert!(patch["beforeMCPExecution"].is_array(),   "PreToolUse → beforeMCPExecution");
        assert!(patch["beforeShellExecution"].is_array(), "PreToolUse → beforeShellExecution");
    }

    #[test]
    fn cursor_post_tool_emits_both_events() {
        let r = ResolvedConfig {
            hooks: vec![make_hook(HookTrigger::PostToolUse, "ship log", None)],
            ..resolved(vec![])
        };
        let out = compile(&r, "cursor").unwrap();
        let patch = out.cursor_hooks_patch.unwrap();
        assert!(patch["afterMCPExecution"].is_array());
        assert!(patch["afterShellExecution"].is_array());
    }

    #[test]
    fn cursor_stop_maps_to_session_end() {
        let r = ResolvedConfig {
            hooks: vec![make_hook(HookTrigger::Stop, "ship end", None)],
            ..resolved(vec![])
        };
        let out = compile(&r, "cursor").unwrap();
        let patch = out.cursor_hooks_patch.unwrap();
        assert!(patch["sessionEnd"].is_array());
    }

    #[test]
    fn cursor_unmapped_triggers_produce_no_patch() {
        // Notification, SubagentStop, PreCompact have no Cursor equivalent
        let r = ResolvedConfig {
            hooks: vec![
                make_hook(HookTrigger::Notification, "cmd", None),
                make_hook(HookTrigger::SubagentStop, "cmd", None),
                make_hook(HookTrigger::PreCompact,   "cmd", None),
            ],
            ..resolved(vec![])
        };
        let out = compile(&r, "cursor").unwrap();
        assert!(out.cursor_hooks_patch.is_none());
    }

    #[test]
    fn cursor_hooks_not_emitted_for_other_providers() {
        let r = ResolvedConfig {
            hooks: vec![make_hook(HookTrigger::PreToolUse, "cmd", None)],
            ..resolved(vec![])
        };
        assert!(compile(&r, "claude").unwrap().cursor_hooks_patch.is_none());
        assert!(compile(&r, "gemini").unwrap().cursor_hooks_patch.is_none());
        assert!(compile(&r, "codex").unwrap().cursor_hooks_patch.is_none());
    }

    // ── Cursor CLI permissions ────────────────────────────────────────────────

    /// Source: https://cursor.com/docs/cli/reference/permissions
    /// Default permissions (allow=[*], deny=[]) must not emit any patch.
    #[test]
    fn cursor_cli_permissions_default_is_none() {
        let out = compile(&resolved(vec![]), "cursor").unwrap();
        assert!(out.cursor_cli_permissions.is_none(),
            "default permissions must not emit a cursor cli permissions patch");
    }

    /// Deny-only patterns translate to Cursor CLI typed format and emit a patch.
    #[test]
    fn cursor_cli_permissions_deny_only_emits_patch() {
        let r = ResolvedConfig {
            permissions: Permissions {
                tools: ToolPermissions {
                    allow: vec!["*".to_string()],
                    deny: vec!["Bash(rm -rf *)".to_string(), "mcp__*__delete*".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..resolved(vec![])
        };
        let out = compile(&r, "cursor").unwrap();
        let patch = out.cursor_cli_permissions.expect("deny patterns must emit cursor cli patch");
        let deny = patch["permissions"]["deny"].as_array().unwrap();
        assert!(deny.iter().any(|v| v == "Shell(rm -rf *)"), "Bash(cmd) → Shell(cmd)");
        assert!(deny.iter().any(|v| v == "Mcp(*:delete*)"), "mcp__*__delete* → Mcp(*:delete*)");
        // allow=[*] must not emit an allow field
        assert!(patch["permissions"].get("allow").is_none(),
            "allow=[*] must not emit allow field in cursor cli permissions");
    }

    /// Explicit allow list translates Claude tool names to Cursor CLI typed format.
    #[test]
    fn cursor_cli_permissions_explicit_allow_translates() {
        let r = ResolvedConfig {
            permissions: Permissions {
                tools: ToolPermissions {
                    allow: vec!["Read".to_string(), "Bash(git *)".to_string(), "mcp__ship__*".to_string()],
                    deny: vec![],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..resolved(vec![])
        };
        let out = compile(&r, "cursor").unwrap();
        let patch = out.cursor_cli_permissions.expect("explicit allow must emit cursor cli patch");
        let allow = patch["permissions"]["allow"].as_array().unwrap();
        assert!(allow.iter().any(|v| v == "Read(*)" ), "Read → Read(*)" );
        assert!(allow.iter().any(|v| v == "Shell(git *)"), "Bash(git *) → Shell(git *)");
        assert!(allow.iter().any(|v| v == "Mcp(ship:*)"), "mcp__ship__* → Mcp(ship:*)");
    }

    /// Cursor CLI permissions are not emitted for other providers.
    #[test]
    fn cursor_cli_permissions_only_for_cursor() {
        let r = ResolvedConfig {
            permissions: Permissions {
                tools: ToolPermissions {
                    allow: vec!["*".to_string()],
                    deny: vec!["Bash(rm -rf *)".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..resolved(vec![])
        };
        assert!(compile(&r, "claude").unwrap().cursor_cli_permissions.is_none());
        assert!(compile(&r, "gemini").unwrap().cursor_cli_permissions.is_none());
        assert!(compile(&r, "codex").unwrap().cursor_cli_permissions.is_none());
    }

    /// `allow=[*]` must NEVER auto-expand to the permissive typed list.
    /// Cursor's default without config is interactive — allow-all must be an explicit
    /// UI choice using the CURSOR_PERMISSIVE_ALLOW constant.
    #[test]
    fn cursor_allow_star_never_auto_expands_to_permissive() {
        let r = ResolvedConfig {
            permissions: Permissions {
                tools: ToolPermissions {
                    allow: vec!["*".to_string()],
                    deny: vec![],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..resolved(vec![])
        };
        let out = compile(&r, "cursor").unwrap();
        assert!(out.cursor_cli_permissions.is_none(),
            "allow=[*] must not auto-emit cursor permissions — permissive must be explicit");
    }

    /// The CURSOR_PERMISSIVE_ALLOW constant covers all five typed permission types.
    #[test]
    fn cursor_permissive_allow_constant_covers_all_types() {
        let types = ["Shell", "Read", "Write", "WebFetch", "Mcp"];
        for t in types {
            assert!(
                CURSOR_PERMISSIVE_ALLOW.iter().any(|p| p.starts_with(t)),
                "CURSOR_PERMISSIVE_ALLOW must include a {t}(*) entry"
            );
        }
    }

    /// Unknown/unsupported tool patterns are silently dropped rather than emitting garbage.
    #[test]
    fn cursor_cli_permissions_unknown_patterns_dropped() {
        let r = ResolvedConfig {
            permissions: Permissions {
                tools: ToolPermissions {
                    allow: vec!["*".to_string()],
                    // NotebookEdit has no Cursor CLI equivalent
                    deny: vec!["NotebookEdit".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..resolved(vec![])
        };
        let out = compile(&r, "cursor").unwrap();
        // Unknown pattern dropped → no translatable patterns → None
        assert!(out.cursor_cli_permissions.is_none(),
            "unrecognised patterns with no translation must not emit a patch");
    }

    // ── Cross-provider round-trips ────────────────────────────────────────────

    /// Every provider must produce identical output when compiled twice from
    /// the same input — no timestamps, random IDs, or non-determinism.
    #[test]
    fn all_providers_are_idempotent() {
        use crate::types::Rule;
        let r = ResolvedConfig {
            mcp_servers: vec![make_server("github"), make_server("linear")],
            skills: vec![{
                let mut s = make_skill("workflow");
                s.content = "Do the thing.".into();
                s
            }],
            rules: vec![Rule { file_name: "style.md".into(), content: "Keep it clean.".into(), always_apply: true, globs: vec![], description: None }],
            hooks: vec![make_hook(HookTrigger::PreToolUse, "ship check", Some("Bash"))],
            ..resolved(vec![])
        };
        for provider in &["claude", "gemini", "codex", "cursor"] {
            let a = compile(&r, provider).unwrap();
            let b = compile(&r, provider).unwrap();
            assert_eq!(
                serde_json::to_string(&a.mcp_servers).unwrap(),
                serde_json::to_string(&b.mcp_servers).unwrap(),
                "{provider}: mcp_servers not idempotent"
            );
            assert_eq!(a.context_content, b.context_content, "{provider}: context not idempotent");
            assert_eq!(a.skill_files, b.skill_files, "{provider}: skill_files not idempotent");
            assert_eq!(
                serde_json::to_string(&a.claude_settings_patch).unwrap(),
                serde_json::to_string(&b.claude_settings_patch).unwrap(),
                "{provider}: claude_settings_patch not idempotent"
            );
            assert_eq!(a.codex_config_patch, b.codex_config_patch, "{provider}: codex_config_patch not idempotent");
            assert_eq!(
                serde_json::to_string(&a.gemini_settings_patch).unwrap(),
                serde_json::to_string(&b.gemini_settings_patch).unwrap(),
                "{provider}: gemini_settings_patch not idempotent"
            );
            assert_eq!(
                serde_json::to_string(&a.cursor_hooks_patch).unwrap(),
                serde_json::to_string(&b.cursor_hooks_patch).unwrap(),
                "{provider}: cursor_hooks_patch not idempotent"
            );
            assert_eq!(
                serde_json::to_string(&a.cursor_cli_permissions).unwrap(),
                serde_json::to_string(&b.cursor_cli_permissions).unwrap(),
                "{provider}: cursor_cli_permissions not idempotent"
            );
            assert_eq!(a.rule_files, b.rule_files, "{provider}: rule_files not idempotent");
            assert_eq!(a.gemini_policy_patch, b.gemini_policy_patch, "{provider}: gemini_policy_patch not idempotent");
        }
    }

    /// All four providers compile the same skill set correctly.
    #[test]
    fn all_providers_emit_skills() {
        let mut skill = make_skill("refactor");
        skill.content = "Refactor carefully.".into();
        let r = ResolvedConfig { skills: vec![skill], ..resolved(vec![]) };
        assert!(compile(&r, "claude").unwrap().skill_files.contains_key(".claude/skills/refactor/SKILL.md"));
        assert!(compile(&r, "gemini").unwrap().skill_files.contains_key(".agents/skills/refactor/SKILL.md"));
        assert!(compile(&r, "codex").unwrap().skill_files.contains_key(".agents/skills/refactor/SKILL.md"));
        assert!(compile(&r, "cursor").unwrap().skill_files.contains_key(".cursor/skills/refactor/SKILL.md"));
    }

    /// Claude, Gemini, Codex emit a context file when rules are present.
    /// Cursor uses per-file .mdc rules instead (rule_files) — no context_content.
    #[test]
    fn all_providers_emit_context_file() {
        use crate::types::Rule;
        let r = ResolvedConfig {
            rules: vec![Rule { file_name: "style.md".into(), content: "Write clean code.".into(), always_apply: true, globs: vec![], description: None }],
            ..resolved(vec![])
        };
        assert!(compile(&r, "claude").unwrap().context_content.is_some(), "claude needs CLAUDE.md");
        assert!(compile(&r, "gemini").unwrap().context_content.is_some(), "gemini needs GEMINI.md");
        assert!(compile(&r, "codex").unwrap().context_content.is_some(), "codex needs AGENTS.md");
        // Cursor uses rule_files (.mdc) instead of a single context file
        assert!(compile(&r, "cursor").unwrap().context_content.is_none(), "cursor uses rule_files, not context_content");
        assert!(!compile(&r, "cursor").unwrap().rule_files.is_empty(), "cursor must populate rule_files");
    }

    /// No provider emits a patch output it doesn't own.
    #[test]
    fn patch_outputs_are_provider_exclusive() {
        use crate::types::Rule;
        let r = ResolvedConfig {
            rules: vec![Rule { file_name: "r.md".into(), content: "x".into(), always_apply: true, globs: vec![], description: None }],
            hooks: vec![make_hook(HookTrigger::PreToolUse, "cmd", None)],
            permissions: Permissions {
                tools: ToolPermissions { deny: vec!["Bash(rm -rf *)".into()], ..Default::default() },
                ..Default::default()
            },
            ..resolved(vec![])
        };
        // claude_settings_patch: claude only
        assert!(compile(&r, "claude").unwrap().claude_settings_patch.is_some());
        for p in &["gemini", "codex", "cursor"] {
            assert!(compile(&r, p).unwrap().claude_settings_patch.is_none(), "{p} must not get claude patch");
        }
        // codex_config_patch: codex only
        assert!(compile(&r, "codex").unwrap().codex_config_patch.is_some());
        for p in &["claude", "gemini", "cursor"] {
            assert!(compile(&r, p).unwrap().codex_config_patch.is_none(), "{p} must not get codex patch");
        }
        // gemini_settings_patch: gemini only
        assert!(compile(&r, "gemini").unwrap().gemini_settings_patch.is_some());
        for p in &["claude", "codex", "cursor"] {
            assert!(compile(&r, p).unwrap().gemini_settings_patch.is_none(), "{p} must not get gemini patch");
        }
        // cursor_hooks_patch: cursor only
        assert!(compile(&r, "cursor").unwrap().cursor_hooks_patch.is_some());
        for p in &["claude", "gemini", "codex"] {
            assert!(compile(&r, p).unwrap().cursor_hooks_patch.is_none(), "{p} must not get cursor patch");
        }
        // cursor_cli_permissions: cursor only (deny pattern translates to cursor format)
        assert!(compile(&r, "cursor").unwrap().cursor_cli_permissions.is_some());
        for p in &["claude", "gemini", "codex"] {
            assert!(compile(&r, p).unwrap().cursor_cli_permissions.is_none(), "{p} must not get cursor cli permissions");
        }
        // rule_files: cursor only
        assert!(!compile(&r, "cursor").unwrap().rule_files.is_empty(), "cursor must have rule_files");
        for p in &["claude", "gemini", "codex"] {
            assert!(compile(&r, p).unwrap().rule_files.is_empty(), "{p} must not get rule_files");
        }
    }

    // ── Ask tier ─────────────────────────────────────────────────────────────

    #[test]
    fn ask_tier_compiles_to_permissions_ask() {
        let r = ResolvedConfig {
            permissions: Permissions {
                tools: ToolPermissions {
                    ask: vec!["mcp__*__delete*".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..resolved(vec![])
        };
        let out = compile(&r, "claude").unwrap();
        let patch = out.claude_settings_patch.expect("ask tier must emit a patch");
        let ask = patch["permissions"]["ask"].as_array().unwrap();
        assert_eq!(ask.len(), 1);
        assert_eq!(ask[0], "mcp__*__delete*");
    }

    #[test]
    fn ask_tier_default_empty_no_patch() {
        // Empty ask + default allow + empty deny = no patch
        let r = ResolvedConfig {
            permissions: Permissions {
                tools: ToolPermissions {
                    allow: vec!["*".to_string()],
                    ask: vec![],
                    deny: vec![],
                },
                ..Default::default()
            },
            ..resolved(vec![])
        };
        let out = compile(&r, "claude").unwrap();
        assert!(out.claude_settings_patch.is_none(), "empty ask with default allow/deny must not emit a patch");
    }

    #[test]
    fn ask_with_deny_both_emitted() {
        let r = ResolvedConfig {
            permissions: Permissions {
                tools: ToolPermissions {
                    ask: vec!["mcp__*__write*".to_string()],
                    deny: vec!["Bash(rm -rf *)".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..resolved(vec![])
        };
        let out = compile(&r, "claude").unwrap();
        let patch = out.claude_settings_patch.expect("ask + deny must emit a patch");
        let perms = &patch["permissions"];
        assert!(perms["ask"].as_array().unwrap().iter().any(|v| v == "mcp__*__write*"));
        assert!(perms["deny"].as_array().unwrap().iter().any(|v| v == "Bash(rm -rf *)"));
    }

    #[test]
    fn ask_tier_not_emitted_for_other_providers() {
        let r = ResolvedConfig {
            permissions: Permissions {
                tools: ToolPermissions {
                    ask: vec!["mcp__*__delete*".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..resolved(vec![])
        };
        assert!(compile(&r, "gemini").unwrap().claude_settings_patch.is_none());
        assert!(compile(&r, "codex").unwrap().claude_settings_patch.is_none());
        assert!(compile(&r, "cursor").unwrap().claude_settings_patch.is_none());
    }

    // ── defaultMode ───────────────────────────────────────────────────────────

    #[test]
    fn default_mode_compiles_to_permissions_default_mode() {
        let r = ResolvedConfig {
            permissions: Permissions {
                default_mode: Some("plan".to_string()),
                ..Default::default()
            },
            ..resolved(vec![])
        };
        let out = compile(&r, "claude").unwrap();
        let patch = out.claude_settings_patch.expect("default_mode must emit a patch");
        assert_eq!(patch["permissions"]["defaultMode"], "plan");
    }

    #[test]
    fn default_mode_none_omitted_from_patch() {
        let out = compile(&resolved(vec![]), "claude").unwrap();
        assert!(out.claude_settings_patch.is_none(), "no default_mode → no patch");
    }

    // ── model ─────────────────────────────────────────────────────────────────

    #[test]
    fn model_compiles_to_settings_patch() {
        let r = ResolvedConfig {
            model: Some("claude-opus-4-6".to_string()),
            ..resolved(vec![])
        };
        let out = compile(&r, "claude").unwrap();
        let patch = out.claude_settings_patch.expect("model must emit a patch");
        assert_eq!(patch["model"], "claude-opus-4-6");
    }

    #[test]
    fn model_none_omits_field() {
        let r = ResolvedConfig {
            model: None,
            ..resolved(vec![])
        };
        let out = compile(&r, "claude").unwrap();
        assert!(out.claude_settings_patch.is_none(), "None model must not emit a patch");
    }

    #[test]
    fn model_only_for_claude_not_other_providers() {
        let r = ResolvedConfig {
            model: Some("claude-opus-4-6".to_string()),
            ..resolved(vec![])
        };
        assert!(compile(&r, "gemini").unwrap().claude_settings_patch.is_none());
        assert!(compile(&r, "codex").unwrap().claude_settings_patch.is_none());
        assert!(compile(&r, "cursor").unwrap().claude_settings_patch.is_none());
    }

    // ── additionalDirectories ─────────────────────────────────────────────────

    #[test]
    fn additional_directories_emitted_when_set() {
        let r = ResolvedConfig {
            permissions: Permissions {
                additional_directories: vec!["/tmp/project".to_string()],
                ..Default::default()
            },
            ..resolved(vec![])
        };
        let out = compile(&r, "claude").unwrap();
        let patch = out.claude_settings_patch.expect("additionalDirectories must emit a patch");
        let dirs = patch["permissions"]["additionalDirectories"].as_array().unwrap();
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0], "/tmp/project");
    }

    #[test]
    fn additional_directories_empty_omitted() {
        let r = ResolvedConfig {
            permissions: Permissions {
                additional_directories: vec![],
                ..Default::default()
            },
            ..resolved(vec![])
        };
        let out = compile(&r, "claude").unwrap();
        assert!(out.claude_settings_patch.is_none(), "empty additional_directories must not emit a patch");
    }

    // ── Cursor rule_files ──────────────────────────────────────────────────────

    #[test]
    fn cursor_writes_per_file_mdc_rules() {
        let r = ResolvedConfig {
            rules: vec![
                make_rule("style.md", "Use consistent naming."),
                make_rule("workflow.md", "Run tests before commit."),
            ],
            ..resolved(vec![])
        };
        let out = compile(&r, "cursor").unwrap();
        assert!(out.rule_files.contains_key(".cursor/rules/style.mdc"), "style rule must be .cursor/rules/style.mdc");
        assert!(out.rule_files.contains_key(".cursor/rules/workflow.mdc"), "workflow rule must be .cursor/rules/workflow.mdc");
    }

    #[test]
    fn cursor_rule_file_has_frontmatter() {
        let r = ResolvedConfig {
            rules: vec![make_rule("style.md", "Use consistent naming.")],
            ..resolved(vec![])
        };
        let out = compile(&r, "cursor").unwrap();
        let content = &out.rule_files[".cursor/rules/style.mdc"];
        assert!(content.starts_with("---\n"), "cursor rule file must start with YAML frontmatter");
        assert!(content.contains("---\n\n"), "must have closing frontmatter delimiter");
    }

    #[test]
    fn cursor_rule_alwaysapply_false_in_frontmatter() {
        let r = ResolvedConfig {
            rules: vec![Rule {
                file_name: "conditional.md".to_string(),
                content: "Only when relevant.".to_string(),
                always_apply: false,
                globs: vec![],
                description: None,
            }],
            ..resolved(vec![])
        };
        let out = compile(&r, "cursor").unwrap();
        let content = &out.rule_files[".cursor/rules/conditional.mdc"];
        assert!(content.contains("alwaysApply: false"), "alwaysApply: false must appear in frontmatter");
    }

    #[test]
    fn cursor_rule_globs_in_frontmatter() {
        let r = ResolvedConfig {
            rules: vec![Rule {
                file_name: "rust-only.md".to_string(),
                content: "Rust-specific conventions.".to_string(),
                always_apply: false,
                globs: vec!["**/*.rs".to_string(), "Cargo.toml".to_string()],
                description: None,
            }],
            ..resolved(vec![])
        };
        let out = compile(&r, "cursor").unwrap();
        let content = &out.rule_files[".cursor/rules/rust-only.mdc"];
        assert!(content.contains("globs:"), "globs section must appear");
        assert!(content.contains("**/*.rs"), "glob pattern must appear");
        assert!(content.contains("Cargo.toml"), "second glob must appear");
    }

    #[test]
    fn cursor_rule_description_in_frontmatter() {
        let r = ResolvedConfig {
            rules: vec![Rule {
                file_name: "smart.md".to_string(),
                content: "Apply intelligently.".to_string(),
                always_apply: false,
                globs: vec![],
                description: Some("Apply when writing React components".to_string()),
            }],
            ..resolved(vec![])
        };
        let out = compile(&r, "cursor").unwrap();
        let content = &out.rule_files[".cursor/rules/smart.mdc"];
        assert!(content.contains("description:"), "description must appear in frontmatter");
        assert!(content.contains("Apply when writing React components"), "description value must appear");
    }

    #[test]
    fn cursor_no_context_content() {
        let r = ResolvedConfig {
            rules: vec![make_rule("style.md", "Use consistent naming.")],
            ..resolved(vec![])
        };
        let out = compile(&r, "cursor").unwrap();
        assert!(out.context_content.is_none(), "cursor must not have context_content — uses rule_files instead");
    }

    #[test]
    fn cursor_empty_rule_content_skipped_in_rule_files() {
        let r = ResolvedConfig {
            rules: vec![
                make_rule("empty.md", "   "),
                make_rule("real.md", "Actual content here."),
            ],
            ..resolved(vec![])
        };
        let out = compile(&r, "cursor").unwrap();
        assert!(!out.rule_files.contains_key(".cursor/rules/empty.mdc"), "empty rule content must not produce a file");
        assert!(out.rule_files.contains_key(".cursor/rules/real.mdc"), "non-empty rule must produce a file");
    }

    #[test]
    fn other_providers_have_no_rule_files() {
        let r = ResolvedConfig {
            rules: vec![make_rule("style.md", "Use consistent naming.")],
            ..resolved(vec![])
        };
        assert!(compile(&r, "claude").unwrap().rule_files.is_empty(), "claude must not have rule_files");
        assert!(compile(&r, "gemini").unwrap().rule_files.is_empty(), "gemini must not have rule_files");
        assert!(compile(&r, "codex").unwrap().rule_files.is_empty(), "codex must not have rule_files");
    }

    #[test]
    fn cursor_version_1_in_cli_permissions() {
        let r = ResolvedConfig {
            permissions: Permissions {
                tools: ToolPermissions {
                    allow: vec!["*".to_string()],
                    deny: vec!["Bash(rm -rf *)".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..resolved(vec![])
        };
        let out = compile(&r, "cursor").unwrap();
        let patch = out.cursor_cli_permissions.expect("deny must emit cursor cli permissions");
        assert_eq!(patch["version"], 1, "cursor cli permissions must include version: 1");
    }

    // ── Gemini policy patch ───────────────────────────────────────────────────

    /// Default permissions → no policy patch.
    /// Gemini should not receive any `.gemini/policies/ship.toml` unless
    /// the user has explicitly configured allow/ask/deny overrides.
    #[test]
    fn gemini_policy_default_permissions_emit_none() {
        let out = compile(&resolved(vec![]), "gemini").unwrap();
        assert!(
            out.gemini_policy_patch.is_none(),
            "default permissions must not emit a gemini policy patch"
        );
    }

    /// allow=[*] deny=[] is the identity case — no policy needed.
    #[test]
    fn gemini_policy_allow_star_no_deny_is_none() {
        let r = ResolvedConfig {
            permissions: Permissions {
                tools: ToolPermissions {
                    allow: vec!["*".to_string()],
                    deny: vec![],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..resolved(vec![])
        };
        let out = compile(&r, "gemini").unwrap();
        assert!(out.gemini_policy_patch.is_none());
    }

    /// Deny patterns → `decision = "deny"` in TOML output.
    #[test]
    fn gemini_policy_deny_translates_to_deny_decision() {
        let r = ResolvedConfig {
            permissions: Permissions {
                tools: ToolPermissions {
                    deny: vec!["Bash(rm -rf *)".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..resolved(vec![])
        };
        let out = compile(&r, "gemini").unwrap();
        let toml_str = out.gemini_policy_patch.expect("deny must emit gemini policy patch");
        let parsed: toml::Table = toml::from_str(&toml_str).expect("must be valid TOML");
        let policies = parsed["tool_policies"].as_array().expect("must have tool_policies array");
        assert_eq!(policies.len(), 1);
        assert_eq!(policies[0]["tool"].as_str().unwrap(), "shell");
        assert_eq!(policies[0]["decision"].as_str().unwrap(), "deny");
        // Bash(rm -rf *) pattern → glob-escaped regex
        let pattern = policies[0]["pattern"].as_str().unwrap();
        assert!(pattern.contains("rm"), "pattern must encode the command");
    }

    /// Ask patterns → `decision = "ask_user"` in TOML output.
    #[test]
    fn gemini_policy_ask_translates_to_ask_user() {
        let r = ResolvedConfig {
            permissions: Permissions {
                tools: ToolPermissions {
                    ask: vec!["mcp__*__delete*".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..resolved(vec![])
        };
        let out = compile(&r, "gemini").unwrap();
        let toml_str = out.gemini_policy_patch.expect("ask must emit gemini policy patch");
        let parsed: toml::Table = toml::from_str(&toml_str).expect("valid TOML");
        let policies = parsed["tool_policies"].as_array().unwrap();
        assert_eq!(policies.len(), 1);
        assert_eq!(policies[0]["tool"].as_str().unwrap(), "mcp");
        assert_eq!(policies[0]["decision"].as_str().unwrap(), "ask_user");
    }

    /// Non-default allow list → `decision = "allow"` entries.
    #[test]
    fn gemini_policy_explicit_allow_translates_to_allow_decision() {
        let r = ResolvedConfig {
            permissions: Permissions {
                tools: ToolPermissions {
                    allow: vec!["Bash(git *)".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..resolved(vec![])
        };
        let out = compile(&r, "gemini").unwrap();
        let toml_str = out.gemini_policy_patch.expect("explicit allow must emit gemini policy patch");
        let parsed: toml::Table = toml::from_str(&toml_str).expect("valid TOML");
        let policies = parsed["tool_policies"].as_array().unwrap();
        assert_eq!(policies.len(), 1);
        assert_eq!(policies[0]["tool"].as_str().unwrap(), "shell");
        assert_eq!(policies[0]["decision"].as_str().unwrap(), "allow");
        let pattern = policies[0]["pattern"].as_str().unwrap();
        assert!(pattern.contains("git"), "pattern must encode the git command");
    }

    /// MCP patterns (mcp__server__tool) translate to `tool = "mcp"` with a regex pattern.
    #[test]
    fn gemini_policy_mcp_pattern_translated() {
        let r = ResolvedConfig {
            permissions: Permissions {
                tools: ToolPermissions {
                    deny: vec!["mcp__github__delete_issue".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..resolved(vec![])
        };
        let out = compile(&r, "gemini").unwrap();
        let toml_str = out.gemini_policy_patch.expect("mcp deny must emit gemini policy patch");
        let parsed: toml::Table = toml::from_str(&toml_str).expect("valid TOML");
        let policies = parsed["tool_policies"].as_array().unwrap();
        assert_eq!(policies[0]["tool"].as_str().unwrap(), "mcp");
        let pattern = policies[0]["pattern"].as_str().unwrap();
        assert!(pattern.contains("github"), "mcp pattern must include server name");
        assert!(pattern.contains("delete_issue"), "mcp pattern must include tool name");
    }

    /// Read/Glob → file_read, Write/Edit → file_write, WebFetch → web_fetch.
    #[test]
    fn gemini_policy_file_and_web_tool_mapping() {
        let r = ResolvedConfig {
            permissions: Permissions {
                tools: ToolPermissions {
                    deny: vec![
                        "Read".to_string(),
                        "Write".to_string(),
                        "WebFetch".to_string(),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..resolved(vec![])
        };
        let out = compile(&r, "gemini").unwrap();
        let toml_str = out.gemini_policy_patch.expect("must emit gemini policy patch");
        let parsed: toml::Table = toml::from_str(&toml_str).expect("valid TOML");
        let policies = parsed["tool_policies"].as_array().unwrap();
        assert_eq!(policies.len(), 3);
        let tools: Vec<&str> = policies.iter().map(|p| p["tool"].as_str().unwrap()).collect();
        assert!(tools.contains(&"file_read"),  "Read → file_read");
        assert!(tools.contains(&"file_write"), "Write → file_write");
        assert!(tools.contains(&"web_fetch"),  "WebFetch → web_fetch");
    }

    /// Bare `*` wildcard in deny must not produce a policy entry — it's the default allow-all.
    #[test]
    fn gemini_policy_bare_wildcard_dropped() {
        let r = ResolvedConfig {
            permissions: Permissions {
                tools: ToolPermissions {
                    deny: vec!["*".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..resolved(vec![])
        };
        let out = compile(&r, "gemini").unwrap();
        assert!(out.gemini_policy_patch.is_none(),
            "bare * in deny must not produce a policy entry");
    }

    /// Policy patch is valid TOML with the `[[tool_policies]]` array-of-tables structure.
    #[test]
    fn gemini_policy_output_is_valid_toml() {
        let r = ResolvedConfig {
            permissions: Permissions {
                tools: ToolPermissions {
                    deny: vec!["Bash(rm -rf *)".to_string(), "mcp__*__delete*".to_string()],
                    ask: vec!["Write".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..resolved(vec![])
        };
        let out = compile(&r, "gemini").unwrap();
        let toml_str = out.gemini_policy_patch.expect("must emit policy patch");
        let parsed: toml::Table = toml::from_str(&toml_str).expect("must be valid TOML");
        assert!(parsed.contains_key("tool_policies"), "must use [[tool_policies]] array");
        let policies = parsed["tool_policies"].as_array().unwrap();
        assert_eq!(policies.len(), 3, "deny(2) + ask(1) = 3 entries");
    }

    /// Gemini policy patch is only emitted for the gemini provider.
    #[test]
    fn gemini_policy_only_for_gemini_not_other_providers() {
        let r = ResolvedConfig {
            permissions: Permissions {
                tools: ToolPermissions {
                    deny: vec!["Bash(rm -rf *)".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..resolved(vec![])
        };
        assert!(compile(&r, "claude").unwrap().gemini_policy_patch.is_none(), "claude must not get gemini policy patch");
        assert!(compile(&r, "codex").unwrap().gemini_policy_patch.is_none(),  "codex must not get gemini policy patch");
        assert!(compile(&r, "cursor").unwrap().gemini_policy_patch.is_none(), "cursor must not get gemini policy patch");
        assert!(compile(&r, "gemini").unwrap().gemini_policy_patch.is_some(), "gemini must get the policy patch");
    }

    /// Glob wildcard `*` is converted to `.*` in the regex output.
    #[test]
    fn gemini_policy_glob_star_converts_to_regex_dotstar() {
        let r = ResolvedConfig {
            permissions: Permissions {
                tools: ToolPermissions {
                    deny: vec!["Bash(git *)".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..resolved(vec![])
        };
        let out = compile(&r, "gemini").unwrap();
        let toml_str = out.gemini_policy_patch.unwrap();
        let parsed: toml::Table = toml::from_str(&toml_str).unwrap();
        let pattern = parsed["tool_policies"].as_array().unwrap()[0]["pattern"].as_str().unwrap();
        assert!(pattern.contains(".*"), "glob * must become .* in regex");
        assert!(!pattern.contains(" *"), "raw glob space-star must not remain");
    }

    // ── Context file audit ────────────────────────────────────────────────────

    #[test]
    fn claude_context_content_contains_only_rules_not_skills() {
        let r = ResolvedConfig {
            rules: vec![make_rule("rule.md", "Rule content here.")],
            skills: vec![make_skill("my-skill")],
            ..resolved(vec![])
        };
        let out = compile(&r, "claude").unwrap();
        let content = out.context_content.expect("claude must have context_content when rules present");
        assert!(content.contains("Rule content here."), "rules must appear in context");
        assert!(!content.contains("Do the thing."), "skill content must NOT appear in context file");
    }

    #[test]
    fn gemini_context_content_contains_only_rules() {
        let r = ResolvedConfig {
            rules: vec![make_rule("rule.md", "Gemini rule content.")],
            skills: vec![make_skill("my-skill")],
            ..resolved(vec![])
        };
        let out = compile(&r, "gemini").unwrap();
        let content = out.context_content.expect("gemini must have context_content");
        assert!(content.contains("Gemini rule content."));
        assert!(!content.contains("Do the thing."), "skill content must not leak into gemini context");
    }

    #[test]
    fn codex_context_content_contains_only_rules() {
        let r = ResolvedConfig {
            rules: vec![make_rule("rule.md", "Codex rule content.")],
            skills: vec![make_skill("my-skill")],
            ..resolved(vec![])
        };
        let out = compile(&r, "codex").unwrap();
        let content = out.context_content.expect("codex must have context_content");
        assert!(content.contains("Codex rule content."));
        assert!(!content.contains("Do the thing."), "skill content must not leak into codex context");
    }

    #[test]
    fn context_file_is_none_when_all_rule_content_empty() {
        let r = ResolvedConfig {
            rules: vec![
                make_rule("blank.md", ""),
                make_rule("whitespace.md", "   \n  "),
            ],
            ..resolved(vec![])
        };
        // For text-based context providers (claude, gemini, codex) — all empty rules
        // should still produce a context file (they include the trimmed whitespace)
        // but with empty content the parts vec is empty → None
        let claude_out = compile(&r, "claude").unwrap();
        assert!(claude_out.context_content.is_none(), "all-blank rules must not produce a claude context file");
        let gemini_out = compile(&r, "gemini").unwrap();
        assert!(gemini_out.context_content.is_none(), "all-blank rules must not produce a gemini context file");
        let codex_out = compile(&r, "codex").unwrap();
        assert!(codex_out.context_content.is_none(), "all-blank rules must not produce a codex context file");
    }
}
