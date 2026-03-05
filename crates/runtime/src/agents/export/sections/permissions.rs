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

fn export_claude_settings(hooks: &[HookConfig], permissions: &Permissions) -> Result<()> {
    let path = home()?.join(".claude").join("settings.json");
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
        .ok_or_else(|| anyhow!("~/.claude/settings.json is not an object"))?;

    if has_claude_permission_overrides(permissions) {
        let perms = obj.entry("permissions").or_insert(serde_json::json!({}));
        let p = perms
            .as_object_mut()
            .ok_or_else(|| anyhow!("permissions not an object"))?;
        p.insert("allow".into(), serde_json::json!(permissions.tools.allow));
        p.insert("deny".into(), serde_json::json!(permissions.tools.deny));
    }

    if !hooks.is_empty() {
        let hooks_val = obj.entry("hooks").or_insert(serde_json::json!({}));
        let hooks_map = hooks_val
            .as_object_mut()
            .ok_or_else(|| anyhow!("hooks not an object"))?;
        let mut by_trigger: HashMap<&str, Vec<serde_json::Value>> = HashMap::new();
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

fn apply_codex_permissions(root: &mut toml::value::Table, permissions: &Permissions) {
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

    if !permissions.commands.allow.is_empty() {
        root.insert(
            "allow".to_string(),
            toml::Value::Array(
                permissions
                    .commands
                    .allow
                    .iter()
                    .cloned()
                    .map(toml::Value::String)
                    .collect(),
            ),
        );
    }

    let sandbox_entry = root
        .entry("sandbox_workspace_write".to_string())
        .or_insert_with(|| toml::Value::Table(toml::value::Table::new()));
    if let Some(table) = sandbox_entry.as_table_mut() {
        table.insert(
            "network_access".to_string(),
            toml::Value::Boolean(network_access),
        );
    }

    let mut prefix_rules = read_codex_prefix_rules(root);
    for pattern in &permissions.commands.deny {
        if let Some(prefix) = command_prefix_from_pattern(pattern) {
            prefix_rules.push((prefix, "forbidden".to_string()));
        }
    }
    for pattern in &permissions.agent.require_confirmation {
        if let Some(prefix) = command_prefix_from_pattern(pattern) {
            prefix_rules.push((prefix, "prompt".to_string()));
        }
    }
    dedupe_pairs(&mut prefix_rules);
    if !prefix_rules.is_empty() {
        let rules_entry = root
            .entry("rules".to_string())
            .or_insert_with(|| toml::Value::Table(toml::value::Table::new()));
        if let Some(rules_table) = rules_entry.as_table_mut() {
            let array = prefix_rules
                .into_iter()
                .map(|(prefix, decision)| {
                    let mut table = toml::value::Table::new();
                    table.insert("prefix".to_string(), toml::Value::String(prefix));
                    table.insert("decision".to_string(), toml::Value::String(decision));
                    toml::Value::Table(table)
                })
                .collect();
            rules_table.insert("prefix_rules".to_string(), toml::Value::Array(array));
        }
    }
}

fn import_permissions_from_claude() -> Result<Option<Permissions>> {
    let path = home()?.join(".claude").join("settings.json");
    if !path.exists() {
        return Ok(None);
    }

    let root: serde_json::Value = serde_json::from_str(&fs::read_to_string(path)?)?;
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
        let pattern = format!("{}*", prefix);
        match decision.as_str() {
            "forbidden" => permissions.commands.deny.push(pattern),
            "prompt" => permissions.agent.require_confirmation.push(pattern),
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

fn read_codex_prefix_rules(root: &toml::value::Table) -> Vec<(String, String)> {
    read_codex_prefix_rules_from_value(&toml::Value::Table(root.clone()))
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

fn dedupe_strings(values: &mut Vec<String>) {
    let mut seen = HashSet::new();
    values.retain(|item| seen.insert(item.clone()));
}

fn dedupe_pairs(values: &mut Vec<(String, String)>) {
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
