pub struct SyncPayload {
    pub servers: Vec<McpServerConfig>,
    pub instruction_skill_id: Option<String>,
    pub instructions: Option<String>,
    pub hooks: Vec<HookConfig>,
    pub permissions: Permissions,
    pub active_mode_id: Option<String>,
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Write a context file (CLAUDE.md, GEMINI.md, Codex instructions, etc.) for the given provider.
///
/// Called by the git module after building provider-agnostic Markdown content.
/// Each provider has a specific destination:
/// - Claude  → `CLAUDE.md` at project root
/// - Gemini  → `GEMINI.md` at project root
/// - Codex / Roo / Amp / Goose → `AGENTS.md` at project root
/// - Unknown provider / `PromptOutput::None` → no-op
pub fn write_context(project_root: &Path, provider_id: &str, content: &str) -> Result<()> {
    let desc = match get_provider(provider_id) {
        Some(d) => d,
        None => return Ok(()),
    };
    match desc.prompt_output {
        PromptOutput::ClaudeMd => {
            let path = project_root.join("CLAUDE.md");
            crate::fs_util::write_atomic(&path, content)?;
        }
        PromptOutput::GeminiMd => {
            let path = project_root.join("GEMINI.md");
            crate::fs_util::write_atomic(&path, content)?;
        }
        PromptOutput::AgentsMd => {
            let path = project_root.join("AGENTS.md");
            crate::fs_util::write_atomic(&path, content)?;
        }
        PromptOutput::None => {}
    }
    Ok(())
}

/// Export the active mode (or global config) to the specified provider.
pub fn export_to(project_dir: PathBuf, target: &str) -> Result<()> {
    export_to_inner(project_dir, target, None, None)
}

/// Like `export_to` but restricts project MCP servers to those whose IDs appear in
/// `server_filter`. Pass `None` to write all project servers (same as `export_to`).
pub fn export_to_filtered(
    project_dir: PathBuf,
    target: &str,
    server_filter: Option<&[String]>,
) -> Result<()> {
    export_to_inner(project_dir, target, server_filter, None)
}

/// Like `export_to` but applies a mode override when building payload.
pub fn export_to_with_mode_override(
    project_dir: PathBuf,
    target: &str,
    active_mode_override: Option<&str>,
) -> Result<()> {
    export_to_inner(project_dir, target, None, active_mode_override)
}

/// Like `export_to_filtered` but applies a mode override when building payload.
pub fn export_to_filtered_with_mode_override(
    project_dir: PathBuf,
    target: &str,
    server_filter: Option<&[String]>,
    active_mode_override: Option<&str>,
) -> Result<()> {
    export_to_inner(project_dir, target, server_filter, active_mode_override)
}

fn export_to_inner(
    project_dir: PathBuf,
    target: &str,
    server_filter: Option<&[String]>,
    active_mode_override: Option<&str>,
) -> Result<()> {
    let desc = require_provider(target)?;
    let mut payload = build_payload_with_mode_override(&project_dir, active_mode_override)?;
    if let Some(ids) = server_filter {
        payload.servers.retain(|s| ids.contains(&s.id));
    }
    let project_root = project_dir
        .parent()
        .ok_or_else(|| anyhow!("Cannot determine project root from {:?}", project_dir))?;
    let mut state = load_managed_state(&project_dir);

    match desc.config_format {
        ConfigFormat::Json => export_json(desc, &project_dir, project_root, &payload, &mut state)?,
        ConfigFormat::Toml => export_toml(desc, &project_dir, project_root, &payload, &mut state)?,
    }

    // Skills output (provider-specific)
    match desc.skills_output {
        SkillsOutput::ClaudeSkills => export_skills_to_claude(&project_dir, project_root)?,
        SkillsOutput::AgentSkills => {
            export_skills_to_dir(&project_dir, &project_root.join(".gemini").join("skills"))?
        }
        SkillsOutput::CodexSkills => {
            export_skills_to_dir(&project_dir, &project_root.join(".agents").join("skills"))?
        }
        SkillsOutput::None => {}
    }

    // Provider-native hooks + permissions.
    match target {
        "claude" => {
            write_hook_runtime_artifacts(project_root, &payload)?;
            let provider_hooks = hooks_for_provider("claude", &payload.hooks);
            if !provider_hooks.is_empty() || has_claude_permission_overrides(&payload.permissions)
            {
                export_claude_settings(&provider_hooks, &payload.permissions)?;
            }
        }
        "gemini" => {
            write_hook_runtime_artifacts(project_root, &payload)?;
            let provider_hooks = hooks_for_provider("gemini", &payload.hooks);
            export_gemini_settings(project_root, &provider_hooks)?;
            export_gemini_workspace_policy(project_root, &payload.permissions)?;
        }
        _ => {}
    }

    save_managed_state(&project_dir, &state)?;
    Ok(())
}

/// Remove all Ship-generated config for the given provider.
pub fn teardown(project_dir: PathBuf, target: &str) -> Result<()> {
    let desc = require_provider(target)?;
    let project_root = project_dir
        .parent()
        .ok_or_else(|| anyhow!("Cannot determine project root from {:?}", project_dir))?;
    let mut state = load_managed_state(&project_dir);
    let tool_state = state
        .providers
        .entry(target.to_string())
        .or_default()
        .clone();

    match desc.config_format {
        ConfigFormat::Json => {
            let config_path = project_root.join(desc.project_config);
            teardown_json(
                &config_path,
                desc.mcp_key,
                &desc.managed_marker,
                &tool_state,
            )?;
        }
        ConfigFormat::Toml => {
            let config_path = project_root.join(desc.project_config);
            teardown_toml(&config_path, desc.mcp_key, &tool_state)?;
        }
    }

    // Remove prompt file if applicable
    match desc.prompt_output {
        PromptOutput::ClaudeMd => {
            let f = project_root.join("CLAUDE.md");
            if f.exists() {
                fs::remove_file(&f).with_context(|| format!("Failed to remove {}", f.display()))?;
            }
        }
        PromptOutput::GeminiMd => {
            let f = project_root.join("GEMINI.md");
            if f.exists() {
                fs::remove_file(&f).ok();
            }
        }
        PromptOutput::AgentsMd => {
            let f = project_root.join("AGENTS.md");
            if f.exists() {
                fs::remove_file(&f).ok();
            }
        }
        PromptOutput::None => {}
    }

    // Remove skill files written by Ship
    match desc.skills_output {
        SkillsOutput::ClaudeSkills => {
            remove_ship_managed_skill_dirs(&project_root.join(".claude").join("skills"));
        }
        SkillsOutput::AgentSkills => {
            remove_ship_managed_skill_dirs(&project_root.join(".gemini").join("skills"));
        }
        SkillsOutput::CodexSkills => {
            remove_ship_managed_skill_dirs(&project_root.join(".agents").join("skills"));
        }
        SkillsOutput::None => {}
    }

    // Clear managed state for this provider
    if let Some(ts) = state.providers.get_mut(target) {
        ts.managed_servers.clear();
        ts.last_mode = None;
    }
    save_managed_state(&project_dir, &state)?;
    Ok(())
}

/// Sync all target agents configured for the active mode.
pub fn sync_active_mode(project_dir: &Path) -> Result<Vec<String>> {
    sync_active_mode_with_override(project_dir, None)
}

/// Sync all target agents configured for the active mode (or the override mode when provided).
pub fn sync_active_mode_with_override(
    project_dir: &Path,
    active_mode_override: Option<&str>,
) -> Result<Vec<String>> {
    let config = get_effective_config(Some(project_dir.to_path_buf()))?;
    let (resolved_mode, _) = resolve_active_mode(&config, active_mode_override);
    let mode_targets = resolved_mode
        .map(|m| m.target_agents.clone())
        .unwrap_or_default();
    let targets: Vec<String> = if !mode_targets.is_empty() {
        mode_targets
    } else if !config.providers.is_empty() {
        config.providers.clone()
    } else {
        vec!["claude".to_string()]
    };

    let mut seen = std::collections::HashSet::new();
    let mut synced = Vec::new();
    for target in targets {
        let normalized = target.trim().to_ascii_lowercase();
        if normalized.is_empty() || !seen.insert(normalized.clone()) {
            continue;
        }
        if get_provider(&normalized).is_none() {
            eprintln!("[ship] warning: skipping unknown target agent '{}'", target);
            continue;
        }
        export_to_with_mode_override(
            project_dir.to_path_buf(),
            &normalized,
            active_mode_override,
        )?;
        synced.push(normalized);
    }
    Ok(synced)
}

/// Non-destructive import of MCP servers from a provider's existing config.
/// Returns count of newly-added servers.
pub fn import_from_claude(project_dir: PathBuf) -> Result<usize> {
    import_from_provider("claude", project_dir)
}

pub fn import_from_provider(provider_id: &str, project_dir: PathBuf) -> Result<usize> {
    let desc = require_provider(provider_id)?;
    let (managed, _) =
        crate::state_db::get_managed_state_db(&project_dir, provider_id).unwrap_or_default();

    let mut config = get_config(Some(project_dir.clone()))?;
    let mut added = 0usize;
    let import_paths = provider_import_paths(desc, &project_dir)?;

    for path in import_paths {
        if !path.exists() {
            continue;
        }
        let imported_scope = import_scope_for_path(desc, &project_dir, &path);
        let imported_servers = match desc.config_format {
            ConfigFormat::Json => import_mcp_servers_from_json(desc, &path)?,
            ConfigFormat::Toml => import_mcp_servers_from_toml(desc, &path)?,
        };

        for server in imported_servers {
            let Some(mut server) = normalize_imported_server(server) else {
                continue;
            };
            server.scope = imported_scope.clone();
            if managed.contains(&server.id) {
                continue;
            }
            if config
                .mcp_servers
                .iter()
                .any(|existing| existing.id == server.id)
            {
                continue;
            }
            config.mcp_servers.push(server);
            added += 1;
        }
    }

    if added > 0 {
        crate::config::save_config(&config, Some(project_dir))?;
    }
    Ok(added)
}

fn import_scope_for_path(desc: &ProviderDescriptor, project_dir: &Path, path: &Path) -> String {
    let project_scope_path = project_dir
        .parent()
        .map(|root| root.join(desc.project_config));
    if project_scope_path.as_ref().is_some_and(|p| p == path) {
        "project".to_string()
    } else {
        "global".to_string()
    }
}

fn normalize_imported_server(mut server: McpServerConfig) -> Option<McpServerConfig> {
    server.id = server.id.trim().to_string();
    if server.id.is_empty() {
        return None;
    }

    // Ship server is runtime-managed and always injected on export.
    if server.id.eq_ignore_ascii_case("ship") {
        return None;
    }

    if server.name.trim().is_empty() {
        server.name = server.id.clone();
    }

    match server.server_type {
        McpServerType::Stdio => {
            server.command = server.command.trim().to_string();
            if server.command.is_empty() {
                return None;
            }
            server.url = None;
        }
        McpServerType::Sse | McpServerType::Http => {
            let url = server
                .url
                .as_deref()
                .map(str::trim)
                .filter(|u| !u.is_empty())?
                .to_string();
            server.url = Some(url);
            server.command.clear();
            server.args.clear();
        }
    }

    Some(server)
}

/// Import provider-native permission settings into canonical
/// `.ship/agents/permissions.toml`.
///
/// Returns `true` when permissions were imported and saved, `false` when no
/// importable permissions were found for the provider.
pub fn import_permissions_from_provider(provider_id: &str, project_dir: PathBuf) -> Result<bool> {
    let imported = match provider_id {
        "claude" => import_permissions_from_claude()?,
        "gemini" => import_permissions_from_gemini(&project_dir)?,
        "codex" => import_permissions_from_codex(&project_dir)?,
        _ => return Err(anyhow!("Unsupported provider '{}'", provider_id)),
    };

    if let Some(permissions) = imported {
        crate::permissions::save_permissions(project_dir, &permissions)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

fn provider_import_paths(desc: &ProviderDescriptor, project_dir: &Path) -> Result<Vec<PathBuf>> {
    let project_path_opt = project_dir
        .parent()
        .map(|project_root| project_root.join(desc.project_config));
    let global_path = home()?.join(desc.global_config);
    if let Some(project_path) = project_path_opt.as_ref()
        && project_path.exists()
    {
        // Prefer project config when present. Ship users typically commit to project-scoped
        // config ownership, so mixing in global provider state here is surprising.
        return Ok(vec![project_path.clone()]);
    }

    if global_path.exists() {
        return Ok(vec![global_path.clone()]);
    }

    // No files found yet; return candidate paths (project first, then global) for
    // callers that want to surface diagnostics or future file creation guidance.
    let mut candidates = Vec::new();
    if let Some(project_path) = project_path_opt {
        candidates.push(project_path);
    }
    if !candidates.iter().any(|path| path == &global_path) {
        candidates.push(global_path);
    }
    Ok(candidates)
}

fn import_mcp_servers_from_json(
    desc: &ProviderDescriptor,
    path: &Path,
) -> Result<Vec<McpServerConfig>> {
    let root: serde_json::Value = serde_json::from_str(&fs::read_to_string(path)?)?;
    let Some(mcp_obj) = root.get(desc.mcp_key).and_then(|v| v.as_object()) else {
        return Ok(Vec::new());
    };

    let mut servers = Vec::new();
    for (id, entry) in mcp_obj {
        let command = entry
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let url = entry
            .get(desc.http_url_field)
            .or_else(|| entry.get("url"))
            .and_then(|v| v.as_str())
            .map(str::to_string);

        let server_type = match entry.get("type").and_then(|v| v.as_str()) {
            Some("sse") => McpServerType::Sse,
            Some("http") => McpServerType::Http,
            _ => {
                if command.is_empty() && url.is_some() {
                    McpServerType::Http
                } else {
                    McpServerType::Stdio
                }
            }
        };

        servers.push(McpServerConfig {
            id: id.clone(),
            name: id.clone(),
            command,
            args: entry
                .get("args")
                .and_then(|v| v.as_array())
                .map(|args| {
                    args.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            env: entry
                .get("env")
                .and_then(|v| v.as_object())
                .map(|env| {
                    env.iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect::<HashMap<_, _>>()
                })
                .unwrap_or_default(),
            scope: "global".to_string(),
            server_type,
            url,
            disabled: entry
                .get("disabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            timeout_secs: entry
                .get("startup_timeout_sec")
                .or_else(|| entry.get("timeout_secs"))
                .and_then(|v| v.as_u64())
                .and_then(|v| u32::try_from(v).ok()),
        });
    }
    Ok(servers)
}

fn import_mcp_servers_from_toml(
    desc: &ProviderDescriptor,
    path: &Path,
) -> Result<Vec<McpServerConfig>> {
    let root: toml::Value = toml::from_str(&fs::read_to_string(path)?)?;
    let Some(mcp_table) = root.get(desc.mcp_key).and_then(|v| v.as_table()) else {
        return Ok(Vec::new());
    };

    let mut servers = Vec::new();
    for (id, entry) in mcp_table {
        let Some(table) = entry.as_table() else {
            continue;
        };

        let command = table
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let url = table
            .get(desc.http_url_field)
            .or_else(|| table.get("url"))
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let server_type = match table.get("type").and_then(|v| v.as_str()) {
            Some("sse") => McpServerType::Sse,
            Some("http") => McpServerType::Http,
            _ => {
                if command.is_empty() && url.is_some() {
                    McpServerType::Http
                } else {
                    McpServerType::Stdio
                }
            }
        };

        servers.push(McpServerConfig {
            id: id.clone(),
            name: id.clone(),
            command,
            args: table
                .get("args")
                .and_then(|v| v.as_array())
                .map(|args| {
                    args.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            env: table
                .get("env")
                .and_then(|v| v.as_table())
                .map(|env| {
                    env.iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect::<HashMap<_, _>>()
                })
                .unwrap_or_default(),
            scope: "global".to_string(),
            server_type,
            url,
            disabled: table
                .get("disabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            timeout_secs: table
                .get("startup_timeout_sec")
                .and_then(|v| v.as_integer())
                .and_then(|v| u32::try_from(v).ok()),
        });
    }
    Ok(servers)
}

// ─── Payload builder ──────────────────────────────────────────────────────────

fn resolve_active_mode<'a>(
    config: &'a ProjectConfig,
    active_mode_override: Option<&str>,
) -> (Option<&'a ModeConfig>, Option<String>) {
    let override_mode = active_mode_override
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .and_then(|mode_id| config.modes.iter().find(|mode| mode.id == mode_id))
        .map(|mode| mode.id.clone());

    let selected_mode_id = override_mode.or_else(|| config.active_mode.clone());
    let selected_mode = selected_mode_id
        .as_ref()
        .and_then(|mode_id| config.modes.iter().find(|mode| mode.id == *mode_id));

    (selected_mode, selected_mode_id)
}

#[cfg(test)]
fn build_payload(project_dir: &Path) -> Result<SyncPayload> {
    build_payload_with_mode_override(project_dir, None)
}

fn build_payload_with_mode_override(
    project_dir: &Path,
    active_mode_override: Option<&str>,
) -> Result<SyncPayload> {
    let config = get_effective_config(Some(project_dir.to_path_buf()))?;
    let mut effective_permissions = get_permissions(project_dir.to_path_buf())?;
    let (mode, mode_id) = resolve_active_mode(&config, active_mode_override);

    if let Some(mode) = mode {
        if !mode.permissions.allow.is_empty() {
            effective_permissions.tools.allow = mode.permissions.allow.clone();
        }
        if !mode.permissions.deny.is_empty() {
            effective_permissions.tools.deny = mode.permissions.deny.clone();
        }
        let servers = if mode.mcp_servers.is_empty() {
            config.mcp_servers.clone()
        } else {
            config
                .mcp_servers
                .iter()
                .filter(|s| mode.mcp_servers.contains(&s.id))
                .cloned()
                .collect()
        };
        let instruction_skill = mode
            .prompt_id
            .as_ref()
            .and_then(|id| get_effective_skill(project_dir, id).ok());
        let mut hooks = config.hooks.clone();
        hooks.extend(mode.hooks.clone());
        return Ok(SyncPayload {
            servers,
            instruction_skill_id: instruction_skill.as_ref().map(|skill| skill.id.clone()),
            instructions: instruction_skill.map(|skill| skill.content),
            hooks,
            permissions: effective_permissions,
            active_mode_id: mode_id,
        });
    }

    Ok(SyncPayload {
        servers: config.mcp_servers,
        instruction_skill_id: None,
        instructions: None,
        hooks: config.hooks,
        permissions: effective_permissions,
        active_mode_id: mode_id,
    })
}

// ─── Generic export ───────────────────────────────────────────────────────────

fn export_json(
    desc: &ProviderDescriptor,
    _project_dir: &Path,
    project_root: &Path,
    payload: &SyncPayload,
    state: &mut ManagedState,
) -> Result<()> {
    let config_path = project_root.join(desc.project_config);
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let existing: serde_json::Value = if config_path.exists() {
        serde_json::from_str(&fs::read_to_string(&config_path)?).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    let tool_state = state.providers.entry(desc.id.to_string()).or_default();
    let mut mcp_servers = serde_json::Map::new();

    // Preserve user-defined servers (not Ship-managed)
    if let Some(existing_mcp) = existing.get(desc.mcp_key).and_then(|v| v.as_object()) {
        for (id, entry) in existing_mcp {
            let is_managed = match desc.managed_marker {
                ManagedMarker::Inline => entry
                    .get("_ship")
                    .and_then(|v| v.get("managed"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                ManagedMarker::StateFileOnly => false,
            } || tool_state.managed_servers.contains(id);
            if !is_managed {
                mcp_servers.insert(id.clone(), entry.clone());
            }
        }
    }

    // Always inject Ship's own server
    let (ship_id, mut ship_entry) = ship_server_entry();
    if !desc.emit_type_field {
        ship_entry.as_object_mut().map(|o| o.remove("type"));
    }
    mcp_servers.insert(ship_id.to_string(), ship_entry);

    let mut written_ids = vec![ship_id.to_string()];
    for s in &payload.servers {
        if s.disabled {
            continue;
        }
        let mut entry = json_mcp_entry(desc, s);
        if matches!(desc.managed_marker, ManagedMarker::Inline) {
            entry["_ship"] = serde_json::json!({ "managed": true });
        }
        mcp_servers.insert(s.id.clone(), entry);
        written_ids.push(s.id.clone());
    }

    let mut root = existing.clone();
    if !root.is_object() {
        root = serde_json::json!({});
    }
    root["_ship"] = serde_json::json!({
        "managed": true,
        "note": "Generated by Ship. Do not edit manually — run `ship git sync` to regenerate."
    });
    root[desc.mcp_key] = serde_json::Value::Object(mcp_servers);
    crate::fs_util::write_atomic(&config_path, serde_json::to_string_pretty(&root)?)?;

    // System instructions output (from mode `prompt_id`, which now points to a skill ID).
    if let Some(instructions) = &payload.instructions {
        let instruction_id = payload
            .instruction_skill_id
            .as_deref()
            .unwrap_or("unknown-skill");
        match desc.prompt_output {
            PromptOutput::GeminiMd => {
                let md = project_root.join("GEMINI.md");
                let content = format!(
                    "<!-- managed by ship — instructions skill: {} -->\n\n{}\n",
                    instruction_id, instructions
                );
                crate::fs_util::write_atomic(&md, content)?;
            }
            PromptOutput::ClaudeMd | PromptOutput::AgentsMd | PromptOutput::None => {}
        }
    }

    tool_state.managed_servers = written_ids;
    tool_state.last_mode = payload.active_mode_id.clone();
    Ok(())
}

fn export_toml(
    desc: &ProviderDescriptor,
    _project_dir: &Path,
    project_root: &Path,
    payload: &SyncPayload,
    state: &mut ManagedState,
) -> Result<()> {
    let config_path = project_root.join(desc.project_config);
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let raw_existing = if config_path.exists() {
        fs::read_to_string(&config_path)?
    } else {
        String::new()
    };
    let mut doc: toml::Value = if raw_existing.is_empty() {
        toml::Value::Table(Default::default())
    } else {
        toml::from_str(&raw_existing).map_err(|e| {
            anyhow!(
                "Cannot parse {}: {}. Note: Codex uses 'mcp_servers' (underscore).",
                config_path.display(),
                e
            )
        })?
    };

    let root = match &mut doc {
        toml::Value::Table(t) => t,
        _ => return Err(anyhow!("Config root is not a TOML table")),
    };

    let tool_state = state.providers.entry(desc.id.to_string()).or_default();
    let existing_mcp: toml::value::Table = root
        .get(desc.mcp_key)
        .and_then(|v| v.as_table())
        .cloned()
        .unwrap_or_default();

    let mut new_mcp = toml::value::Table::new();
    // Preserve user servers (not Ship-managed)
    for (id, entry) in &existing_mcp {
        if !tool_state.managed_servers.contains(id) {
            new_mcp.insert(id.clone(), entry.clone());
        }
    }

    // Ship self-entry
    let mut ship_entry = toml::value::Table::new();
    ship_entry.insert("command".into(), toml::Value::String("ship".into()));
    ship_entry.insert(
        "args".into(),
        toml::Value::Array(vec![
            toml::Value::String("mcp".into()),
            toml::Value::String("serve".into()),
        ]),
    );
    new_mcp.insert("ship".into(), toml::Value::Table(ship_entry));
    let mut written_ids = vec!["ship".to_string()];

    for s in &payload.servers {
        if s.disabled {
            continue;
        }
        new_mcp.insert(s.id.clone(), toml_mcp_entry(desc, s));
        written_ids.push(s.id.clone());
    }

    root.insert(desc.mcp_key.to_string(), toml::Value::Table(new_mcp));
    if desc.id == "codex" {
        apply_codex_permissions(root, &payload.permissions);
    }

    let header = "# Generated by Ship. Do not edit manually — run `ship git sync` to regenerate.\n\n";
    let content = format!("{}{}", header, toml::to_string_pretty(&doc)?);
    crate::fs_util::write_atomic(&config_path, content)?;

    // System instructions output (from mode `prompt_id`, which now points to a skill ID).
    if let Some(instructions) = &payload.instructions {
        let instruction_id = payload
            .instruction_skill_id
            .as_deref()
            .unwrap_or("unknown-skill");
        match desc.prompt_output {
            PromptOutput::AgentsMd => {
                let md = project_root.join("AGENTS.md");
                let content = format!(
                    "<!-- managed by ship — instructions skill: {} -->\n\n{}\n",
                    instruction_id, instructions
                );
                crate::fs_util::write_atomic(&md, content)?;
            }
            PromptOutput::ClaudeMd | PromptOutput::GeminiMd | PromptOutput::None => {}
        }
    }

    tool_state.managed_servers = written_ids;
    tool_state.last_mode = payload.active_mode_id.clone();
    Ok(())
}

// ─── Generic teardown ─────────────────────────────────────────────────────────

fn teardown_json(
    config_path: &Path,
    mcp_key: &str,
    managed_marker: &ManagedMarker,
    tool_state: &ToolState,
) -> Result<()> {
    if !config_path.exists() {
        return Ok(());
    }

    let existing: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(config_path)?).unwrap_or(serde_json::json!({}));

    let mut kept = serde_json::Map::new();
    if let Some(servers) = existing.get(mcp_key).and_then(|v| v.as_object()) {
        for (id, entry) in servers {
            let is_managed = match managed_marker {
                ManagedMarker::Inline => entry
                    .get("_ship")
                    .and_then(|v| v.get("managed"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                ManagedMarker::StateFileOnly => false,
            } || tool_state.managed_servers.contains(id);
            if !is_managed {
                kept.insert(id.clone(), entry.clone());
            }
        }
    }

    if kept.is_empty() {
        fs::remove_file(config_path).ok();
    } else {
        let mut root = existing.clone();
        if !root.is_object() {
            root = serde_json::json!({});
        }
        root[mcp_key] = serde_json::Value::Object(kept);
        crate::fs_util::write_atomic(config_path, serde_json::to_string_pretty(&root)?)?;
    }
    Ok(())
}

fn teardown_toml(config_path: &Path, mcp_key: &str, tool_state: &ToolState) -> Result<()> {
    if !config_path.exists() {
        return Ok(());
    }

    let raw = fs::read_to_string(config_path)?;
    let mut doc: toml::Value =
        toml::from_str(&raw).unwrap_or(toml::Value::Table(Default::default()));

    if let toml::Value::Table(root) = &mut doc {
        let existing: toml::value::Table = root
            .get(mcp_key)
            .and_then(|v| v.as_table())
            .cloned()
            .unwrap_or_default();
        let mut kept = toml::value::Table::new();
        for (id, entry) in &existing {
            if !tool_state.managed_servers.contains(id) {
                kept.insert(id.clone(), entry.clone());
            }
        }
        root.insert(mcp_key.to_string(), toml::Value::Table(kept));
    }

    crate::fs_util::write_atomic(config_path, toml::to_string_pretty(&doc)?)?;
    Ok(())
}

// ─── Entry builders ───────────────────────────────────────────────────────────

fn ship_server_entry() -> (&'static str, serde_json::Value) {
    let entry = serde_json::json!({
        "command": "ship",
        "args": ["mcp"],
        "type": "stdio",
        "_ship": { "managed": true }
    });
    ("ship", entry)
}

fn json_mcp_entry(desc: &ProviderDescriptor, s: &McpServerConfig) -> serde_json::Value {
    match s.server_type {
        McpServerType::Stdio => {
            let mut entry = serde_json::json!({ "command": s.command });
            if desc.emit_type_field {
                entry["type"] = serde_json::json!("stdio");
            }
            if !s.args.is_empty() {
                entry["args"] = serde_json::json!(s.args);
            }
            if !s.env.is_empty() {
                entry["env"] = serde_json::json!(s.env);
            }
            entry
        }
        McpServerType::Http | McpServerType::Sse => {
            let mut entry = serde_json::json!({ desc.http_url_field: s.url });
            if desc.emit_type_field {
                let type_str = if matches!(s.server_type, McpServerType::Sse) {
                    "sse"
                } else {
                    "http"
                };
                entry["type"] = serde_json::json!(type_str);
            }
            if let Some(t) = s.timeout_secs {
                // Gemini timeout is in ms
                let key = "timeout";
                entry[key] = serde_json::json!(if desc.http_url_field == "httpUrl" {
                    t * 1000
                } else {
                    t
                });
            }
            entry
        }
    }
}

fn toml_mcp_entry(desc: &ProviderDescriptor, s: &McpServerConfig) -> toml::Value {
    let mut entry = toml::value::Table::new();
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
                let env: toml::value::Table = s
                    .env
                    .iter()
                    .map(|(k, v)| (k.clone(), toml::Value::String(v.clone())))
                    .collect();
                entry.insert("env".into(), toml::Value::Table(env));
            }
        }
        McpServerType::Http | McpServerType::Sse => {
            if let Some(url) = &s.url {
                entry.insert(desc.http_url_field.into(), toml::Value::String(url.clone()));
            }
            // Bearer token: if env has a *_TOKEN or *_KEY, surface it
            for k in s.env.keys() {
                if k.ends_with("_TOKEN") || k.ends_with("_KEY") {
                    entry.insert(
                        "bearer_token_env_var".into(),
                        toml::Value::String(k.clone()),
                    );
                    break;
                }
            }
        }
    }
    if let Some(t) = s.timeout_secs {
        entry.insert("startup_timeout_sec".into(), toml::Value::Integer(t as i64));
    }
    toml::Value::Table(entry)
}

// ─── Skills ───────────────────────────────────────────────────────────────────

fn export_skills_to_claude(project_dir: &Path, project_root: &Path) -> Result<()> {
    export_skills_to_dir(project_dir, &project_root.join(".claude").join("skills"))
}

fn resolve_skills_for_export(project_dir: &Path) -> Result<Vec<crate::skill::Skill>> {
    let config = get_effective_config(Some(project_dir.to_path_buf()))?;
    let mut skills = list_effective_skills(project_dir)?;

    if let Some(active_mode_id) = config.active_mode.as_deref()
        && let Some(mode) = config.modes.iter().find(|m| m.id == active_mode_id)
        && !mode.skills.is_empty()
    {
        skills.retain(|skill| mode.skills.contains(&skill.id));
    }

    Ok(skills)
}

/// Write skills using the agentskills.io layout: `<skills_dir>/<skill-id>/SKILL.md`
fn export_skills_to_dir(project_dir: &Path, skills_dir: &Path) -> Result<()> {
    let skills = resolve_skills_for_export(project_dir)?;
    let retain_ids: HashSet<String> = skills.iter().map(|skill| skill.id.clone()).collect();
    prune_stale_managed_skill_dirs(skills_dir, &retain_ids);
    if skills.is_empty() {
        return Ok(());
    }

    fs::create_dir_all(skills_dir)?;
    for skill in &skills {
        let skill_dir = skills_dir.join(&skill.id);
        fs::create_dir_all(&skill_dir)?;
        let path = skill_dir.join("SKILL.md");
        let content = format!(
            "<!-- managed by ship — skill: {} -->\n\n{}\n",
            skill.id, skill.content
        );
        crate::fs_util::write_atomic(&path, content)?;
    }
    Ok(())
}

fn prune_stale_managed_skill_dirs(skills_dir: &Path, retain_ids: &HashSet<String>) {
    if !skills_dir.exists() {
        return;
    }

    if let Ok(entries) = fs::read_dir(skills_dir) {
        for entry in entries.flatten() {
            let skill_dir = entry.path();
            if !skill_dir.is_dir() {
                continue;
            }

            let skill_id = entry.file_name().to_string_lossy().to_string();
            if retain_ids.contains(&skill_id) {
                continue;
            }

            let skill_md = skill_dir.join("SKILL.md");
            if skill_md.exists()
                && let Ok(content) = fs::read_to_string(&skill_md)
                && content.starts_with("<!-- managed by ship")
            {
                fs::remove_dir_all(&skill_dir).ok();
            }
        }
    }
}

/// Remove skill subdirectories that were written by Ship (identified by the
/// `<!-- managed by ship` header in their SKILL.md).
fn remove_ship_managed_skill_dirs(skills_dir: &Path) {
    if !skills_dir.exists() {
        return;
    }
    if let Ok(entries) = fs::read_dir(skills_dir) {
        for entry in entries.flatten() {
            let skill_dir = entry.path();
            if !skill_dir.is_dir() {
                continue;
            }
            let skill_md = skill_dir.join("SKILL.md");
            if skill_md.exists()
                && let Ok(c) = fs::read_to_string(&skill_md)
                && c.starts_with("<!-- managed by ship")
            {
                fs::remove_dir_all(&skill_dir).ok();
            }
        }
    }
}
