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
        codex_enabled_tools: vec![],
        codex_disabled_tools: vec![],
        gemini_trust: None,
        gemini_include_tools: vec![],
        gemini_exclude_tools: vec![],
        gemini_timeout_ms: None,
        cursor_env_file: None,
    }
}

pub fn make_skill(id: &str) -> Skill {
    Skill {
        id: id.to_string(),
        name: id.to_string(),
        stable_id: None,
        description: Some(format!("{} skill", id)),
        license: None,
        compatibility: None,
        allowed_tools: vec![],
        metadata: Default::default(),
        content: format!("# {}\n\nDo the thing.", id),
        source: Default::default(),
        vars: Default::default(),
        artifacts: vec![],
    }
}

pub fn make_hook(trigger: HookTrigger, command: &str, matcher: Option<&str>) -> HookConfig {
    HookConfig {
        id: "test-hook".to_string(),
        trigger,
        matcher: matcher.map(str::to_string),
        command: command.to_string(),
        cursor_event: None,
        gemini_event: None,
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
        gemini_default_approval_mode: None,
        gemini_max_session_turns: None,
        gemini_disable_yolo_mode: None,
        gemini_disable_always_allow: None,
        gemini_tools_sandbox: None,
        gemini_settings_extra: None,
        codex_approval_policy: None,
        codex_reasoning_effort: None,
        codex_max_threads: None,
        codex_max_depth: None,
        codex_job_max_runtime_seconds: None,
        codex_shell_env_policy: None,
        codex_notify: None,
        codex_settings_extra: None,
        opencode_settings_extra: None,
        cursor_environment: None,
        cursor_settings_extra: None,
        claude_theme: None,
        claude_auto_updates: None,
        claude_include_co_authored_by: None,
        studio_mcp_url: None,
    }
}
