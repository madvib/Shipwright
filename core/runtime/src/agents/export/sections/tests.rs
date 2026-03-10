#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        HookConfig, HookTrigger, McpServerConfig, McpServerType, ModeConfig, PermissionConfig,
        ProjectConfig, save_config,
    };
    use crate::permissions::{Permissions, save_permissions};
    use crate::project::init_project;
    use crate::skill::{create_skill, delete_skill};
    use std::collections::HashMap;
    use std::ffi::OsString;
    use std::sync::{Mutex, MutexGuard};
    use tempfile::tempdir;

    static HOME_ENV_LOCK: Mutex<()> = Mutex::new(());
    static HOOK_ENV_LOCK: Mutex<()> = Mutex::new(());

    struct HomeEnvGuard<'a> {
        _lock: MutexGuard<'a, ()>,
        previous_home: Option<OsString>,
    }

    impl Drop for HomeEnvGuard<'_> {
        fn drop(&mut self) {
            if let Some(value) = self.previous_home.take() {
                unsafe {
                    std::env::set_var("HOME", value);
                }
            } else {
                unsafe {
                    std::env::remove_var("HOME");
                }
            }
        }
    }

    fn lock_home_for_test(path: &std::path::Path) -> HomeEnvGuard<'static> {
        let lock = HOME_ENV_LOCK.lock().expect("HOME env lock poisoned");
        let previous_home = std::env::var_os("HOME");
        unsafe {
            std::env::set_var("HOME", path);
        }
        HomeEnvGuard {
            _lock: lock,
            previous_home,
        }
    }

    struct EnvVarGuard<'a> {
        _lock: MutexGuard<'a, ()>,
        key: &'static str,
        previous: Option<OsString>,
    }

    impl Drop for EnvVarGuard<'_> {
        fn drop(&mut self) {
            if let Some(value) = self.previous.take() {
                unsafe {
                    std::env::set_var(self.key, value);
                }
            } else {
                unsafe {
                    std::env::remove_var(self.key);
                }
            }
        }
    }

    fn lock_env_var_for_test(key: &'static str, value: Option<&str>) -> EnvVarGuard<'static> {
        let lock = HOOK_ENV_LOCK.lock().expect("hook env lock poisoned");
        let previous = std::env::var_os(key);
        match value {
            Some(next) => unsafe {
                std::env::set_var(key, next);
            },
            None => unsafe {
                std::env::remove_var(key);
            },
        }
        EnvVarGuard {
            _lock: lock,
            key,
            previous,
        }
    }

    fn make_stdio_server(id: &str) -> McpServerConfig {
        McpServerConfig {
            id: id.to_string(),
            name: id.to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), format!("@mcp/{}", id)],
            env: HashMap::new(),
            scope: "project".to_string(),
            server_type: McpServerType::Stdio,
            url: None,
            disabled: false,
            timeout_secs: None,
        }
    }

    fn make_http_server(id: &str, url: &str) -> McpServerConfig {
        McpServerConfig {
            id: id.to_string(),
            name: id.to_string(),
            command: String::new(),
            args: vec![],
            env: HashMap::new(),
            scope: "project".to_string(),
            server_type: McpServerType::Http,
            url: Some(url.to_string()),
            disabled: false,
            timeout_secs: None,
        }
    }

    fn project_with_servers(servers: Vec<McpServerConfig>) -> (tempfile::TempDir, PathBuf) {
        let tmp = tempdir().unwrap();
        let project_dir = init_project(tmp.path().to_path_buf()).unwrap();
        let config = ProjectConfig {
            mcp_servers: servers,
            ..ProjectConfig::default()
        };
        save_config(&config, Some(project_dir.clone())).unwrap();
        (tmp, project_dir)
    }

    #[test]
    fn build_payload_active_mode_filters_servers_and_applies_mode_hooks_permissions() {
        let (_tmp, project_dir) = project_with_servers(vec![
            make_stdio_server("allowed"),
            make_stdio_server("blocked"),
        ]);
        save_permissions(
            project_dir.clone(),
            &Permissions {
                tools: crate::permissions::ToolPermissions {
                    allow: vec!["Read(*)".to_string()],
                    deny: vec!["Edit(*)".to_string()],
                },
                ..Default::default()
            },
        )
        .unwrap();
        let mut config = crate::config::get_config(Some(project_dir.clone())).unwrap();
        config.hooks = vec![HookConfig {
            id: "project-global-hook".to_string(),
            trigger: HookTrigger::PreToolUse,
            matcher: Some("Bash".to_string()),
            timeout_ms: None,
            description: None,
            command: "echo global".to_string(),
        }];
        config.modes = vec![ModeConfig {
            id: "focus".to_string(),
            name: "Focus".to_string(),
            description: None,
            active_tools: vec![],
            mcp_servers: vec!["allowed".to_string()],
            skills: vec![],
            rules: vec![],
            prompt_id: None,
            hooks: vec![HookConfig {
                id: "mode-hook".to_string(),
                trigger: HookTrigger::PostToolUse,
                matcher: Some("Bash".to_string()),
                timeout_ms: None,
                description: None,
                command: "echo mode".to_string(),
            }],
            permissions: PermissionConfig {
                allow: vec!["Bash(*)".to_string()],
                deny: vec!["WebFetch(*)".to_string()],
            },
            target_agents: vec![],
        }];
        config.active_mode = Some("focus".to_string());
        save_config(&config, Some(project_dir.clone())).unwrap();

        let payload = build_payload(&project_dir).unwrap();
        let server_ids: Vec<_> = payload.servers.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(server_ids, vec!["allowed"]);
        assert_eq!(payload.active_mode_id.as_deref(), Some("focus"));
        assert_eq!(payload.permissions.tools.allow, vec!["Bash(*)".to_string()]);
        assert_eq!(
            payload.permissions.tools.deny,
            vec!["WebFetch(*)".to_string()]
        );

        let hook_ids: Vec<_> = payload.hooks.iter().map(|h| h.id.as_str()).collect();
        let global_idx = hook_ids
            .iter()
            .position(|id| *id == "project-global-hook")
            .expect("global hook missing");
        let mode_idx = hook_ids
            .iter()
            .position(|id| *id == "mode-hook")
            .expect("mode hook missing");
        assert!(
            global_idx < mode_idx,
            "mode hooks must append after global hooks"
        );
    }

    #[test]
    fn build_payload_without_active_mode_uses_permissions_toml() {
        let (_tmp, project_dir) = project_with_servers(vec![make_stdio_server("github")]);
        save_permissions(
            project_dir.clone(),
            &Permissions {
                tools: crate::permissions::ToolPermissions {
                    allow: vec!["Read(*)".to_string()],
                    deny: vec!["Edit(*)".to_string()],
                },
                ..Default::default()
            },
        )
        .unwrap();
        let mut config = crate::config::get_config(Some(project_dir.clone())).unwrap();
        config.hooks = vec![HookConfig {
            id: "project-global-hook-only".to_string(),
            trigger: HookTrigger::Notification,
            matcher: None,
            timeout_ms: None,
            description: None,
            command: "echo global".to_string(),
        }];
        config.modes = vec![ModeConfig {
            id: "unused".to_string(),
            name: "Unused".to_string(),
            description: None,
            active_tools: vec![],
            mcp_servers: vec![],
            skills: vec![],
            rules: vec![],
            prompt_id: None,
            hooks: vec![HookConfig {
                id: "unused-mode-hook".to_string(),
                trigger: HookTrigger::Stop,
                matcher: None,
                timeout_ms: None,
                description: None,
                command: "echo unused".to_string(),
            }],
            permissions: PermissionConfig {
                allow: vec!["Bash(*)".to_string()],
                deny: vec!["WebFetch(*)".to_string()],
            },
            target_agents: vec![],
        }];
        config.active_mode = None;
        save_config(&config, Some(project_dir.clone())).unwrap();

        let payload = build_payload(&project_dir).unwrap();
        assert_eq!(payload.active_mode_id, None);
        assert_eq!(payload.permissions.tools.allow, vec!["Read(*)".to_string()]);
        assert_eq!(payload.permissions.tools.deny, vec!["Edit(*)".to_string()]);
        assert!(
            payload
                .hooks
                .iter()
                .any(|hook| hook.id == "project-global-hook-only")
        );
        assert!(
            !payload
                .hooks
                .iter()
                .any(|hook| hook.id == "unused-mode-hook")
        );
        assert!(payload.servers.iter().any(|server| server.id == "github"));
    }

    #[test]
    fn build_payload_mode_overrides_replace_only_tool_permissions() {
        let (_tmp, project_dir) = project_with_servers(vec![make_stdio_server("github")]);
        save_permissions(
            project_dir.clone(),
            &Permissions {
                tools: crate::permissions::ToolPermissions {
                    allow: vec!["Read(*)".to_string()],
                    deny: vec!["Edit(*)".to_string()],
                },
                network: crate::permissions::NetworkPermissions {
                    policy: crate::permissions::NetworkPolicy::AllowList,
                    allow_hosts: vec!["api.example.com".to_string()],
                },
                ..Default::default()
            },
        )
        .unwrap();

        let mut config = crate::config::get_config(Some(project_dir.clone())).unwrap();
        config.modes = vec![ModeConfig {
            id: "focus".to_string(),
            name: "Focus".to_string(),
            permissions: PermissionConfig {
                allow: vec!["Bash(*)".to_string()],
                deny: vec!["WebFetch(*)".to_string()],
            },
            ..Default::default()
        }];
        config.active_mode = Some("focus".to_string());
        save_config(&config, Some(project_dir.clone())).unwrap();

        let payload = build_payload(&project_dir).unwrap();
        assert_eq!(payload.permissions.tools.allow, vec!["Bash(*)".to_string()]);
        assert_eq!(
            payload.permissions.tools.deny,
            vec!["WebFetch(*)".to_string()]
        );
        assert_eq!(
            payload.permissions.network.policy,
            crate::permissions::NetworkPolicy::AllowList
        );
        assert_eq!(
            payload.permissions.network.allow_hosts,
            vec!["api.example.com".to_string()]
        );
    }

    #[test]
    fn build_payload_workspace_mode_override_takes_precedence() {
        let (_tmp, project_dir) = project_with_servers(vec![
            make_stdio_server("planning-server"),
            make_stdio_server("code-server"),
        ]);
        save_permissions(
            project_dir.clone(),
            &Permissions {
                tools: crate::permissions::ToolPermissions {
                    allow: vec!["Read(*)".to_string()],
                    deny: vec!["Edit(*)".to_string()],
                },
                ..Default::default()
            },
        )
        .unwrap();

        let mut config = crate::config::get_config(Some(project_dir.clone())).unwrap();
        config.active_mode = Some("planning".to_string());
        config.modes = vec![
            ModeConfig {
                id: "planning".to_string(),
                name: "Planning".to_string(),
                mcp_servers: vec!["planning-server".to_string()],
                permissions: PermissionConfig {
                    allow: vec!["WebFetch(*)".to_string()],
                    deny: vec!["Bash(*)".to_string()],
                },
                ..Default::default()
            },
            ModeConfig {
                id: "code".to_string(),
                name: "Code".to_string(),
                mcp_servers: vec!["code-server".to_string()],
                permissions: PermissionConfig {
                    allow: vec!["Bash(*)".to_string()],
                    deny: vec!["WebFetch(*)".to_string()],
                },
                ..Default::default()
            },
        ];
        save_config(&config, Some(project_dir.clone())).unwrap();

        let payload = build_payload_with_mode_override(&project_dir, Some("code")).unwrap();
        assert_eq!(payload.active_mode_id.as_deref(), Some("code"));
        let server_ids: Vec<_> = payload.servers.iter().map(|server| server.id.as_str()).collect();
        assert_eq!(server_ids, vec!["code-server"]);
        assert_eq!(payload.permissions.tools.allow, vec!["Bash(*)".to_string()]);
        assert_eq!(
            payload.permissions.tools.deny,
            vec!["WebFetch(*)".to_string()]
        );
    }

    #[test]
    fn sync_active_mode_uses_connected_providers_when_targets_empty() {
        let (tmp, project_dir) = project_with_servers(vec![make_stdio_server("github")]);
        let mut config = crate::config::get_config(Some(project_dir.clone())).unwrap();
        config.providers = vec!["codex".to_string()];
        config.modes = vec![ModeConfig {
            id: "focus".to_string(),
            name: "Focus".to_string(),
            target_agents: vec![],
            ..Default::default()
        }];
        config.active_mode = Some("focus".to_string());
        save_config(&config, Some(project_dir.clone())).unwrap();

        let synced = sync_active_mode(&project_dir).unwrap();
        assert_eq!(synced, vec!["codex".to_string()]);
        assert!(tmp.path().join(".codex").join("config.toml").exists());
    }

    #[test]
    fn sync_active_mode_without_active_mode_uses_connected_providers() {
        let (tmp, project_dir) = project_with_servers(vec![make_stdio_server("github")]);
        let mut config = crate::config::get_config(Some(project_dir.clone())).unwrap();
        config.providers = vec!["gemini".to_string()];
        config.active_mode = None;
        save_config(&config, Some(project_dir.clone())).unwrap();

        let synced = sync_active_mode(&project_dir).unwrap();
        assert_eq!(synced, vec!["gemini".to_string()]);
        assert!(tmp.path().join(".gemini").join("settings.json").exists());
    }

    #[test]
    fn sync_active_mode_normalizes_targets_and_skips_unknown_values() {
        let (tmp, project_dir) = project_with_servers(vec![make_stdio_server("github")]);
        let mut config = crate::config::get_config(Some(project_dir.clone())).unwrap();
        config.modes = vec![ModeConfig {
            id: "focus".to_string(),
            name: "Focus".to_string(),
            target_agents: vec![
                " codex ".to_string(),
                "unknown-agent".to_string(),
                "CLAUDE".to_string(),
                "claude".to_string(),
                "".to_string(),
            ],
            ..Default::default()
        }];
        config.active_mode = Some("focus".to_string());
        save_config(&config, Some(project_dir.clone())).unwrap();

        let synced = sync_active_mode(&project_dir).unwrap();
        assert_eq!(
            synced,
            vec!["codex".to_string(), "claude".to_string()],
            "targets should be normalized, deduped, and unknown providers skipped"
        );
        assert!(tmp.path().join(".codex").join("config.toml").exists());
        assert!(tmp.path().join(".mcp.json").exists());
    }

    #[test]
    fn sync_active_mode_with_override_uses_override_targets() {
        let (tmp, project_dir) = project_with_servers(vec![make_stdio_server("github")]);
        let mut config = crate::config::get_config(Some(project_dir.clone())).unwrap();
        config.providers = vec!["claude".to_string()];
        config.active_mode = Some("planning".to_string());
        config.modes = vec![
            ModeConfig {
                id: "planning".to_string(),
                name: "Planning".to_string(),
                target_agents: vec!["claude".to_string()],
                ..Default::default()
            },
            ModeConfig {
                id: "code".to_string(),
                name: "Code".to_string(),
                target_agents: vec!["codex".to_string()],
                ..Default::default()
            },
        ];
        save_config(&config, Some(project_dir.clone())).unwrap();

        let synced = sync_active_mode_with_override(&project_dir, Some("code")).unwrap();
        assert_eq!(synced, vec!["codex".to_string()]);
        assert!(tmp.path().join(".codex").join("config.toml").exists());
    }

    // ── Registry ───────────────────────────────────────────────────────────────

    #[test]
    fn all_provider_ids_are_unique() {
        let ids: Vec<_> = PROVIDERS.iter().map(|p| p.id).collect();
        let mut seen = std::collections::HashSet::new();
        for id in &ids {
            assert!(seen.insert(id), "duplicate provider id: {}", id);
        }
    }

    #[test]
    fn require_provider_errors_on_unknown() {
        let err = require_provider("vscode").unwrap_err();
        assert!(err.to_string().contains("vscode"));
        assert!(err.to_string().contains("claude"));
    }

    #[test]
    fn list_providers_discovers_models_from_provider_config() {
        let (tmp, project_dir) = project_with_servers(vec![]);
        let gemini_dir = tmp.path().join(".gemini");
        std::fs::create_dir_all(&gemini_dir).unwrap();
        std::fs::write(
            gemini_dir.join("settings.json"),
            r#"{
  "model": "gemini-2.5-pro",
  "modelConfigs": {
    "fast": { "name": "gemini-2.0-flash" }
  }
}"#,
        )
        .unwrap();

        let providers = list_providers(&project_dir).unwrap();
        let gemini = providers
            .iter()
            .find(|provider| provider.id == "gemini")
            .expect("gemini provider should exist");
        let ids: Vec<&str> = gemini.models.iter().map(|model| model.id.as_str()).collect();
        assert!(ids.contains(&"gemini-2.5-pro"));
        assert!(ids.contains(&"gemini-2.0-flash"));
    }

    // ── Claude ─────────────────────────────────────────────────────────────────

    #[test]
    fn claude_writes_mcp_json_at_project_root() {
        let (tmp, project_dir) = project_with_servers(vec![make_stdio_server("github")]);
        export_to(project_dir, "claude").unwrap();
        assert!(tmp.path().join(".mcp.json").exists());
    }

    #[test]
    fn claude_round_trip_stdio_server() {
        let (tmp, project_dir) = project_with_servers(vec![make_stdio_server("github")]);
        export_to(project_dir, "claude").unwrap();
        let val: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(tmp.path().join(".mcp.json")).unwrap())
                .unwrap();
        let mcp = val["mcpServers"]["github"].as_object().unwrap();
        assert_eq!(mcp["command"].as_str().unwrap(), "npx");
        assert_eq!(mcp["type"].as_str().unwrap(), "stdio");
    }

    #[test]
    fn claude_round_trip_http_server() {
        let (tmp, project_dir) = project_with_servers(vec![make_http_server(
            "postgres",
            "http://localhost:5433/mcp",
        )]);
        export_to(project_dir, "claude").unwrap();
        let val: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(tmp.path().join(".mcp.json")).unwrap())
                .unwrap();
        assert_eq!(
            val["mcpServers"]["postgres"]["type"].as_str().unwrap(),
            "http"
        );
        assert_eq!(
            val["mcpServers"]["postgres"]["url"].as_str().unwrap(),
            "http://localhost:5433/mcp"
        );
    }

    #[test]
    fn claude_ship_server_always_injected() {
        let (tmp, project_dir) = project_with_servers(vec![]);
        export_to(project_dir, "claude").unwrap();
        let val: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(tmp.path().join(".mcp.json")).unwrap())
                .unwrap();
        assert!(val["mcpServers"]["ship"].is_object());
    }

    #[test]
    fn claude_marks_managed_servers() {
        let (tmp, project_dir) = project_with_servers(vec![make_stdio_server("github")]);
        export_to(project_dir, "claude").unwrap();
        let val: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(tmp.path().join(".mcp.json")).unwrap())
                .unwrap();
        assert_eq!(
            val["mcpServers"]["github"]["_ship"]["managed"].as_bool(),
            Some(true)
        );
    }

    #[test]
    fn claude_preserves_user_servers_across_write() {
        let (tmp, project_dir) = project_with_servers(vec![make_stdio_server("mine")]);
        export_to(project_dir.clone(), "claude").unwrap();
        let mcp_json = tmp.path().join(".mcp.json");
        let mut val: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&mcp_json).unwrap()).unwrap();
        val["mcpServers"]["user-server"] =
            serde_json::json!({ "command": "user-tool", "args": [] });
        std::fs::write(&mcp_json, serde_json::to_string_pretty(&val).unwrap()).unwrap();
        export_to(project_dir, "claude").unwrap();
        let val2: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&mcp_json).unwrap()).unwrap();
        assert!(
            val2["mcpServers"]["user-server"].is_object(),
            "user server was clobbered"
        );
    }

    #[test]
    fn claude_disabled_server_not_exported() {
        let mut s = make_stdio_server("disabled-one");
        s.disabled = true;
        let (tmp, project_dir) = project_with_servers(vec![s]);
        export_to(project_dir, "claude").unwrap();
        let val: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(tmp.path().join(".mcp.json")).unwrap())
                .unwrap();
        assert!(val["mcpServers"]["disabled-one"].is_null());
    }

    #[test]
    fn claude_managed_state_written() {
        let (_tmp, project_dir) = project_with_servers(vec![make_stdio_server("gh")]);
        export_to(project_dir.clone(), "claude").unwrap();
        let (ids, _mode) = crate::state_db::get_managed_state_db(&project_dir, "claude").unwrap();
        assert!(
            ids.contains(&"gh".to_string()),
            "managed server not recorded in state"
        );
        // Clean up DB created in ~/.ship/state/ for this temp project
        std::fs::remove_file(crate::state_db::project_db_path(&project_dir).unwrap()).ok();
    }

    #[test]
    fn claude_permissions_round_trip_imports_back_to_canonical() {
        let (_tmp, project_dir) = project_with_servers(vec![]);
        let home = tempdir().unwrap();
        let _home_guard = lock_home_for_test(home.path());
        save_permissions(
            project_dir.clone(),
            &Permissions {
                tools: crate::permissions::ToolPermissions {
                    allow: vec!["Bash(*)".to_string()],
                    deny: vec!["WebFetch(*)".to_string()],
                },
                ..Default::default()
            },
        )
        .unwrap();

        export_to(project_dir.clone(), "claude").unwrap();

        save_permissions(project_dir.clone(), &Permissions::default()).unwrap();
        let imported = import_permissions_from_provider("claude", project_dir.clone()).unwrap();
        assert!(imported);
        let restored = crate::permissions::get_permissions(project_dir).unwrap();
        assert_eq!(restored.tools.allow, vec!["Bash(*)".to_string()]);
        assert_eq!(restored.tools.deny, vec!["WebFetch(*)".to_string()]);
    }

    #[test]
    fn claude_exports_grouped_hook_schema_with_metadata() {
        let (_tmp, project_dir) = project_with_servers(vec![]);
        let home = tempdir().unwrap();
        let _home_guard = lock_home_for_test(home.path());
        let _managed_guard = lock_env_var_for_test("SHIP_MANAGED_HOOKS", Some("1"));

        let mut config = crate::config::get_config(Some(project_dir.clone())).unwrap();
        config.hooks = vec![
            HookConfig {
                id: "session-context".to_string(),
                trigger: HookTrigger::SessionStart,
                matcher: None,
                timeout_ms: Some(2000),
                description: Some("Inject workspace context".to_string()),
                command: "$SHIP_HOOKS_BIN".to_string(),
            },
            HookConfig {
                id: "tool-guard".to_string(),
                trigger: HookTrigger::PreToolUse,
                matcher: Some("Bash".to_string()),
                timeout_ms: Some(1500),
                description: Some("Validate command scope".to_string()),
                command: "$SHIP_HOOKS_BIN".to_string(),
            },
        ];
        save_config(&config, Some(project_dir.clone())).unwrap();

        export_to(project_dir, "claude").unwrap();
        let settings_path = home.path().join(".claude").join("settings.json");
        let val: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(settings_path).unwrap()).unwrap();

        let session_hook = &val["hooks"]["SessionStart"][0]["hooks"][0];
        assert_eq!(session_hook["type"].as_str(), Some("command"));
        assert_eq!(session_hook["command"].as_str(), Some("$SHIP_HOOKS_BIN"));
        assert_eq!(session_hook["timeout"].as_u64(), Some(2000));
        assert_eq!(
            session_hook["description"].as_str(),
            Some("Inject workspace context")
        );

        let pre_tool_group = &val["hooks"]["PreToolUse"][0];
        assert_eq!(pre_tool_group["matcher"].as_str(), Some("Bash"));
        assert_eq!(
            pre_tool_group["hooks"][0]["description"].as_str(),
            Some("Validate command scope")
        );
    }

    #[test]
    fn claude_exports_ship_managed_hooks_baseline_when_no_custom_hooks() {
        let (_tmp, project_dir) = project_with_servers(vec![]);
        let home = tempdir().unwrap();
        let _home_guard = lock_home_for_test(home.path());
        let _managed_guard = lock_env_var_for_test("SHIP_MANAGED_HOOKS", Some("1"));

        export_to(project_dir, "claude").unwrap();
        let settings_path = home.path().join(".claude").join("settings.json");
        let val: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(settings_path).unwrap()).unwrap();

        let session_start = &val["hooks"]["SessionStart"][0]["hooks"][0];
        assert_eq!(
            session_start["command"].as_str(),
            Some("ship hooks run --provider claude")
        );
        let pre_tool = &val["hooks"]["PreToolUse"][0];
        assert_eq!(pre_tool["matcher"].as_str(), Some("Bash"));
    }

    // ── Gemini ─────────────────────────────────────────────────────────────────

    #[test]
    fn gemini_writes_to_gemini_settings_json() {
        let (tmp, project_dir) = project_with_servers(vec![make_stdio_server("git")]);
        export_to(project_dir, "gemini").unwrap();
        assert!(tmp.path().join(".gemini/settings.json").exists());
    }

    #[test]
    fn gemini_http_uses_httpurl_not_url() {
        let (tmp, project_dir) =
            project_with_servers(vec![make_http_server("figma", "https://mcp.figma.com/mcp")]);
        export_to(project_dir, "gemini").unwrap();
        let val: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(tmp.path().join(".gemini/settings.json")).unwrap(),
        )
        .unwrap();
        assert!(
            val["mcpServers"]["figma"]["httpUrl"].is_string(),
            "Gemini must use httpUrl"
        );
        assert!(
            val["mcpServers"]["figma"]["url"].is_null(),
            "Gemini must not use url"
        );
    }

    #[test]
    fn gemini_preserves_non_mcp_fields() {
        let (tmp, project_dir) = project_with_servers(vec![make_stdio_server("git")]);
        let settings_dir = tmp.path().join(".gemini");
        std::fs::create_dir_all(&settings_dir).unwrap();
        std::fs::write(
            settings_dir.join("settings.json"),
            r#"{"theme": "Dracula", "selectedAuthType": "gemini-api-key", "mcpServers": {}}"#,
        )
        .unwrap();
        export_to(project_dir, "gemini").unwrap();
        let val: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(tmp.path().join(".gemini/settings.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(val["theme"].as_str().unwrap(), "Dracula");
        assert_eq!(val["selectedAuthType"].as_str().unwrap(), "gemini-api-key");
    }

    #[test]
    fn gemini_ship_server_always_injected() {
        let (tmp, project_dir) = project_with_servers(vec![]);
        export_to(project_dir, "gemini").unwrap();
        let val: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(tmp.path().join(".gemini/settings.json")).unwrap(),
        )
        .unwrap();
        assert!(val["mcpServers"]["ship"].is_object());
    }

    #[test]
    fn gemini_exports_workspace_policy_from_permissions() {
        let (tmp, project_dir) = project_with_servers(vec![]);
        save_permissions(
            project_dir.clone(),
            &Permissions {
                tools: crate::permissions::ToolPermissions {
                    allow: vec!["mcp__ship__*".to_string()],
                    deny: vec!["WebFetch(*)".to_string()],
                },
                commands: crate::permissions::CommandPermissions {
                    allow: vec!["git status*".to_string()],
                    deny: vec!["rm -rf *".to_string()],
                },
                agent: crate::permissions::AgentLimits {
                    require_confirmation: vec!["git push *".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .unwrap();

        export_to(project_dir, "gemini").unwrap();
        let policy_path = tmp
            .path()
            .join(".gemini")
            .join("policies")
            .join("ship-permissions.toml");
        assert!(policy_path.exists());
        let content = std::fs::read_to_string(policy_path).unwrap();
        assert!(content.contains("toolName = \"run_shell_command\""));
        assert!(content.contains("commandPrefix = \"git status\""));
        assert!(content.contains("decision = \"ask_user\""));
    }

    #[test]
    fn gemini_exports_hooks_to_settings_json() {
        let (tmp, project_dir) = project_with_servers(vec![]);
        let _managed_guard = lock_env_var_for_test("SHIP_MANAGED_HOOKS", Some("1"));
        let mut config = crate::config::get_config(Some(project_dir.clone())).unwrap();
        config.hooks = vec![HookConfig {
            id: "before-tool-guard".to_string(),
            trigger: HookTrigger::PreToolUse,
            matcher: Some("run_shell_command".to_string()),
            timeout_ms: Some(1200),
            description: Some("Decompose chained shell command".to_string()),
            command: "$SHIP_HOOKS_BIN".to_string(),
        }];
        save_config(&config, Some(project_dir.clone())).unwrap();

        export_to(project_dir, "gemini").unwrap();
        let val: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(tmp.path().join(".gemini/settings.json")).unwrap(),
        )
        .unwrap();

        let group = &val["hooks"]["BeforeTool"][0];
        assert_eq!(group["matcher"].as_str(), Some("run_shell_command"));
        let hook = &group["hooks"][0];
        assert_eq!(hook["name"].as_str(), Some("before-tool-guard"));
        assert_eq!(hook["type"].as_str(), Some("command"));
        assert_eq!(hook["command"].as_str(), Some("$SHIP_HOOKS_BIN"));
        assert_eq!(hook["timeout"].as_u64(), Some(1200));
        assert_eq!(
            hook["description"].as_str(),
            Some("Decompose chained shell command")
        );
    }

    #[test]
    fn gemini_exports_ship_managed_hook_commands_with_provider_hint() {
        let (tmp, project_dir) = project_with_servers(vec![]);
        let _managed_guard = lock_env_var_for_test("SHIP_MANAGED_HOOKS", Some("1"));

        export_to(project_dir, "gemini").unwrap();
        let settings_path = tmp.path().join(".gemini").join("settings.json");
        let val: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(settings_path).unwrap()).unwrap();

        let session_start = &val["hooks"]["SessionStart"][0]["hooks"][0];
        assert_eq!(
            session_start["command"].as_str(),
            Some("ship hooks run --provider gemini")
        );
        let before_tool = &val["hooks"]["BeforeTool"][0]["hooks"][0];
        assert_eq!(
            before_tool["command"].as_str(),
            Some("ship hooks run --provider gemini")
        );
    }

    #[test]
    fn export_writes_hook_runtime_artifacts() {
        let (tmp, project_dir) = project_with_servers(vec![make_stdio_server("github")]);
        let _managed_guard = lock_env_var_for_test("SHIP_MANAGED_HOOKS", Some("1"));
        save_permissions(
            project_dir.clone(),
            &Permissions {
                filesystem: crate::permissions::FsPermissions {
                    allow: vec!["src/auth/**".to_string()],
                    deny: vec![],
                },
                commands: crate::permissions::CommandPermissions {
                    allow: vec!["git status*".to_string()],
                    deny: vec!["rm -rf *".to_string()],
                },
                ..Default::default()
            },
        )
        .unwrap();

        export_to(project_dir, "gemini").unwrap();

        let runtime_dir = tmp.path().join(".ship").join("agents").join("runtime");
        let envelope_path = runtime_dir.join("envelope.json");
        let context_path = runtime_dir.join("hook-context.md");
        assert!(envelope_path.exists(), "expected hook envelope file");
        assert!(context_path.exists(), "expected hook context file");

        let envelope: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(envelope_path).unwrap()).unwrap();
        let expected_root = tmp.path().to_string_lossy().to_string();
        assert_eq!(envelope["workspace_root"].as_str(), Some(expected_root.as_str()));
        assert!(envelope["auto_approve_patterns"].is_array());
        assert!(envelope["always_block_patterns"].is_array());
    }

    #[test]
    fn gemini_permissions_round_trip_imports_back_to_canonical() {
        let (tmp, project_dir) = project_with_servers(vec![]);
        let policy_dir = tmp.path().join(".gemini").join("policies");
        std::fs::create_dir_all(&policy_dir).unwrap();
        std::fs::write(
            policy_dir.join("ship-permissions.toml"),
            r#"
[[rule]]
toolName = "run_shell_command"
commandPrefix = "git "
decision = "allow"
priority = 100

[[rule]]
toolName = "run_shell_command"
commandPrefix = "rm -rf "
decision = "deny"
priority = 900

[[rule]]
toolName = "run_shell_command"
commandPrefix = "git push "
decision = "ask_user"
priority = 800

[[rule]]
toolName = "mcp__ship__*"
decision = "allow"
priority = 700
"#,
        )
        .unwrap();

        let imported = import_permissions_from_provider("gemini", project_dir.clone()).unwrap();
        assert!(imported);
        let restored = crate::permissions::get_permissions(project_dir).unwrap();
        assert!(
            restored.commands.allow.contains(&"git *".to_string()),
            "expected command allow imported from Gemini policy"
        );
        assert!(restored.commands.deny.contains(&"rm -rf *".to_string()));
        assert!(
            restored
                .agent
                .require_confirmation
                .contains(&"git push *".to_string())
        );
        assert!(restored.tools.allow.contains(&"mcp__ship__*".to_string()));
    }

    // ── Codex ──────────────────────────────────────────────────────────────────

    #[test]
    fn codex_writes_to_codex_config_toml() {
        let (tmp, project_dir) = project_with_servers(vec![make_stdio_server("gh")]);
        export_to(project_dir, "codex").unwrap();
        assert!(tmp.path().join(".codex/config.toml").exists());
    }

    #[test]
    fn codex_uses_mcp_servers_underscore_not_hyphen() {
        let (tmp, project_dir) = project_with_servers(vec![make_stdio_server("gh")]);
        export_to(project_dir, "codex").unwrap();
        let content = std::fs::read_to_string(tmp.path().join(".codex/config.toml")).unwrap();
        assert!(
            content.contains("[mcp_servers."),
            "must use mcp_servers (underscore)"
        );
        assert!(
            !content.contains("[mcp-servers."),
            "must NOT use mcp-servers (hyphen)"
        );
    }

    #[test]
    fn codex_round_trip_stdio_server() {
        let (tmp, project_dir) = project_with_servers(vec![make_stdio_server("gh")]);
        export_to(project_dir, "codex").unwrap();
        let val: toml::Value = toml::from_str(
            &std::fs::read_to_string(tmp.path().join(".codex/config.toml")).unwrap(),
        )
        .unwrap();
        assert_eq!(val["mcp_servers"]["gh"]["command"].as_str().unwrap(), "npx");
    }

    #[test]
    fn codex_preserves_user_servers() {
        let (tmp, project_dir) = project_with_servers(vec![make_stdio_server("mine")]);
        export_to(project_dir.clone(), "codex").unwrap();
        let config_path = tmp.path().join(".codex/config.toml");
        let mut content = std::fs::read_to_string(&config_path).unwrap();
        content.push_str("\n[mcp_servers.user-tool]\ncommand = \"user-tool\"\n");
        std::fs::write(&config_path, &content).unwrap();
        export_to(project_dir, "codex").unwrap();
        let val: toml::Value =
            toml::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();
        assert!(
            val["mcp_servers"]["user-tool"].is_table(),
            "user server was clobbered"
        );
    }

    #[test]
    fn codex_ship_server_always_injected() {
        let (tmp, project_dir) = project_with_servers(vec![]);
        export_to(project_dir, "codex").unwrap();
        let val: toml::Value = toml::from_str(
            &std::fs::read_to_string(tmp.path().join(".codex/config.toml")).unwrap(),
        )
        .unwrap();
        assert!(val["mcp_servers"]["ship"].is_table());
    }

    #[test]
    fn codex_exports_permissions_to_native_fields() {
        let (tmp, project_dir) = project_with_servers(vec![]);
        save_permissions(
            project_dir.clone(),
            &Permissions {
                commands: crate::permissions::CommandPermissions {
                    allow: vec!["cargo *".to_string()],
                    deny: vec!["rm -rf *".to_string()],
                },
                network: crate::permissions::NetworkPermissions {
                    policy: crate::permissions::NetworkPolicy::AllowList,
                    allow_hosts: vec!["github.com".to_string()],
                },
                agent: crate::permissions::AgentLimits {
                    require_confirmation: vec!["git push *".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .unwrap();

        export_to(project_dir, "codex").unwrap();
        let val: toml::Value = toml::from_str(
            &std::fs::read_to_string(tmp.path().join(".codex/config.toml")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            val["sandbox_mode"].as_str(),
            Some("workspace-write"),
            "codex export should enforce workspace-write sandbox for mapped permissions"
        );
        assert_eq!(
            val["sandbox_workspace_write"]["network_access"].as_bool(),
            Some(true)
        );
        assert_eq!(val["approval_policy"].as_str(), Some("on-request"));
        assert!(val["rules"]["prefix_rules"].is_array());
    }

    #[test]
    fn codex_permissions_round_trip_imports_back_to_canonical() {
        let (tmp, project_dir) = project_with_servers(vec![]);
        let codex_dir = tmp.path().join(".codex");
        std::fs::create_dir_all(&codex_dir).unwrap();
        std::fs::write(
            codex_dir.join("config.toml"),
            r#"
sandbox_mode = "workspace-write"
approval_policy = "on-request"
allow = ["cargo *"]

[sandbox_workspace_write]
network_access = false

[rules]
prefix_rules = [
  { prefix = "rm -rf ", decision = "forbidden" },
  { prefix = "git push ", decision = "prompt" }
]
"#,
        )
        .unwrap();

        let imported = import_permissions_from_provider("codex", project_dir.clone()).unwrap();
        assert!(imported);
        let restored = crate::permissions::get_permissions(project_dir).unwrap();
        assert_eq!(
            restored.network.policy,
            crate::permissions::NetworkPolicy::None
        );
        assert!(restored.commands.allow.contains(&"cargo *".to_string()));
        assert!(restored.commands.deny.contains(&"rm -rf *".to_string()));
        assert!(
            restored
                .agent
                .require_confirmation
                .contains(&"git push *".to_string())
        );
    }

    #[test]
    fn codex_export_prunes_stale_managed_skill_dirs() {
        let (tmp, project_dir) = project_with_servers(vec![]);
        create_skill(&project_dir, "rt-live-skill", "Live", "live body").unwrap();
        create_skill(&project_dir, "rt-stale-skill", "Stale", "stale body").unwrap();

        export_to(project_dir.clone(), "codex").unwrap();
        let skills_dir = tmp.path().join(".agents").join("skills");
        let live_skill_dir = skills_dir.join("rt-live-skill");
        let stale_skill_dir = skills_dir.join("rt-stale-skill");
        assert!(live_skill_dir.join("SKILL.md").exists());
        assert!(stale_skill_dir.join("SKILL.md").exists());

        delete_skill(&project_dir, "rt-stale-skill").unwrap();
        export_to(project_dir, "codex").unwrap();

        assert!(live_skill_dir.join("SKILL.md").exists());
        assert!(
            !stale_skill_dir.exists(),
            "stale managed skill directory should be pruned on export"
        );
    }

    #[test]
    fn codex_export_applies_active_mode_skill_filter() {
        let (tmp, project_dir) = project_with_servers(vec![]);
        create_skill(&project_dir, "rt-allowed-skill", "Allowed", "allowed body").unwrap();
        create_skill(&project_dir, "rt-blocked-skill", "Blocked", "blocked body").unwrap();

        let mut config = crate::config::get_config(Some(project_dir.clone())).unwrap();
        config.modes = vec![ModeConfig {
            id: "focus".to_string(),
            name: "Focus".to_string(),
            description: None,
            active_tools: vec![],
            mcp_servers: vec![],
            skills: vec!["rt-allowed-skill".to_string()],
            rules: vec![],
            prompt_id: None,
            hooks: vec![],
            permissions: PermissionConfig::default(),
            target_agents: vec![],
        }];
        config.active_mode = Some("focus".to_string());
        save_config(&config, Some(project_dir.clone())).unwrap();

        export_to(project_dir, "codex").unwrap();
        let skills_dir = tmp.path().join(".agents").join("skills");
        assert!(
            skills_dir
                .join("rt-allowed-skill")
                .join("SKILL.md")
                .exists()
        );
        assert!(
            !skills_dir.join("rt-blocked-skill").exists(),
            "skills excluded by active mode should not be exported"
        );
    }

    #[test]
    fn codex_export_preserves_unmanaged_skill_dirs() {
        let (tmp, project_dir) = project_with_servers(vec![]);
        let unmanaged_dir = tmp
            .path()
            .join(".agents")
            .join("skills")
            .join("rt-unmanaged-skill");
        std::fs::create_dir_all(&unmanaged_dir).unwrap();
        let unmanaged_file = unmanaged_dir.join("SKILL.md");
        std::fs::write(&unmanaged_file, "manual skill content").unwrap();

        export_to(project_dir, "codex").unwrap();

        assert!(unmanaged_dir.exists());
        let content = std::fs::read_to_string(&unmanaged_file).unwrap();
        assert_eq!(content, "manual skill content");
    }

    // ── Import ─────────────────────────────────────────────────────────────────

    #[test]
    fn import_from_claude_reads_project_config() {
        let (tmp, project_dir) = project_with_servers(vec![]);
        std::fs::write(
            tmp.path().join(".mcp.json"),
            r#"{
  "mcpServers": {
    "github": { "command": "npx", "args": ["-y", "@mcp/github"], "type": "stdio" }
  }
}"#,
        )
        .unwrap();

        let added = import_from_provider("claude", project_dir.clone()).unwrap();
        assert_eq!(added, 1);
        let reloaded = crate::config::get_config(Some(project_dir)).unwrap();
        assert!(reloaded.mcp_servers.iter().any(|s| s.id == "github"));
    }

    #[test]
    fn import_from_gemini_reads_project_config() {
        let (tmp, project_dir) = project_with_servers(vec![]);
        let gemini_dir = tmp.path().join(".gemini");
        std::fs::create_dir_all(&gemini_dir).unwrap();
        std::fs::write(
            gemini_dir.join("settings.json"),
            r#"{
  "mcpServers": {
    "figma": { "httpUrl": "https://mcp.figma.com/mcp" }
  }
}"#,
        )
        .unwrap();

        let added = import_from_provider("gemini", project_dir.clone()).unwrap();
        assert_eq!(added, 1);
        let reloaded = crate::config::get_config(Some(project_dir)).unwrap();
        let figma = reloaded
            .mcp_servers
            .iter()
            .find(|s| s.id == "figma")
            .expect("figma server should be imported");
        assert_eq!(figma.server_type, McpServerType::Http);
        assert_eq!(figma.url.as_deref(), Some("https://mcp.figma.com/mcp"));
    }

    #[test]
    fn import_from_codex_reads_project_config() {
        let (tmp, project_dir) = project_with_servers(vec![]);
        let codex_dir = tmp.path().join(".codex");
        std::fs::create_dir_all(&codex_dir).unwrap();
        std::fs::write(
            codex_dir.join("config.toml"),
            r#"[mcp_servers.github]
command = "npx"
args = ["-y", "@mcp/github"]
"#,
        )
        .unwrap();

        let added = import_from_provider("codex", project_dir.clone()).unwrap();
        assert_eq!(added, 1);
        let reloaded = crate::config::get_config(Some(project_dir)).unwrap();
        let gh = reloaded
            .mcp_servers
            .iter()
            .find(|s| s.id == "github")
            .expect("github server should be imported");
        assert_eq!(gh.command, "npx");
        assert_eq!(gh.server_type, McpServerType::Stdio);
    }

    #[test]
    fn import_from_provider_dedupes_existing_server_ids() {
        let (_tmp, project_dir) = project_with_servers(vec![]);
        let mut config = crate::config::get_config(Some(project_dir.clone())).unwrap();
        config.mcp_servers.push(make_stdio_server("github"));
        save_config(&config, Some(project_dir.clone())).unwrap();

        let project_root = project_dir.parent().unwrap();
        std::fs::write(
            project_root.join(".mcp.json"),
            r#"{
  "mcpServers": {
    "github": { "command": "npx", "args": ["-y", "@mcp/github"], "type": "stdio" }
  }
}"#,
        )
        .unwrap();

        let added = import_from_provider("claude", project_dir.clone()).unwrap();
        assert_eq!(added, 0);
        let reloaded = crate::config::get_config(Some(project_dir)).unwrap();
        assert_eq!(
            reloaded
                .mcp_servers
                .iter()
                .filter(|s| s.id == "github")
                .count(),
            1
        );
    }

    #[test]
    fn import_from_claude_uses_global_fallback_when_project_config_missing() {
        let (_tmp, project_dir) = project_with_servers(vec![]);
        let home = tempdir().unwrap();
        let _home_guard = lock_home_for_test(home.path());

        std::fs::write(
            home.path().join(".claude.json"),
            r#"{
  "mcpServers": {
    "github": { "command": "npx", "args": ["-y", "@mcp/github"], "type": "stdio" }
  }
}"#,
        )
        .unwrap();

        let added = import_from_provider("claude", project_dir.clone()).unwrap();
        assert_eq!(added, 1);
        let reloaded = crate::config::get_config(Some(project_dir)).unwrap();
        let server = reloaded
            .mcp_servers
            .iter()
            .find(|s| s.id == "github")
            .expect("global fallback server should be imported");
        assert_eq!(server.scope, "global");
    }

    #[test]
    fn import_from_claude_prefers_project_config_over_global() {
        let (tmp, project_dir) = project_with_servers(vec![]);
        let home = tempdir().unwrap();
        let _home_guard = lock_home_for_test(home.path());

        std::fs::write(
            home.path().join(".claude.json"),
            r#"{
  "mcpServers": {
    "global-only": { "command": "npx", "args": ["-y", "@mcp/global-only"], "type": "stdio" }
  }
}"#,
        )
        .unwrap();
        std::fs::write(
            tmp.path().join(".mcp.json"),
            r#"{
  "mcpServers": {
    "project-only": { "command": "npx", "args": ["-y", "@mcp/project-only"], "type": "stdio" }
  }
}"#,
        )
        .unwrap();

        let added = import_from_provider("claude", project_dir.clone()).unwrap();
        assert_eq!(added, 1);
        let reloaded = crate::config::get_config(Some(project_dir)).unwrap();
        assert!(reloaded.mcp_servers.iter().any(|s| s.id == "project-only"));
        assert!(
            !reloaded.mcp_servers.iter().any(|s| s.id == "global-only"),
            "project config should short-circuit global import"
        );
    }

    #[test]
    fn import_from_provider_skips_reserved_ship_and_invalid_entries() {
        let (tmp, project_dir) = project_with_servers(vec![]);
        std::fs::write(
            tmp.path().join(".mcp.json"),
            r#"{
  "mcpServers": {
    "ship": { "command": "ship", "args": ["mcp", "serve"], "type": "stdio" },
    "bad-stdio": { "type": "stdio" },
    "bad-http": { "type": "http" },
    "ok-http": { "type": "http", "url": "https://example.com/mcp" }
  }
}"#,
        )
        .unwrap();

        let added = import_from_provider("claude", project_dir.clone()).unwrap();
        assert_eq!(added, 1, "only valid, non-reserved entries should import");
        let reloaded = crate::config::get_config(Some(project_dir)).unwrap();
        assert!(reloaded.mcp_servers.iter().any(|s| s.id == "ok-http"));
        assert!(!reloaded.mcp_servers.iter().any(|s| s.id == "ship"));
        assert!(!reloaded.mcp_servers.iter().any(|s| s.id == "bad-stdio"));
        assert!(!reloaded.mcp_servers.iter().any(|s| s.id == "bad-http"));
    }

    #[test]
    fn import_from_codex_uses_global_fallback_when_project_config_missing() {
        let (_tmp, project_dir) = project_with_servers(vec![]);
        let home = tempdir().unwrap();
        let _home_guard = lock_home_for_test(home.path());

        let codex_dir = home.path().join(".codex");
        std::fs::create_dir_all(&codex_dir).unwrap();
        std::fs::write(
            codex_dir.join("config.toml"),
            r#"[mcp_servers.global-gh]
command = "npx"
args = ["-y", "@mcp/github"]
"#,
        )
        .unwrap();

        let added = import_from_provider("codex", project_dir.clone()).unwrap();
        assert_eq!(added, 1);
        let reloaded = crate::config::get_config(Some(project_dir)).unwrap();
        let server = reloaded
            .mcp_servers
            .iter()
            .find(|s| s.id == "global-gh")
            .expect("global codex fallback server should be imported");
        assert_eq!(server.scope, "global");
    }
}
