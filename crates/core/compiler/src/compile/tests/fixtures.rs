use crate::resolve::ResolvedConfig;
use crate::types::{
    HookConfig, HookTrigger, McpServerConfig, McpServerType, Permissions, Rule, Skill,
};

pub fn make_server(id: &str) -> McpServerConfig {
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

pub fn make_skill(id: &str) -> Skill {
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

pub fn make_hook(trigger: HookTrigger, command: &str, matcher: Option<&str>) -> HookConfig {
    HookConfig {
        id: "test-hook".to_string(),
        trigger,
        matcher: matcher.map(str::to_string),
        command: command.to_string(),
    }
}

pub fn make_rule(file_name: &str, content: &str) -> Rule {
    Rule {
        file_name: file_name.to_string(),
        content: content.to_string(),
        always_apply: true,
        globs: vec![],
        description: None,
    }
}

pub fn resolved(servers: Vec<McpServerConfig>) -> ResolvedConfig {
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
        active_agent: None,
        plugins: Default::default(),
        claude_settings_extra: None,
        agent_profiles: vec![],
        claude_team_agents: vec![],
        env: Default::default(),
        available_models: vec![],
        codex_sandbox: None,
    }
}
