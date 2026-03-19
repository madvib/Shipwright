use crate::compile::{compile, get_provider};
use crate::resolve::ResolvedConfig;
use crate::types::{McpServerType, Rule};

use super::fixtures::*;

// ── MCP server output correctness ─────────────────────────────────────────────

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
    assert!(out.mcp_servers.is_object());
    assert!(out.mcp_servers.as_object().unwrap().contains_key("x"));
}

#[test]
fn codex_uses_mcp_servers_key() {
    let desc = get_provider("codex").unwrap();
    assert_eq!(desc.mcp_key.as_str(), "mcp_servers");
}

/// Source: https://geminicli.com/docs/tools/mcp-server
/// Gemini has NO "type" field — transport is inferred from which property is present.
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
    assert!(out.mcp_servers["sse-server"].get("httpUrl").is_none());
}

#[test]
fn timeout_secs_maps_to_startup_timeout_sec() {
    let mut s = make_server("slow");
    s.timeout_secs = Some(30);
    let out = compile(&resolved(vec![s]), "claude").unwrap();
    assert_eq!(out.mcp_servers["slow"]["startup_timeout_sec"], 30);
}

#[test]
fn claude_mcp_config_path_in_output() {
    let r = resolved(vec![]);
    let out = compile(&r, "claude").unwrap();
    assert_eq!(out.mcp_config_path.as_deref(), Some(".mcp.json"));
}

// ── Context file (CLAUDE.md / AGENTS.md / GEMINI.md) ─────────────────────────

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
    assert!(out.context_content.is_none());
}

// ── Active mode in context ───────────────────────────────────────────────────

#[test]
fn active_mode_appears_in_context_content() {
    let r = ResolvedConfig {
        rules: vec![make_rule("style.md", "Be concise.")],
        active_agent: Some("planning".to_string()),
        ..resolved(vec![])
    };
    let out = compile(&r, "claude").unwrap();
    let content = out.context_content.expect("must have context");
    assert!(content.contains("planning"), "context must include active mode notice");
}

#[test]
fn no_active_mode_no_mode_notice() {
    let r = ResolvedConfig {
        rules: vec![make_rule("style.md", "Be concise.")],
        active_agent: None,
        ..resolved(vec![])
    };
    let out = compile(&r, "claude").unwrap();
    let content = out.context_content.expect("must have context");
    assert!(!content.contains("active mode"));
}

// ── Skill files ───────────────────────────────────────────────────────────────

#[test]
fn empty_skill_content_is_filtered() {
    let mut stub = make_skill("git-commit");
    stub.content = String::new();
    let mut real = make_skill("code-review");
    real.content = "## Instructions\nReview the diff carefully.".to_string();
    let r = ResolvedConfig { skills: vec![stub, real], ..resolved(vec![]) };
    let out = compile(&r, "claude").unwrap();
    assert!(!out.skill_files.contains_key(".claude/skills/git-commit/SKILL.md"));
    assert!(out.skill_files.contains_key(".claude/skills/code-review/SKILL.md"));
}

#[test]
fn skill_files_provider_directories() {
    let r = ResolvedConfig { skills: vec![make_skill("rust-expert")], ..resolved(vec![]) };
    assert!(compile(&r, "claude").unwrap().skill_files.contains_key(".claude/skills/rust-expert/SKILL.md"));
    assert!(compile(&r, "gemini").unwrap().skill_files.contains_key(".agents/skills/rust-expert/SKILL.md"));
    assert!(compile(&r, "codex").unwrap().skill_files.contains_key(".agents/skills/rust-expert/SKILL.md"));
}

#[test]
fn skill_file_has_yaml_frontmatter_and_content() {
    let r = ResolvedConfig { skills: vec![make_skill("my-skill")], ..resolved(vec![]) };
    let out = compile(&r, "claude").unwrap();
    let content = &out.skill_files[".claude/skills/my-skill/SKILL.md"];
    assert!(content.starts_with("---\n"), "skill file must start with YAML frontmatter");
    assert!(content.contains("name: my-skill"));
    assert!(content.contains("# my-skill"));
}
