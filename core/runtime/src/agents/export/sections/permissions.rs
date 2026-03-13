// ─── Hooks + permissions (provider-native mappings) ──────────────────────────

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Default)]
struct GeminiPolicyDoc {
    #[serde(rename = "rule", default)]
    rules: Vec<GeminiPolicyRule>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Default)]
struct GeminiPolicyRule {
    #[serde(rename = "toolName", skip_serializing_if = "Option::is_none")]
    tool_name: Option<String>,
    #[serde(rename = "mcpName", skip_serializing_if = "Option::is_none")]
    mcp_name: Option<String>,
    #[serde(rename = "commandPrefix", skip_serializing_if = "Option::is_none")]
    command_prefix: Option<String>,
    #[serde(rename = "commandRegex", skip_serializing_if = "Option::is_none")]
    command_regex: Option<String>,
    decision: String,
    priority: i32,
}

fn is_default_tool_permissions(permissions: &Permissions) -> bool {
    (permissions.tools.allow.is_empty()
        || (permissions.tools.allow.len() == 1 && permissions.tools.allow[0] == "*"))
        && permissions.tools.deny.is_empty()
}

fn has_claude_permission_overrides(permissions: &Permissions) -> bool {
    !is_default_tool_permissions(permissions)
}

fn has_gemini_policy_overrides(permissions: &Permissions) -> bool {
    !is_default_tool_permissions(permissions)
        || !permissions.commands.allow.is_empty()
        || !permissions.commands.deny.is_empty()
        || !permissions.agent.require_confirmation.is_empty()
}

fn managed_hooks_enabled() -> bool {
    match std::env::var("SHIP_MANAGED_HOOKS") {
        Ok(raw) => !matches!(
            raw.trim().to_ascii_lowercase().as_str(),
            "0" | "false" | "off" | "no"
        ),
        Err(_) => true,
    }
}

