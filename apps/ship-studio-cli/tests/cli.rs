//! End-to-end tests for the `ship` CLI binary.
//!
//! Each test runs the actual binary against an isolated temp directory.
//! No network calls are made — auth commands are stubs, compile operates
//! on local fixture files only.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Bare CLI command — no DB isolation. Use `ship_in()` for tests that touch
/// the database (ship use, ship status, etc.).
fn ship() -> Command {
    Command::cargo_bin("ship").unwrap()
}

/// CLI command with DB isolated to `tmp`'s temp dir.
/// All calls within the same test share state; different tests are isolated.
fn ship_in(tmp: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("ship").unwrap();
    cmd.env("SHIP_GLOBAL_DIR", tmp.path());
    cmd
}

fn write(dir: &Path, rel: &str, content: &str) {
    let p = dir.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, content).unwrap();
}

// ── ship init ─────────────────────────────────────────────────────────────────

#[test]
fn init_creates_ship_structure() {
    let tmp = TempDir::new().unwrap();

    ship()
        .args(["init"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("initialized .ship/"));

    assert!(tmp.path().join(".ship").is_dir());
    assert!(tmp.path().join(".ship/ship.toml").exists());
    assert!(tmp.path().join(".ship/README.md").exists());
    assert!(tmp.path().join(".ship/.gitignore").exists());
}

#[test]
fn init_is_idempotent() {
    let tmp = TempDir::new().unwrap();

    ship().args(["init"]).current_dir(tmp.path()).assert().success();

    let original = fs::read_to_string(tmp.path().join(".ship/ship.toml")).unwrap();

    // Second run must succeed without overwriting existing files.
    ship().args(["init"]).current_dir(tmp.path()).assert().success();

    let after = fs::read_to_string(tmp.path().join(".ship/ship.toml")).unwrap();
    assert_eq!(original, after, "init must not overwrite existing ship.toml");
}

#[test]
fn init_with_provider_sets_provider_in_toml() {
    let tmp = TempDir::new().unwrap();

    ship()
        .args(["init", "--provider", "gemini"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let toml = fs::read_to_string(tmp.path().join(".ship/ship.toml")).unwrap();
    assert!(toml.contains("gemini"), "provider should appear in ship.toml");
}

// ── ship use ──────────────────────────────────────────────────────────────────

#[test]
fn use_activates_agent_writes_claude_md_and_state() {
    let tmp = TempDir::new().unwrap();

    ship_in(&tmp).args(["init"]).current_dir(tmp.path()).assert().success();
    write(tmp.path(), ".ship/agents/rules/style.md", "Use explicit types.");
    write(
        tmp.path(),
        ".ship/agents/my-agent.toml",
        r#"[agent]
name = "My Agent"
id = "my-agent"
providers = ["claude"]
"#,
    );

    ship_in(&tmp)
        .args(["use", "my-agent", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("activated agent 'my-agent'"));

    // State is persisted to platform.db (at ~/.ship/). Verify via ship status.
    ship_in(&tmp)
        .args(["status", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("my-agent"));
    assert!(tmp.path().join("CLAUDE.md").exists(), "CLAUDE.md must be written");
}

#[test]
fn use_same_agent_twice_is_idempotent() {
    let tmp = TempDir::new().unwrap();

    ship_in(&tmp).args(["init"]).current_dir(tmp.path()).assert().success();
    write(tmp.path(), ".ship/agents/rules/style.md", "Use explicit types.");
    write(
        tmp.path(),
        ".ship/agents/my-agent.toml",
        r#"[agent]
name = "My Agent"
id = "my-agent"
providers = ["claude"]
"#,
    );

    let args = ["use", "my-agent", "--path", tmp.path().to_str().unwrap()];
    ship_in(&tmp).args(args).assert().success();
    ship_in(&tmp).args(args).assert().success();

    ship_in(&tmp)
        .args(["status", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("my-agent"));
}

#[test]
fn use_fails_without_ship_dir() {
    let tmp = TempDir::new().unwrap();

    ship_in(&tmp)
        .args(["use", "my-profile", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains(".ship/ not found"));
}

#[test]
fn use_fails_for_unknown_agent() {
    let tmp = TempDir::new().unwrap();

    ship().args(["init"]).current_dir(tmp.path()).assert().success();

    ship()
        .args(["use", "nonexistent", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent"));
}

// ── ship compile ──────────────────────────────────────────────────────────────

#[test]
fn compile_writes_claude_md() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), ".ship/agents/rules/style.md", "Use explicit types.");

    ship()
        .args(["compile", "--provider", "claude", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
    assert!(content.contains("Use explicit types."));
}

#[test]
fn compile_dry_run_writes_nothing() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), ".ship/agents/rules/style.md", "Use explicit types.");

    ship()
        .args([
            "compile",
            "--provider",
            "claude",
            "--dry-run",
            "--path",
            tmp.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("[dry-run]"));

    assert!(!tmp.path().join("CLAUDE.md").exists(), "dry-run must not write any files");
}

#[test]
fn compile_fails_without_ship_dir() {
    let tmp = TempDir::new().unwrap();

    ship()
        .args(["compile", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains(".ship/ not found"));
}

// ── ship skill list ───────────────────────────────────────────────────────────

#[test]
fn skill_list_no_skills_installed() {
    let tmp = TempDir::new().unwrap();

    // Isolate HOME so no real global skills are picked up.
    ship()
        .args(["skill", "list"])
        .current_dir(tmp.path())
        .env("HOME", tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No skills installed."));
}

#[test]
fn skill_list_shows_project_skill() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        ".ship/agents/skills/rust-expert/SKILL.md",
        "---\nname: Rust Expert\ndescription: Rust coding\n---\n\nBe a Rust expert.",
    );

    ship()
        .args(["skill", "list"])
        .current_dir(tmp.path())
        .env("HOME", tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("rust-expert"))
        .stdout(predicate::str::contains("Project skills"));
}

#[test]
fn skill_list_shows_global_skill() {
    let tmp = TempDir::new().unwrap();
    // Global skills live at $HOME/.ship/skills/<id>/SKILL.md
    write(
        tmp.path(),
        ".ship/skills/go-expert/SKILL.md",
        "---\nname: Go Expert\n---\n\nBe a Go expert.",
    );

    ship()
        .args(["skill", "list"])
        .current_dir(tmp.path())
        .env("HOME", tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("go-expert"))
        .stdout(predicate::str::contains("Global skills"));
}

// ── ship compile --provider ───────────────────────────────────────────────────

#[test]
fn compile_provider_gemini_writes_gemini_md() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), ".ship/agents/rules/style.md", "Use explicit types.");

    ship()
        .args(["compile", "--provider", "gemini"])
        .current_dir(tmp.path())
        .assert()
        .success();

    assert!(tmp.path().join("GEMINI.md").exists(), "GEMINI.md must be written for gemini compile");
    let content = fs::read_to_string(tmp.path().join("GEMINI.md")).unwrap();
    assert!(content.contains("Use explicit types."));
}

#[test]
fn compile_provider_codex_writes_agents_md() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), ".ship/agents/rules/style.md", "Use explicit types.");

    ship()
        .args(["compile", "--provider", "codex"])
        .current_dir(tmp.path())
        .assert()
        .success();

    assert!(tmp.path().join("AGENTS.md").exists(), "AGENTS.md must be written for codex compile");
    let content = fs::read_to_string(tmp.path().join("AGENTS.md")).unwrap();
    assert!(content.contains("Use explicit types."));
}

