use crate::compile::{compile, CURSOR_PERMISSIVE_ALLOW};
use crate::resolve::ResolvedConfig;
use crate::types::{Permissions, Rule, ToolPermissions};

use super::fixtures::*;

// ── Cursor CLI permissions ────────────────────────────────────────────────────

/// Source: https://cursor.com/docs/cli/reference/permissions
/// Default permissions (allow=[*], deny=[]) must not emit any patch.
#[test]
fn cursor_cli_permissions_default_is_none() {
    let out = compile(&resolved(vec![]), "cursor").unwrap();
    assert!(out.cursor_cli_permissions.is_none());
}

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
    assert!(patch["permissions"].get("allow").is_none(), "allow=[*] must not emit allow field");
}

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
    assert!(allow.iter().any(|v| v == "Read(*)" ), "Read → Read(*)");
    assert!(allow.iter().any(|v| v == "Shell(git *)"), "Bash(git *) → Shell(git *)");
    assert!(allow.iter().any(|v| v == "Mcp(ship:*)"), "mcp__ship__* → Mcp(ship:*)");
}

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

#[test]
fn cursor_allow_star_never_auto_expands_to_permissive() {
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions { allow: vec!["*".to_string()], deny: vec![], ..Default::default() },
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "cursor").unwrap();
    assert!(out.cursor_cli_permissions.is_none());
}

#[test]
fn cursor_permissive_allow_constant_covers_all_types() {
    let types = ["Shell", "Read", "Write", "WebFetch", "Mcp"];
    for t in types {
        assert!(CURSOR_PERMISSIVE_ALLOW.iter().any(|p| p.starts_with(t)), "must include a {t}(*) entry");
    }
}

#[test]
fn cursor_cli_permissions_unknown_patterns_dropped() {
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions {
                allow: vec!["*".to_string()],
                deny: vec!["NotebookEdit".to_string()],
                ..Default::default()
            },
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "cursor").unwrap();
    assert!(out.cursor_cli_permissions.is_none());
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

// ── Cursor rule_files ─────────────────────────────────────────────────────────

#[test]
fn cursor_writes_per_file_mdc_rules() {
    let r = ResolvedConfig {
        rules: vec![make_rule("style.md", "Use consistent naming."), make_rule("workflow.md", "Run tests before commit.")],
        ..resolved(vec![])
    };
    let out = compile(&r, "cursor").unwrap();
    assert!(out.rule_files.contains_key(".cursor/rules/style.mdc"));
    assert!(out.rule_files.contains_key(".cursor/rules/workflow.mdc"));
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
        rules: vec![Rule { file_name: "conditional.md".into(), content: "Only when relevant.".into(), always_apply: false, globs: vec![], description: None }],
        ..resolved(vec![])
    };
    let out = compile(&r, "cursor").unwrap();
    let content = &out.rule_files[".cursor/rules/conditional.mdc"];
    assert!(content.contains("alwaysApply: false"));
}

#[test]
fn cursor_rule_globs_in_frontmatter() {
    let r = ResolvedConfig {
        rules: vec![Rule { file_name: "rust-only.md".into(), content: "Rust conventions.".into(), always_apply: false, globs: vec!["**/*.rs".into(), "Cargo.toml".into()], description: None }],
        ..resolved(vec![])
    };
    let out = compile(&r, "cursor").unwrap();
    let content = &out.rule_files[".cursor/rules/rust-only.mdc"];
    assert!(content.contains("globs:"));
    assert!(content.contains("**/*.rs"));
    assert!(content.contains("Cargo.toml"));
}

#[test]
fn cursor_rule_description_in_frontmatter() {
    let r = ResolvedConfig {
        rules: vec![Rule { file_name: "smart.md".into(), content: "Apply intelligently.".into(), always_apply: false, globs: vec![], description: Some("Apply when writing React components".into()) }],
        ..resolved(vec![])
    };
    let out = compile(&r, "cursor").unwrap();
    let content = &out.rule_files[".cursor/rules/smart.mdc"];
    assert!(content.contains("description:"));
    assert!(content.contains("Apply when writing React components"));
}

#[test]
fn cursor_no_context_content() {
    let r = ResolvedConfig { rules: vec![make_rule("style.md", "Use consistent naming.")], ..resolved(vec![]) };
    let out = compile(&r, "cursor").unwrap();
    assert!(out.context_content.is_none());
}

#[test]
fn cursor_empty_rule_content_skipped_in_rule_files() {
    let r = ResolvedConfig {
        rules: vec![make_rule("empty.md", "   "), make_rule("real.md", "Actual content here.")],
        ..resolved(vec![])
    };
    let out = compile(&r, "cursor").unwrap();
    assert!(!out.rule_files.contains_key(".cursor/rules/empty.mdc"));
    assert!(out.rule_files.contains_key(".cursor/rules/real.mdc"));
}

#[test]
fn other_providers_have_no_rule_files() {
    let r = ResolvedConfig { rules: vec![make_rule("style.md", "Use consistent naming.")], ..resolved(vec![]) };
    assert!(compile(&r, "claude").unwrap().rule_files.is_empty());
    assert!(compile(&r, "gemini").unwrap().rule_files.is_empty());
    assert!(compile(&r, "codex").unwrap().rule_files.is_empty());
}