fn managed_hooks_command() -> Option<String> {
    if let Some(explicit) = std::env::var("SHIP_HOOKS_BIN")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        return Some(explicit);
    }

    let probe = std::process::Command::new("ship")
        .args(["hooks", "run", "--help"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    match probe {
        Ok(status) if status.success() => Some("ship hooks run".to_string()),
        _ => None,
    }
}

fn managed_hooks_command_for_provider(provider_id: &str) -> Option<String> {
    let base = managed_hooks_command()?;
    if base.contains("--provider") {
        return Some(base);
    }
    if base.trim_start().starts_with("ship hooks run") {
        return Some(format!("{base} --provider {provider_id}"));
    }
    Some(base)
}

fn managed_hook(
    id: &str,
    trigger: HookTrigger,
    matcher: Option<&str>,
    timeout_ms: Option<u64>,
    description: &str,
    command: &str,
) -> HookConfig {
    HookConfig {
        id: id.to_string(),
        trigger,
        matcher: matcher.map(str::to_string),
        timeout_ms,
        description: Some(description.to_string()),
        command: command.to_string(),
    }
}

fn managed_hooks_for_provider(provider_id: &str) -> Vec<HookConfig> {
    let Some(command) = managed_hooks_command_for_provider(provider_id) else {
        return Vec::new();
    };
    match provider_id {
        "claude" => vec![
            managed_hook(
                "ship-session-start",
                HookTrigger::SessionStart,
                None,
                None,
                "Inject Ship workspace context before first prompt.",
                &command,
            ),
            managed_hook(
                "ship-user-prompt",
                HookTrigger::UserPromptSubmit,
                None,
                None,
                "Augment prompts with current Ship workspace scope.",
                &command,
            ),
            managed_hook(
                "ship-pre-tool-guard",
                HookTrigger::PreToolUse,
                Some("Bash"),
                Some(2000),
                "Apply Ship shell-command policy envelope (decompose, validate, enforce).",
                &command,
            ),
            managed_hook(
                "ship-permission-request",
                HookTrigger::PermissionRequest,
                None,
                Some(2000),
                "Resolve approvals using Ship permission envelope hints.",
                &command,
            ),
            managed_hook(
                "ship-post-tool-log",
                HookTrigger::PostToolUse,
                Some("Bash"),
                Some(1500),
                "Log tool execution for policy hardening and conflict analysis.",
                &command,
            ),
            managed_hook(
                "ship-notification-stream",
                HookTrigger::Notification,
                None,
                None,
                "Stream agent lifecycle updates to Ship runtime telemetry.",
                &command,
            ),
            managed_hook(
                "ship-stop-close-loop",
                HookTrigger::Stop,
                None,
                None,
                "Trigger session close-loop checks and documentation updates.",
                &command,
            ),
            managed_hook(
                "ship-subagent-stop",
                HookTrigger::SubagentStop,
                None,
                None,
                "Coordinate multi-agent completion signals through Ship runtime.",
                &command,
            ),
        ],
        "gemini" => vec![
            managed_hook(
                "ship-session-start",
                HookTrigger::SessionStart,
                None,
                None,
                "Inject Ship workspace context at Gemini session start.",
                &command,
            ),
            managed_hook(
                "ship-before-tool-guard",
                HookTrigger::BeforeTool,
                Some("run_shell_command"),
                Some(2000),
                "Apply Ship shell-command policy envelope (decompose, validate, enforce).",
                &command,
            ),
            managed_hook(
                "ship-after-tool-log",
                HookTrigger::AfterTool,
                Some("run_shell_command"),
                Some(1500),
                "Log tool execution for policy hardening and conflict analysis.",
                &command,
            ),
            managed_hook(
                "ship-notification-stream",
                HookTrigger::Notification,
                None,
                None,
                "Stream agent lifecycle updates to Ship runtime telemetry.",
                &command,
            ),
            managed_hook(
                "ship-session-end-close-loop",
                HookTrigger::SessionEnd,
                None,
                None,
                "Trigger session close-loop checks and documentation updates.",
                &command,
            ),
        ],
        _ => Vec::new(),
    }
}

fn hooks_for_provider(provider_id: &str, hooks: &[HookConfig]) -> Vec<HookConfig> {
    let mut merged = hooks.to_vec();
    if !managed_hooks_enabled() {
        return merged;
    }

    let existing_ids: HashSet<String> = merged.iter().map(|hook| hook.id.clone()).collect();
    for managed in managed_hooks_for_provider(provider_id) {
        if !existing_ids.contains(&managed.id) {
            merged.push(managed);
        }
    }
    merged
}

fn network_allowed(policy: &crate::permissions::NetworkPolicy) -> bool {
    !matches!(policy, crate::permissions::NetworkPolicy::None)
}

fn command_installs_allowed(allow_patterns: &[String]) -> bool {
    allow_patterns.iter().any(|pattern| {
        let normalized = pattern.trim().to_ascii_lowercase();
        normalized.starts_with("npm install")
            || normalized.starts_with("pnpm add")
            || normalized.starts_with("yarn add")
            || normalized.starts_with("pip install")
            || normalized.starts_with("uv pip install")
            || normalized.starts_with("cargo add")
            || normalized.starts_with("go get")
            || normalized.starts_with("brew install")
    })
}

fn dedupe_patterns(values: &mut Vec<String>) {
    let mut seen = HashSet::new();
    values.retain(|value| seen.insert(value.clone()));
}

fn write_hook_runtime_artifacts(project_root: &Path, payload: &SyncPayload) -> Result<()> {
    let runtime_dir = project_root.join(".ship").join("generated").join("runtime");
    fs::create_dir_all(&runtime_dir)?;

    let generated_at = chrono::Utc::now().to_rfc3339();
    let mut auto_approve_patterns = vec![
        "find *".to_string(),
        "grep *".to_string(),
        "rg *".to_string(),
        "cat *".to_string(),
        "ls *".to_string(),
        "git status*".to_string(),
        "git log*".to_string(),
        "git diff*".to_string(),
    ];
    auto_approve_patterns.extend(payload.permissions.commands.allow.clone());
    dedupe_patterns(&mut auto_approve_patterns);

    let mut always_block_patterns = vec![
        "rm -rf *".to_string(),
        "git push --force*".to_string(),
        "npm publish*".to_string(),
        "cargo publish*".to_string(),
    ];
    always_block_patterns.extend(payload.permissions.commands.deny.clone());
    dedupe_patterns(&mut always_block_patterns);

    let mut allowed_paths = payload.permissions.filesystem.allow.clone();
    if allowed_paths.is_empty() {
        allowed_paths.push(".".to_string());
    }

    let servers = payload
        .servers
        .iter()
        .filter(|server| !server.disabled)
        .map(|server| {
            serde_json::json!({
                "id": server.id,
                "name": server.name,
                "transport": match server.server_type {
                    McpServerType::Stdio => "stdio",
                    McpServerType::Sse => "sse",
                    McpServerType::Http => "http",
                },
            })
        })
        .collect::<Vec<_>>();

    let envelope = serde_json::json!({
        "_ship": {
            "managed": true,
            "version": 1,
            "generated_at": generated_at,
        },
        "ship_first": true,
        "workspace_root": project_root.to_string_lossy().to_string(),
        "active_mode": payload.active_mode_id,
        "allowed_paths": allowed_paths,
        "allow_network": network_allowed(&payload.permissions.network.policy),
        "allow_installs": command_installs_allowed(&payload.permissions.commands.allow),
        "auto_approve_patterns": auto_approve_patterns,
        "always_block_patterns": always_block_patterns,
        "require_confirmation": payload.permissions.agent.require_confirmation.clone(),
        "tools_allow": payload.permissions.tools.allow.clone(),
        "tools_deny": payload.permissions.tools.deny.clone(),
        "mcp_servers": servers,
    });

    crate::fs_util::write_atomic(
        &runtime_dir.join("envelope.json"),
        serde_json::to_string_pretty(&envelope)?,
    )?;

    let context = format!(
        "# Ship Hook Context\n\n\
         - Generated: `{}`\n\
         - Active mode: `{}`\n\
         - MCP servers: `{}`\n\
         - Network access: `{}`\n\
         - Package installs: `{}`\n\n\
         ## Execution Policy\n\
         - Use `mcp__ship__*` tools first for workspace-aware operations.\n\
         - Prefer structured MCP actions over raw shell commands whenever possible.\n\
         - Treat direct shell usage as fallback and stay within declared scope.\n\n\
         ## File Scope\n\
         {}\n\n\
         ## Command Policy\n\
         - Allow patterns: `{}`\n\
         - Deny patterns: `{}`\n\
         - Require confirmation: `{}`\n",
        generated_at,
        payload
            .active_mode_id
            .as_deref()
            .unwrap_or("default"),
        payload
            .servers
            .iter()
            .filter(|server| !server.disabled)
            .map(|server| server.id.as_str())
            .collect::<Vec<_>>()
            .join(", "),
        if network_allowed(&payload.permissions.network.policy) {
            "enabled"
        } else {
            "disabled"
        },
        if command_installs_allowed(&payload.permissions.commands.allow) {
            "allowed"
        } else {
            "blocked"
        },
        payload
            .permissions
            .filesystem
            .allow
            .iter()
            .map(|path| format!("- `{}`", path))
            .collect::<Vec<_>>()
            .join("\n"),
        payload.permissions.commands.allow.join(", "),
        payload.permissions.commands.deny.join(", "),
        payload.permissions.agent.require_confirmation.join(", "),
    );
    crate::fs_util::write_atomic(&runtime_dir.join("hook-context.md"), context)?;

    // Best-effort cleanup of legacy runtime path from older builds.
    let legacy_runtime_dir = project_root.join(".ship").join("agents").join("runtime");
    for name in ["envelope.json", "hook-context.md"] {
        let _ = fs::remove_file(legacy_runtime_dir.join(name));
    }
    let _ = fs::remove_dir(&legacy_runtime_dir);
    Ok(())
}

fn export_claude_settings(
    project_root: &Path,
    hooks: &[HookConfig],
    permissions: &Permissions,
) -> Result<()> {
    let path = project_root.join(".claude").join("settings.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut root: serde_json::Value = if path.exists() {
        serde_json::from_str(&fs::read_to_string(&path)?).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    let obj = root
        .as_object_mut()
        .ok_or_else(|| anyhow!(".claude/settings.json is not an object"))?;

    if has_claude_permission_overrides(permissions) {
        let perms = obj.entry("permissions").or_insert(serde_json::json!({}));
        let p = perms
            .as_object_mut()
            .ok_or_else(|| anyhow!("permissions not an object"))?;
        p.insert("allow".into(), serde_json::json!(permissions.tools.allow));
        p.insert("deny".into(), serde_json::json!(permissions.tools.deny));
    }

    let should_reconcile_hooks = !hooks.is_empty() || obj.get("hooks").is_some();
    if should_reconcile_hooks {
        let hooks_val = obj.entry("hooks").or_insert(serde_json::json!({}));
        let hooks_map = hooks_val
            .as_object_mut()
            .ok_or_else(|| anyhow!("hooks not an object"))?;
        let mut by_trigger: HashMap<&str, Vec<serde_json::Value>> = HashMap::new();
        for hook in hooks {
            let Some(key) = claude_trigger_name(&hook.trigger) else {
                continue;
            };
            let mut command_hook = serde_json::json!({
                "type": "command",
                "command": hook.command,
            });
            if let Some(timeout) = hook.timeout_ms {
                command_hook["timeout"] = serde_json::json!(timeout);
            }
            if let Some(description) = &hook.description {
                command_hook["description"] = serde_json::json!(description);
            }
            let mut group = serde_json::json!({
                "hooks": [command_hook]
            });
            if let Some(m) = &hook.matcher {
                group["matcher"] = serde_json::json!(m);
            }
            by_trigger.entry(key).or_default().push(group);
        }

        // Keep Claude hook triggers in sync with the active hook set to avoid stale keys.
        for trigger in claude_managed_trigger_keys() {
            if let Some(entries) = by_trigger.remove(trigger) {
                hooks_map.insert((*trigger).to_string(), serde_json::json!(entries));
            } else {
                hooks_map.remove(*trigger);
            }
        }

        for (trigger, entries) in by_trigger {
            hooks_map.insert(trigger.to_string(), serde_json::json!(entries));
        }

        if hooks_map.is_empty() {
            obj.remove("hooks");
        }
    }

    crate::fs_util::write_atomic(&path, serde_json::to_string_pretty(&root)?)
}

fn export_gemini_settings(project_root: &Path, hooks: &[HookConfig]) -> Result<()> {
    if hooks.is_empty() {
        return Ok(());
    }

    let path = project_root.join(".gemini").join("settings.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut root: serde_json::Value = if path.exists() {
        serde_json::from_str(&fs::read_to_string(&path)?).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    let obj = root
        .as_object_mut()
        .ok_or_else(|| anyhow!(".gemini/settings.json is not an object"))?;
    let hooks_val = obj.entry("hooks").or_insert(serde_json::json!({}));
    let hooks_map = hooks_val
        .as_object_mut()
        .ok_or_else(|| anyhow!("hooks not an object"))?;

    let mut by_trigger: HashMap<&str, Vec<serde_json::Value>> = HashMap::new();
    for hook in hooks {
        let Some(key) = gemini_trigger_name(&hook.trigger) else {
            continue;
        };
        let mut command_hook = serde_json::json!({
            "name": hook.id,
            "type": "command",
            "command": hook.command,
        });
        if let Some(timeout) = hook.timeout_ms {
            command_hook["timeout"] = serde_json::json!(timeout);
        }
        if let Some(description) = &hook.description {
            command_hook["description"] = serde_json::json!(description);
        }
        let mut group = serde_json::json!({
            "hooks": [command_hook]
        });
        if let Some(matcher) = &hook.matcher {
            group["matcher"] = serde_json::json!(matcher);
        }
        by_trigger.entry(key).or_default().push(group);
    }

    for (trigger, entries) in by_trigger {
        hooks_map.insert(trigger.to_string(), serde_json::json!(entries));
    }
    crate::fs_util::write_atomic(&path, serde_json::to_string_pretty(&root)?)
}

fn claude_trigger_name(trigger: &HookTrigger) -> Option<&'static str> {
    match trigger {
        HookTrigger::SessionStart => Some("SessionStart"),
        HookTrigger::UserPromptSubmit => Some("UserPromptSubmit"),
        HookTrigger::PreToolUse | HookTrigger::BeforeTool => Some("PreToolUse"),
        HookTrigger::PermissionRequest => Some("PermissionRequest"),
        HookTrigger::PostToolUse | HookTrigger::AfterTool => Some("PostToolUse"),
        HookTrigger::PostToolUseFailure => Some("PostToolUseFailure"),
        HookTrigger::Notification => Some("Notification"),
        HookTrigger::SubagentStart => Some("SubagentStart"),
        HookTrigger::SubagentStop => Some("SubagentStop"),
        HookTrigger::Stop | HookTrigger::SessionEnd => Some("Stop"),
        HookTrigger::PreCompact => Some("PreCompact"),
        HookTrigger::BeforeAgent
        | HookTrigger::AfterAgent
        | HookTrigger::BeforeModel
        | HookTrigger::AfterModel
        | HookTrigger::BeforeToolSelection => None,
    }
}

fn claude_managed_trigger_keys() -> &'static [&'static str] {
    &[
        "SessionStart",
        "UserPromptSubmit",
        "PreToolUse",
        "PermissionRequest",
        "PostToolUse",
        "PostToolUseFailure",
        "Notification",
        "SubagentStart",
        "SubagentStop",
        "Stop",
        "PreCompact",
    ]
}

fn gemini_trigger_name(trigger: &HookTrigger) -> Option<&'static str> {
    match trigger {
        HookTrigger::BeforeTool | HookTrigger::PreToolUse => Some("BeforeTool"),
        HookTrigger::AfterTool | HookTrigger::PostToolUse => Some("AfterTool"),
        HookTrigger::BeforeAgent => Some("BeforeAgent"),
        HookTrigger::AfterAgent => Some("AfterAgent"),
        HookTrigger::Notification => Some("Notification"),
        HookTrigger::SessionStart => Some("SessionStart"),
        HookTrigger::SessionEnd | HookTrigger::Stop => Some("SessionEnd"),
        HookTrigger::PreCompact => Some("PreCompress"),
        HookTrigger::BeforeModel => Some("BeforeModel"),
        HookTrigger::AfterModel => Some("AfterModel"),
        HookTrigger::BeforeToolSelection => Some("BeforeToolSelection"),
        HookTrigger::UserPromptSubmit
        | HookTrigger::PermissionRequest
        | HookTrigger::PostToolUseFailure
        | HookTrigger::SubagentStart
        | HookTrigger::SubagentStop => None,
    }
}