#[test]
fn compile_provider_codex_writes_codex_config_toml() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), ".ship/agents/rules/style.md", "Use explicit types.");

    ship()
        .args(["compile", "--provider", "codex"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let path = tmp.path().join(".codex/config.toml");
    assert!(path.exists(), ".codex/config.toml must be written for codex compile");
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("mcp_servers"), "config.toml must contain mcp_servers");
}

// ── ship status ───────────────────────────────────────────────────────────────

#[test]
fn status_shows_no_active_agent_before_use() {
    let tmp = TempDir::new().unwrap();

    ship_in(&tmp).args(["init"]).current_dir(tmp.path()).assert().success();

    ship_in(&tmp)
        .args(["status", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("No active agent"));
}

// ── ship compile idempotency ───────────────────────────────────────────────────

#[test]
fn compile_is_idempotent() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), ".ship/agents/rules/style.md", "Use explicit types.");

    let args = [
        "compile",
        "--provider",
        "claude",
        "--path",
        tmp.path().to_str().unwrap(),
    ];

    ship().args(args).assert().success();
    let first = fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();

    ship().args(args).assert().success();
    let second = fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();

    assert_eq!(first, second, "compile output must be identical on second run");
}

// ── ship install ──────────────────────────────────────────────────────────────

#[test]
fn install_no_manifest_exits_nonzero() {
    let tmp = TempDir::new().unwrap();
    ship().args(["init"]).current_dir(tmp.path()).assert().success();

    // .ship/ship.toml from `ship init` is a project-config file (no [module]),
    // so install must fail with a clear error.
    ship()
        .args(["install"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("[module]").or(predicate::str::contains("registry manifest")));
}

#[test]
fn install_empty_deps_writes_lock() {
    let tmp = TempDir::new().unwrap();
    // Write a minimal registry manifest with no dependencies.
    write(
        tmp.path(),
        ".ship/ship.toml",
        "[module]\nname = \"github.com/test/repo\"\nversion = \"0.1.0\"\n",
    );

    // install with no deps needs no network; lock is written immediately.
    ship()
        .args(["install"])
        .current_dir(tmp.path())
        .assert()
        .success();

    assert!(
        tmp.path().join(".ship/ship.lock").exists(),
        ".ship/ship.lock must be created by ship install"
    );
}

// ── ship validate ─────────────────────────────────────────────────────────────

#[test]
fn validate_passes_on_valid_config() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        ".ship/agents/permissions.toml",
        "[ship-standard]\ndefault_mode = \"acceptEdits\"\n",
    );
    write(
        tmp.path(),
        ".ship/agents/default.toml",
        r#"[agent]
name = "Default"
id = "default"
providers = ["claude"]
[skills]
refs = []
[mcp]
servers = []
[permissions]
preset = "ship-standard"
"#,
    );

    ship()
        .args(["validate", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("valid"));
}

#[test]
fn validate_reports_error_on_bad_toml() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), ".ship/agents/bad.toml", "not valid toml [[");

    ship()
        .args(["validate", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .failure()
        .stdout(predicate::str::contains("TOML parse error").or(predicate::str::contains("✗")));
}

// ── auth ──────────────────────────────────────────────────────────────────────

#[test]
fn logout_when_not_logged_in() {
    let tmp = TempDir::new().unwrap();

    ship()
        .args(["logout"])
        .env("HOME", tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Not logged in"));
}

#[test]
fn whoami_not_logged_in_when_no_config() {
    let tmp = TempDir::new().unwrap();

    // Isolate HOME so no real ~/.ship/config.toml is read.
    ship()
        .args(["whoami"])
        .env("HOME", tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Not logged in"));
}
