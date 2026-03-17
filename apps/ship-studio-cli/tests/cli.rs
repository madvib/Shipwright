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

fn ship() -> Command {
    Command::cargo_bin("ship").unwrap()
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
fn use_activates_profile_writes_claude_md_and_lock() {
    let tmp = TempDir::new().unwrap();

    ship().args(["init"]).current_dir(tmp.path()).assert().success();
    write(tmp.path(), ".ship/agents/rules/style.md", "Use explicit types.");
    write(
        tmp.path(),
        ".ship/agents/profiles/my-profile.toml",
        r#"[profile]
name = "My Profile"
id = "my-profile"
providers = ["claude"]
"#,
    );

    ship()
        .args(["use", "my-profile", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("activated profile 'my-profile'"));

    let lock = fs::read_to_string(tmp.path().join(".ship/ship.lock")).unwrap();
    assert!(
        lock.contains("active_profile") && lock.contains("my-profile"),
        "ship.lock must record active_profile"
    );
    assert!(tmp.path().join("CLAUDE.md").exists(), "CLAUDE.md must be written");
}

#[test]
fn use_same_profile_twice_is_idempotent() {
    let tmp = TempDir::new().unwrap();

    ship().args(["init"]).current_dir(tmp.path()).assert().success();
    write(tmp.path(), ".ship/agents/rules/style.md", "Use explicit types.");
    write(
        tmp.path(),
        ".ship/agents/profiles/my-profile.toml",
        r#"[profile]
name = "My Profile"
id = "my-profile"
providers = ["claude"]
"#,
    );

    let args = ["use", "my-profile", "--path", tmp.path().to_str().unwrap()];
    ship().args(args).assert().success();
    ship().args(args).assert().success();

    let lock = fs::read_to_string(tmp.path().join(".ship/ship.lock")).unwrap();
    assert!(lock.contains("my-profile"));
}

#[test]
fn use_fails_without_ship_dir() {
    let tmp = TempDir::new().unwrap();

    ship()
        .args(["use", "my-profile", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains(".ship/ not found"));
}

#[test]
fn use_fails_for_unknown_profile() {
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

// ── ship export ───────────────────────────────────────────────────────────────

#[test]
fn export_gemini_writes_gemini_md() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), ".ship/agents/rules/style.md", "Use explicit types.");

    ship()
        .args(["export", "gemini"])
        .current_dir(tmp.path())
        .assert()
        .success();

    assert!(tmp.path().join("GEMINI.md").exists(), "GEMINI.md must be written for gemini export");
    let content = fs::read_to_string(tmp.path().join("GEMINI.md")).unwrap();
    assert!(content.contains("Use explicit types."));
}

#[test]
fn export_codex_writes_agents_md() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), ".ship/agents/rules/style.md", "Use explicit types.");

    ship()
        .args(["export", "codex"])
        .current_dir(tmp.path())
        .assert()
        .success();

    assert!(tmp.path().join("AGENTS.md").exists(), "AGENTS.md must be written for codex export");
    let content = fs::read_to_string(tmp.path().join("AGENTS.md")).unwrap();
    assert!(content.contains("Use explicit types."));
}

#[test]
fn export_codex_writes_codex_config_toml() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), ".ship/agents/rules/style.md", "Use explicit types.");

    ship()
        .args(["export", "codex"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let path = tmp.path().join(".codex/config.toml");
    assert!(path.exists(), ".codex/config.toml must be written for codex export");
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("mcp_servers"), "config.toml must contain mcp_servers");
}

// ── auth ──────────────────────────────────────────────────────────────────────

#[test]
fn login_opens_browser_flow() {
    // login starts an OAuth flow — verify it prints the auth URL and exits
    // (test environment has no browser; the 60 s callback wait is not exercised here
    //  because the test binary does not trigger the server listen path)
    ship()
        .args(["login"])
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .stdout(predicate::str::contains("Opening browser"));
}

#[test]
fn logout_when_not_logged_in_exits_zero() {
    let tmp = TempDir::new().unwrap();

    ship()
        .args(["logout"])
        .env("HOME", tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Not logged in."));
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
        .stdout(predicate::str::contains("not logged in"));
}