fn export_gemini_workspace_policy(project_root: &Path, permissions: &Permissions) -> Result<()> {
    let path = project_root
        .join(".gemini")
        .join("policies")
        .join("ship-permissions.toml");

    if !has_gemini_policy_overrides(permissions) {
        fs::remove_file(&path).ok();
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut rules = Vec::new();

    // Highest priority: explicit denies
    for pattern in &permissions.tools.deny {
        rules.push(GeminiPolicyRule {
            tool_name: Some(pattern.clone()),
            decision: "deny".to_string(),
            priority: 900,
            ..Default::default()
        });
    }
    for pattern in &permissions.commands.deny {
        let (prefix, regex) = command_pattern_fields(pattern);
        rules.push(GeminiPolicyRule {
            tool_name: Some("run_shell_command".to_string()),
            command_prefix: prefix,
            command_regex: regex,
            decision: "deny".to_string(),
            priority: 900,
            ..Default::default()
        });
    }

    // Mid priority: explicit confirmation
    for pattern in &permissions.agent.require_confirmation {
        let (prefix, regex) = command_pattern_fields(pattern);
        rules.push(GeminiPolicyRule {
            tool_name: Some("run_shell_command".to_string()),
            command_prefix: prefix,
            command_regex: regex,
            decision: "ask_user".to_string(),
            priority: 800,
            ..Default::default()
        });
    }

    // Lower priority: allows
    for pattern in &permissions.tools.allow {
        rules.push(GeminiPolicyRule {
            tool_name: Some(pattern.clone()),
            decision: "allow".to_string(),
            priority: 700,
            ..Default::default()
        });
    }
    for pattern in &permissions.commands.allow {
        let (prefix, regex) = command_pattern_fields(pattern);
        rules.push(GeminiPolicyRule {
            tool_name: Some("run_shell_command".to_string()),
            command_prefix: prefix,
            command_regex: regex,
            decision: "allow".to_string(),
            priority: 700,
            ..Default::default()
        });
    }

    let doc = GeminiPolicyDoc { rules };
    let body = toml::to_string_pretty(&doc)?;
    let content = format!(
        "# managed by ship\n# source: .ship/agents/permissions.toml\n\n{}",
        body
    );
    crate::fs_util::write_atomic(&path, content)
}

fn codex_writable_roots(project_root: &Path, permissions: &Permissions) -> Vec<toml::Value> {
    let mut seen = HashSet::new();
    let mut roots: Vec<toml::Value> = Vec::new();
    let mut push_root = |path: PathBuf| {
        let normalized = fs::canonicalize(&path).unwrap_or(path);
        let key = normalized.to_string_lossy().to_string();
        if seen.insert(key.clone()) {
            roots.push(toml::Value::String(key));
        }
    };

    push_root(project_root.to_path_buf());

    let home_dir = std::env::var("HOME").ok().map(PathBuf::from);
    for raw in &permissions.filesystem.allow {
        let value = raw.trim();
        if value.is_empty()
            || value.contains('*')
            || value.contains('?')
            || value.contains('[')
            || value.contains('{')
        {
            continue;
        }

        let path = if value == "." || value == "./" {
            project_root.to_path_buf()
        } else if let Some(stripped) = value.strip_prefix("~/") {
            if let Some(home) = &home_dir {
                home.join(stripped)
            } else {
                continue;
            }
        } else {
            let parsed = PathBuf::from(value);
            if parsed.is_absolute() {
                parsed
            } else {
                let rel = value.strip_prefix("./").unwrap_or(value);
                project_root.join(rel)
            }
        };
        push_root(path);
    }

    roots
}

fn apply_codex_permissions(
    root: &mut toml::value::Table,
    project_root: &Path,
    permissions: &Permissions,
) {
    let network_access = matches!(
        permissions.network.policy,
        crate::permissions::NetworkPolicy::AllowList
            | crate::permissions::NetworkPolicy::Unrestricted
    );
    root.insert(
        "sandbox_mode".to_string(),
        toml::Value::String("workspace-write".to_string()),
    );
    let approval = if permissions.agent.require_confirmation.is_empty()
        && permissions.commands.deny.is_empty()
        && permissions.tools.deny.is_empty()
        && permissions.tools.allow.iter().any(|p| p == "*")
    {
        "on-failure"
    } else {
        "on-request"
    };
    root.insert(
        "approval_policy".to_string(),
        toml::Value::String(approval.to_string()),
    );

    let sandbox_entry = root
        .entry("sandbox_workspace_write".to_string())
        .or_insert_with(|| toml::Value::Table(toml::value::Table::new()));
    if let Some(table) = sandbox_entry.as_table_mut() {
        table.insert(
            "network_access".to_string(),
            toml::Value::Boolean(network_access),
        );
        table.insert(
            "writable_roots".to_string(),
            toml::Value::Array(codex_writable_roots(project_root, permissions)),
        );
    }
    // Legacy Ship exports wrote command policy into top-level `allow` and
    // `[rules].prefix_rules`. Current Codex expects Starlark rules files under
    // `.codex/rules/*.rules`, so clear stale legacy fields on each export.
    root.remove("allow");
    if let Some(rules_val) = root.get_mut("rules")
        && let Some(rules_table) = rules_val.as_table_mut()
    {
        rules_table.remove("prefix_rules");
        if rules_table.is_empty() {
            root.remove("rules");
        }
    }
}

fn export_codex_execpolicy(project_root: &Path, permissions: &Permissions) -> Result<()> {
    let rules_dir = project_root.join(".codex").join("rules");
    let rules_path = rules_dir.join("ship.rules");
    let rules = codex_execpolicy_rules_from_permissions(permissions);
    if rules.is_empty() {
        if rules_path.exists() {
            fs::remove_file(&rules_path).ok();
        }
        return Ok(());
    }

    fs::create_dir_all(&rules_dir)?;
    let mut content = String::new();
    content.push_str(
        "# Generated by Ship. Do not edit manually — run `ship git sync` to regenerate.\n",
    );
    content.push_str("# Source: .ship/agents/permissions.toml\n\n");
    for (tokens, decision, source_pattern) in rules {
        content.push_str("prefix_rule(\n");
        content.push_str("    pattern = [");
        content.push_str(
            &tokens
                .iter()
                .map(|token| format!("\"{}\"", escape_starlark_string(token)))
                .collect::<Vec<_>>()
                .join(", "),
        );
        content.push_str("],\n");
        content.push_str(&format!("    decision = \"{}\",\n", decision));
        content.push_str(&format!(
            "    justification = \"Managed by Ship permissions (pattern: {})\",\n",
            escape_starlark_string(&source_pattern)
        ));
        content.push_str(")\n\n");
    }
    crate::fs_util::write_atomic(&rules_path, content)
}

fn teardown_codex_execpolicy(project_root: &Path) -> Result<()> {
    let rules_path = project_root.join(".codex").join("rules").join("ship.rules");
    if rules_path.exists() {
        fs::remove_file(&rules_path)?;
    }
    Ok(())
}

fn codex_execpolicy_rules_from_permissions(
    permissions: &Permissions,
) -> Vec<(Vec<String>, String, String)> {
    let mut out = Vec::new();
    for pattern in &permissions.commands.deny {
        if let Some(tokens) = codex_pattern_tokens(pattern) {
            out.push((tokens, "forbidden".to_string(), pattern.clone()));
        }
    }
    for pattern in &permissions.agent.require_confirmation {
        if let Some(tokens) = codex_pattern_tokens(pattern) {
            out.push((tokens, "prompt".to_string(), pattern.clone()));
        }
    }
    for pattern in &permissions.commands.allow {
        if let Some(tokens) = codex_pattern_tokens(pattern) {
            out.push((tokens, "allow".to_string(), pattern.clone()));
        }
    }
    let mut seen = HashSet::new();
    out.retain(|(tokens, decision, _)| seen.insert((tokens.clone(), decision.clone())));
    out
}

fn codex_pattern_tokens(pattern: &str) -> Option<Vec<String>> {
    let trimmed = pattern.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(regex) = trimmed.strip_prefix("regex:") {
        if !regex.trim().is_empty() {
            return None;
        }
    }
    let base = if let Some(prefix) = command_prefix_from_pattern(trimmed) {
        prefix
    } else {
        if trimmed.contains('*') {
            return None;
        }
        trimmed.to_string()
    };
    let tokens: Vec<String> = base
        .split_whitespace()
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(str::to_string)
        .collect();
    if tokens.is_empty() {
        None
    } else {
        Some(tokens)
    }
}

fn escape_starlark_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn import_permissions_from_claude(project_dir: &Path) -> Result<Option<Permissions>> {
    let Some(project_root) = project_dir.parent() else {
        return Ok(None);
    };
    let project_path = project_root.join(".claude").join("settings.json");
    let global_path = home()?.join(".claude").join("settings.json");
    let path = if project_path.exists() {
        project_path
    } else if global_path.exists() {
        global_path
    } else {
        return Ok(None);
    };

    let root: serde_json::Value = serde_json::from_str(&fs::read_to_string(&path)?)?;
    let Some(perms) = root.get("permissions").and_then(|p| p.as_object()) else {
        return Ok(None);
    };
    let allow = perms
        .get("allow")
        .and_then(|v| v.as_array())
        .map(|values| {
            values
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let deny = perms
        .get("deny")
        .and_then(|v| v.as_array())
        .map(|values| {
            values
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if allow.is_empty() && deny.is_empty() {
        return Ok(None);
    }

    let mut permissions = Permissions::default();
    if !allow.is_empty() {
        permissions.tools.allow = allow;
    }
    permissions.tools.deny = deny;
    Ok(Some(permissions))
}

fn import_permissions_from_gemini(project_dir: &Path) -> Result<Option<Permissions>> {
    let Some(project_root) = project_dir.parent() else {
        return Ok(None);
    };
    let path = project_root
        .join(".gemini")
        .join("policies")
        .join("ship-permissions.toml");
    if !path.exists() {
        return Ok(None);
    }

    let root: toml::Value = toml::from_str(&fs::read_to_string(path)?)?;
    let rules = root
        .get("rule")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if rules.is_empty() {
        return Ok(None);
    }

    let mut permissions = Permissions::default();
    permissions.tools.allow.clear();
    permissions.tools.deny.clear();
    permissions.commands.allow.clear();
    permissions.commands.deny.clear();
    permissions.agent.require_confirmation.clear();

    for value in rules {
        let Some(rule) = value.as_table() else {
            continue;
        };
        let decision = rule
            .get("decision")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if decision.is_empty() {
            continue;
        }

        let tool_names = value_to_string_list(rule.get("toolName"));
        let mcp_names = value_to_string_list(rule.get("mcpName"));
        let command_prefixes = value_to_string_list(rule.get("commandPrefix"));
        let command_regexes = value_to_string_list(rule.get("commandRegex"));

        for command in command_prefixes {
            match decision.as_str() {
                "allow" => permissions.commands.allow.push(format!("{}*", command)),
                "deny" => permissions.commands.deny.push(format!("{}*", command)),
                "ask_user" => permissions
                    .agent
                    .require_confirmation
                    .push(format!("{}*", command)),
                _ => {}
            }
        }
        for regex in command_regexes {
            let pattern = format!("regex:{}", regex);
            match decision.as_str() {
                "allow" => permissions.commands.allow.push(pattern),
                "deny" => permissions.commands.deny.push(pattern),
                "ask_user" => permissions.agent.require_confirmation.push(pattern),
                _ => {}
            }
        }

        let mut composite_tools = Vec::new();
        if tool_names.is_empty() && mcp_names.is_empty() {
            continue;
        }
        if tool_names.is_empty() {
            for mcp_name in &mcp_names {
                composite_tools.push(format!("{}__*", mcp_name));
            }
        } else if mcp_names.is_empty() {
            composite_tools.extend(tool_names.clone());
        } else {
            for mcp_name in &mcp_names {
                for tool_name in &tool_names {
                    composite_tools.push(format!("{}__{}", mcp_name, tool_name));
                }
            }
        }

        for tool in composite_tools {
            if tool == "run_shell_command" {
                continue;
            }
            match decision.as_str() {
                "allow" => permissions.tools.allow.push(tool),
                "deny" => permissions.tools.deny.push(tool),
                _ => {}
            }
        }
    }

    dedupe_strings(&mut permissions.tools.allow);
    dedupe_strings(&mut permissions.tools.deny);
    dedupe_strings(&mut permissions.commands.allow);
    dedupe_strings(&mut permissions.commands.deny);
    dedupe_strings(&mut permissions.agent.require_confirmation);

    if permissions.tools.allow.is_empty() {
        permissions.tools.allow.push("*".to_string());
    }
    Ok(Some(permissions))
}

fn import_permissions_from_codex(project_dir: &Path) -> Result<Option<Permissions>> {
    let Some(project_root) = project_dir.parent() else {
        return Ok(None);
    };
    let path = project_root.join(".codex").join("config.toml");
    if !path.exists() {
        return Ok(None);
    }

    let root: toml::Value = toml::from_str(&fs::read_to_string(path)?)?;
    let mut imported = false;
    let mut permissions = Permissions::default();

    // Legacy imports: pre-execpolicy Codex fields used by older Ship versions.
    if let Some(allow) = root.get("allow").and_then(|v| v.as_array()) {
        permissions.commands.allow = allow
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect();
        imported = true;
    }

    if let Some(network_access) = root
        .get("sandbox_workspace_write")
        .and_then(|v| v.as_table())
        .and_then(|t| t.get("network_access"))
        .and_then(|v| v.as_bool())
    {
        imported = true;
        permissions.network.policy = if network_access {
            crate::permissions::NetworkPolicy::Unrestricted
        } else {
            crate::permissions::NetworkPolicy::None
        };
    }

    let prefix_rules = read_codex_prefix_rules_from_value(&root);
    for (prefix, decision) in prefix_rules {
        imported = true;
        let pattern = codex_prefix_to_pattern(&prefix);
        match decision.as_str() {
            "forbidden" => permissions.commands.deny.push(pattern),
            "prompt" => permissions.agent.require_confirmation.push(pattern),
            "allow" => permissions.commands.allow.push(pattern),
            _ => {}
        }
    }

    let rules_dir = project_root.join(".codex").join("rules");
    let starlark_rules = read_codex_execpolicy_rules(&rules_dir);
    for (tokens, decision) in starlark_rules {
        imported = true;
        let pattern = format!("{} *", tokens.join(" "));
        match decision.as_str() {
            "forbidden" => permissions.commands.deny.push(pattern),
            "prompt" => permissions.agent.require_confirmation.push(pattern),
            "allow" => permissions.commands.allow.push(pattern),
            _ => {}
        }
    }

    if !imported {
        return Ok(None);
    }
    dedupe_strings(&mut permissions.commands.allow);
    dedupe_strings(&mut permissions.commands.deny);
    dedupe_strings(&mut permissions.agent.require_confirmation);
    Ok(Some(permissions))
}

fn command_pattern_fields(pattern: &str) -> (Option<String>, Option<String>) {
    if let Some(prefix) = command_prefix_from_pattern(pattern) {
        return (Some(prefix), None);
    }
    (None, Some(glob_to_regex(pattern)))
}

fn command_prefix_from_pattern(pattern: &str) -> Option<String> {
    let trimmed = pattern.trim();
    if !trimmed.ends_with('*') || trimmed.matches('*').count() != 1 {
        return None;
    }
    let prefix = trimmed.trim_end_matches('*').trim();
    if prefix.is_empty() {
        return None;
    }
    Some(prefix.to_string())
}

fn glob_to_regex(glob: &str) -> String {
    let mut out = String::new();
    for ch in glob.chars() {
        match ch {
            '*' => out.push_str(".*"),
            '\\' | '.' | '+' | '?' | '^' | '$' | '(' | ')' | '[' | ']' | '{' | '}' | '|' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out
}

fn value_to_string_list(value: Option<&toml::Value>) -> Vec<String> {
    match value {
        Some(toml::Value::String(s)) => vec![s.to_string()],
        Some(toml::Value::Array(values)) => values
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect(),
        _ => Vec::new(),
    }
}

fn read_codex_prefix_rules_from_value(root: &toml::Value) -> Vec<(String, String)> {
    root.get("rules")
        .and_then(|v| v.as_table())
        .and_then(|table| table.get("prefix_rules"))
        .and_then(|v| v.as_array())
        .map(|entries| {
            entries
                .iter()
                .filter_map(|entry| {
                    let table = entry.as_table()?;
                    let prefix = table.get("prefix")?.as_str()?.to_string();
                    let decision = table.get("decision")?.as_str()?.to_string();
                    Some((prefix, decision))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn codex_prefix_to_pattern(prefix: &str) -> String {
    let trimmed = prefix.trim();
    if trimmed.is_empty() {
        "*".to_string()
    } else {
        format!("{trimmed} *")
    }
}

fn read_codex_execpolicy_rules(rules_dir: &Path) -> Vec<(Vec<String>, String)> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(rules_dir) else {
        return out;
    };
    let mut files: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| {
            path.extension()
                .and_then(|e| e.to_str())
                .map(|ext| ext == "rules" || ext == "codexpolicy")
                .unwrap_or(false)
        })
        .collect();
    files.sort();
    for path in files {
        let Ok(content) = fs::read_to_string(path) else {
            continue;
        };
        out.extend(parse_codex_execpolicy_rules(&content));
    }
    out
}

fn parse_codex_execpolicy_rules(content: &str) -> Vec<(Vec<String>, String)> {
    let mut out = Vec::new();
    let mut current = Vec::new();
    let mut in_rule = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if !in_rule {
            if trimmed.starts_with("prefix_rule(") {
                in_rule = true;
                current.clear();
                current.push(line.to_string());
                if trimmed.ends_with(')') {
                    if let Some(parsed) = parse_codex_execpolicy_rule_block(&current.join("\n")) {
                        out.push(parsed);
                    }
                    in_rule = false;
                    current.clear();
                }
            }
            continue;
        }

        current.push(line.to_string());
        if trimmed.starts_with(')') {
            if let Some(parsed) = parse_codex_execpolicy_rule_block(&current.join("\n")) {
                out.push(parsed);
            }
            in_rule = false;
            current.clear();
        }
    }
    out
}

fn parse_codex_execpolicy_rule_block(block: &str) -> Option<(Vec<String>, String)> {
    let pattern = extract_list_strings_for_key(block, "pattern")?;
    if pattern.is_empty() {
        return None;
    }
    let decision =
        extract_string_for_key(block, "decision").unwrap_or_else(|| "allow".to_string());
    if !matches!(decision.as_str(), "allow" | "prompt" | "forbidden") {
        return None;
    }
    Some((pattern, decision))
}

fn extract_string_for_key(block: &str, key: &str) -> Option<String> {
    let needle = format!("{key} =");
    for line in block.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with(&needle) {
            continue;
        }
        let first = trimmed.find('"')?;
        let remainder = &trimmed[(first + 1)..];
        let second = remainder.find('"')?;
        return Some(remainder[..second].to_string());
    }
    None
}

fn extract_list_strings_for_key(block: &str, key: &str) -> Option<Vec<String>> {
    let key_pos = block.find(key)?;
    let after_key = &block[key_pos..];
    let eq_pos = after_key.find('=')?;
    let after_eq = &after_key[(eq_pos + 1)..];
    let open_pos = after_eq.find('[')?;
    let list_fragment = &after_eq[open_pos..];

    let mut depth = 0usize;
    let mut end_idx = None;
    for (idx, ch) in list_fragment.char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => {
                if depth == 0 {
                    return None;
                }
                depth -= 1;
                if depth == 0 {
                    end_idx = Some(idx);
                    break;
                }
            }
            _ => {}
        }
    }
    let end = end_idx?;
    let array_src = &list_fragment[..=end];
    let mut values = Vec::new();
    let mut chars = array_src.chars().peekable();
    let mut list_depth = 0usize;
    while let Some(ch) = chars.next() {
        match ch {
            '[' => list_depth += 1,
            ']' => {
                if list_depth > 0 {
                    list_depth -= 1;
                }
            }
            '"' if list_depth == 1 => {
                let mut value = String::new();
                while let Some(next) = chars.next() {
                    if next == '\\' {
                        if let Some(escaped) = chars.next() {
                            value.push(escaped);
                        }
                        continue;
                    }
                    if next == '"' {
                        break;
                    }
                    value.push(next);
                }
                values.push(value);
            }
            _ => {}
        }
    }
    Some(values)
}

fn dedupe_strings(values: &mut Vec<String>) {
    let mut seen = HashSet::new();
    values.retain(|item| seen.insert(item.clone()));
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn home() -> Result<PathBuf> {
    if let Some(home) = std::env::var_os("HOME") {
        let path = PathBuf::from(home);
        if !path.as_os_str().is_empty() {
            return Ok(path);
        }
    }
    home::home_dir().ok_or_else(|| anyhow!("Cannot determine home directory"))
}

// ─── Tests ────────────────────────────────────────────────────────────────────
