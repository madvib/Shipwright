//! Tests for `--with` extra skill injection in `run_compile`.

use super::*;
use std::path::Path;
use tempfile::TempDir;

fn write(dir: &Path, rel: &str, content: &str) {
    let p = dir.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn write_agent_with_skill_filter(dir: &Path) {
    write(
        dir,
        ".ship/agents/rust-lane.toml",
        r#"
[agent]
name = "Rust Lane"
id = "rust-lane"
providers = ["claude"]

[skills]
refs = ["base-skill"]
"#,
    );
    write(
        dir,
        ".ship/skills/base-skill/SKILL.md",
        "---\nname: base-skill\n---\nBase skill content.",
    );
}

/// `--with compiler-docs` adds the injected skill to output when agent has refs filter.
#[test]
fn extra_skill_included_in_output() {
    let tmp = TempDir::new().unwrap();
    write_agent_with_skill_filter(tmp.path());
    write(
        tmp.path(),
        ".ship/skills/compiler-docs/SKILL.md",
        "---\nname: compiler-docs\n---\nCompiler docs content.",
    );
    run_compile(CompileOptions {
        project_root: tmp.path(),
        output_root: None,
        provider: Some("claude"),
        dry_run: false,
        active_agent: Some("rust-lane"),
        extra_skills: vec!["compiler-docs".to_string()],
    })
    .unwrap();
    let injected =
        std::fs::read_to_string(tmp.path().join(".claude/skills/compiler-docs/SKILL.md"))
            .expect("injected skill file must exist");
    assert!(
        injected.contains("Compiler docs content."),
        "injected skill must appear in output: {injected}"
    );
    let base =
        std::fs::read_to_string(tmp.path().join(".claude/skills/base-skill/SKILL.md"))
            .expect("base skill file must exist");
    assert!(
        base.contains("Base skill content."),
        "base skill must still appear in output: {base}"
    );
}

/// `--with nonexistent` returns a clear error with the skill ID.
#[test]
fn nonexistent_extra_skill_returns_clear_error() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        ".ship/agents/rust-lane.toml",
        r#"
[agent]
name = "Rust Lane"
id = "rust-lane"
providers = ["claude"]
"#,
    );
    let err = run_compile(CompileOptions {
        project_root: tmp.path(),
        output_root: None,
        provider: Some("claude"),
        dry_run: false,
        active_agent: Some("rust-lane"),
        extra_skills: vec!["nonexistent".to_string()],
    })
    .unwrap_err();
    assert!(
        err.to_string().contains("skill not found: nonexistent"),
        "error message must identify the missing skill: {err}"
    );
}

/// `--with a --with b` includes both injected skills alongside the base skill.
#[test]
fn multiple_extra_skills_all_included() {
    let tmp = TempDir::new().unwrap();
    write_agent_with_skill_filter(tmp.path());
    write(
        tmp.path(),
        ".ship/skills/auth-feature/SKILL.md",
        "---\nname: auth-feature\n---\nAuth feature content.",
    );
    write(
        tmp.path(),
        ".ship/skills/tdd/SKILL.md",
        "---\nname: tdd\n---\nTDD content.",
    );
    run_compile(CompileOptions {
        project_root: tmp.path(),
        output_root: None,
        provider: Some("claude"),
        dry_run: false,
        active_agent: Some("rust-lane"),
        extra_skills: vec!["auth-feature".to_string(), "tdd".to_string()],
    })
    .unwrap();
    let auth =
        std::fs::read_to_string(tmp.path().join(".claude/skills/auth-feature/SKILL.md"))
            .expect("auth-feature skill file must exist");
    assert!(auth.contains("Auth feature content."), "auth-feature must appear: {auth}");
    let tdd =
        std::fs::read_to_string(tmp.path().join(".claude/skills/tdd/SKILL.md"))
            .expect("tdd skill file must exist");
    assert!(tdd.contains("TDD content."), "tdd must appear: {tdd}");
    let base =
        std::fs::read_to_string(tmp.path().join(".claude/skills/base-skill/SKILL.md"))
            .expect("base skill file must exist");
    assert!(base.contains("Base skill content."), "base skill must still appear: {base}");
}

/// No `--with` flags: behaviour is identical to before. Non-referenced skill excluded.
#[test]
fn no_extra_skills_behaviour_unchanged() {
    let tmp = TempDir::new().unwrap();
    write_agent_with_skill_filter(tmp.path());
    write(
        tmp.path(),
        ".ship/skills/other/SKILL.md",
        "---\nname: other\n---\nOther content.",
    );
    run_compile(CompileOptions {
        project_root: tmp.path(),
        output_root: None,
        provider: Some("claude"),
        dry_run: false,
        active_agent: Some("rust-lane"),
        extra_skills: vec![],
    })
    .unwrap();
    let base =
        std::fs::read_to_string(tmp.path().join(".claude/skills/base-skill/SKILL.md"))
            .expect("base skill file must exist");
    assert!(base.contains("Base skill content."), "base skill must appear: {base}");
    assert!(
        !tmp.path().join(".claude/skills/other/SKILL.md").exists(),
        "non-referenced skill must not be written when refs filter is active"
    );
}
