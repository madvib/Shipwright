use super::*;
use std::collections::HashMap;
use std::fs;
use tempfile::tempdir;

#[test]
fn save_config_keeps_ship_toml_free_of_agent_sections() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let ship_dir = tmp.path().join(".ship");
    fs::create_dir_all(&ship_dir)?;

    let config = ProjectConfig {
        providers: vec!["claude".to_string(), "codex".to_string()],
        active_agent: Some("planning".to_string()),
        hooks: vec![HookConfig {
            id: "audit".to_string(),
            trigger: HookTrigger::PostToolUse,
            matcher: Some("Bash".to_string()),
            timeout_ms: None,
            description: None,
            command: "echo audit".to_string(),
        }],
        ai: Some(AiConfig {
            provider: Some("codex".to_string()),
            model: Some("gpt-5".to_string()),
            cli_path: None,
        }),
        agent: AgentLayerConfig {
            skills: vec!["task-policy".to_string()],
            prompts: vec![],
            context: vec!["project/README.md".to_string()],
        },
        modes: vec![AgentProfile {
            id: "planning".to_string(),
            name: "Planning".to_string(),
            ..Default::default()
        }],
        mcp_servers: vec![McpServerConfig {
            id: "github".to_string(),
            name: "GitHub".to_string(),
            command: "npx".to_string(),
            args: vec![
                "-y".to_string(),
                "@modelcontextprotocol/server-github".to_string(),
            ],
            env: HashMap::new(),
            scope: "project".to_string(),
            server_type: McpServerType::Stdio,
            url: None,
            disabled: false,
            timeout_secs: Some(30),
        }],
        ..Default::default()
    };

    save_config(&config, Some(ship_dir.clone()))?;

    let ship_config = fs::read_to_string(ship_dir.join(PRIMARY_CONFIG_FILE))?;
    assert!(
        !ship_config.contains("modes"),
        "ship.jsonc must not persist mode definitions"
    );
    assert!(
        !ship_config.contains("mcp_servers"),
        "ship.jsonc must not persist MCP servers"
    );
    assert!(
        !ship_config.contains("\"agent\""),
        "ship.jsonc must not persist agent block"
    );
    assert!(
        !ship_config.contains("\"providers\""),
        "ship.jsonc must not persist providers"
    );
    assert!(
        !ship_config.contains("active_agent"),
        "ship.jsonc must not persist active_agent"
    );
    assert!(
        !ship_config.contains("statuses"),
        "ship.jsonc must not persist statuses"
    );
    assert!(
        !ship_config.contains("\"git\""),
        "ship.jsonc must not persist git policy"
    );
    assert!(
        !ship_config.contains("\"ai\""),
        "ship.jsonc must not persist ai block"
    );
    assert!(
        !ship_config.contains("namespaces"),
        "ship.jsonc must not persist namespace claims"
    );

    assert!(
        !ship_dir.join("agents").join("config.toml").exists(),
        "legacy agents/config.toml should not be written"
    );

    // Verify runtime settings stored in kv_state
    let providers: Vec<String> = crate::db::kv::get("runtime", "providers")?
        .map(|v| serde_json::from_value(v).unwrap())
        .unwrap_or_default();
    assert_eq!(providers, vec!["claude".to_string(), "codex".to_string()]);

    let active_agent: Option<String> = crate::db::kv::get("runtime", "active_agent")?
        .and_then(|v| serde_json::from_value(v).ok());
    assert_eq!(active_agent.as_deref(), Some("planning"));

    let hooks_json = crate::db::kv::get("runtime", "hooks")?.unwrap().to_string();
    assert!(hooks_json.contains("\"audit\""));

    let statuses_json = crate::db::kv::get("runtime", "statuses")?.unwrap().to_string();
    assert!(statuses_json.contains("\"backlog\""));

    let ai_json = crate::db::kv::get("runtime", "ai")?.unwrap().to_string();
    assert!(ai_json.contains("\"codex\""));

    let git_json = crate::db::kv::get("runtime", "git")?.unwrap().to_string();
    assert!(git_json.contains("\"ship.jsonc\""));

    let ns_json = crate::db::kv::get("runtime", "namespaces")?.unwrap().to_string();
    assert!(ns_json.contains("\"project\""));

    // Verify modes stored in kv_state
    let modes_val = crate::db::kv::get("runtime", "modes")?.expect("expected modes in kv");
    let modes: Vec<serde_json::Value> = serde_json::from_value(modes_val)?;
    assert_eq!(modes.len(), 1);
    assert_eq!(modes[0]["id"], "planning");

    let mcp_cfg = fs::read_to_string(ship_dir.join("mcp.jsonc"))?;
    assert!(mcp_cfg.contains("\"github\""));
    Ok(())
}

#[test]
fn get_config_round_trips_agent_sidecars() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let ship_dir = tmp.path().join(".ship");
    fs::create_dir_all(&ship_dir)?;

    let config = ProjectConfig {
        providers: vec!["gemini".to_string()],
        active_agent: Some("focus".to_string()),
        ai: Some(AiConfig {
            provider: Some("codex".to_string()),
            model: Some("gpt-5".to_string()),
            cli_path: Some("codex".to_string()),
        }),
        statuses: vec![StatusConfig {
            id: "qa".to_string(),
            name: "QA".to_string(),
            color: "teal".to_string(),
        }],
        git: GitConfig {
            ignore: vec!["project/features".to_string()],
            commit: vec!["ship.toml".to_string(), "rules".to_string()],
        },
        namespaces: vec![NamespaceConfig {
            id: "plugin:demo".to_string(),
            path: "demo".to_string(),
            owner: "plugins".to_string(),
        }],
        modes: vec![AgentProfile {
            id: "focus".to_string(),
            name: "Focus".to_string(),
            mcp_servers: vec!["github".to_string()],
            ..Default::default()
        }],
        mcp_servers: vec![McpServerConfig {
            id: "github".to_string(),
            name: "GitHub".to_string(),
            command: "npx".to_string(),
            args: vec![],
            env: HashMap::new(),
            scope: "project".to_string(),
            server_type: McpServerType::Stdio,
            url: None,
            disabled: false,
            timeout_secs: None,
        }],
        ..Default::default()
    };

    save_config(&config, Some(ship_dir.clone()))?;
    let loaded = get_config(Some(ship_dir))?;

    assert_eq!(loaded.providers, vec!["gemini".to_string()]);
    assert_eq!(loaded.active_agent.as_deref(), Some("focus"));
    assert_eq!(
        loaded
            .ai
            .as_ref()
            .and_then(|ai| ai.provider.clone())
            .as_deref(),
        Some("codex")
    );
    assert!(loaded.statuses.iter().any(|status| status.id == "qa"));
    assert!(loaded.git.commit.iter().any(|entry| entry == "rules"));
    assert!(loaded.namespaces.iter().any(|ns| ns.id == "plugin:demo"));
    assert!(loaded.agent.skills.is_empty());
    assert_eq!(loaded.modes.len(), 1);
    assert_eq!(loaded.modes[0].id, "focus");
    assert_eq!(loaded.mcp_servers.len(), 1);
    assert_eq!(loaded.mcp_servers[0].id, "github");
    Ok(())
}
