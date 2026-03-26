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
    assert!(tmp.path().join(".ship/ship.jsonc").exists());
    assert!(tmp.path().join(".ship/README.md").exists());
    assert!(tmp.path().join(".ship/.gitignore").exists());
}

#[test]
fn init_is_idempotent() {
    let tmp = TempDir::new().unwrap();

    ship()
        .args(["init"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let original = fs::read_to_string(tmp.path().join(".ship/ship.jsonc")).unwrap();

    // Second run must succeed without overwriting existing files.
    ship()
        .args(["init"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let after = fs::read_to_string(tmp.path().join(".ship/ship.jsonc")).unwrap();
    assert_eq!(
        original, after,
        "init must not overwrite existing ship.jsonc"
    );
}

#[test]
fn init_with_provider_sets_provider_in_toml() {
    let tmp = TempDir::new().unwrap();

    ship()
        .args(["init", "--provider", "gemini"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join(".ship/ship.jsonc")).unwrap();
    assert!(
        content.contains("gemini"),
        "provider should appear in ship.jsonc"
    );
}

// ── ship use ──────────────────────────────────────────────────────────────────

#[test]
fn use_activates_agent_writes_claude_md_and_state() {
    let tmp = TempDir::new().unwrap();

    ship_in(&tmp)
        .args(["init"])
        .current_dir(tmp.path())
        .assert()
        .success();
    write(tmp.path(), ".ship/rules/style.md", "Use explicit types.");
    write(
        tmp.path(),
        ".ship/agents/my-agent.jsonc",
        r#"{ "agent": { "name": "My Agent", "id": "my-agent", "providers": ["claude"] } }"#,
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
    assert!(
        tmp.path().join("CLAUDE.md").exists(),
        "CLAUDE.md must be written"
    );
}

#[test]
fn use_same_agent_twice_is_idempotent() {
    let tmp = TempDir::new().unwrap();

    ship_in(&tmp)
        .args(["init"])
        .current_dir(tmp.path())
        .assert()
        .success();
    write(tmp.path(), ".ship/rules/style.md", "Use explicit types.");
    write(
        tmp.path(),
        ".ship/agents/my-agent.jsonc",
        r#"{ "agent": { "name": "My Agent", "id": "my-agent", "providers": ["claude"] } }"#,
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

    ship()
        .args(["init"])
        .current_dir(tmp.path())
        .assert()
        .success();

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
    write(tmp.path(), ".ship/rules/style.md", "Use explicit types.");

    ship()
        .args([
            "compile",
            "--provider",
            "claude",
            "--path",
            tmp.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
    assert!(content.contains("Use explicit types."));
}

#[test]
fn compile_dry_run_writes_nothing() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), ".ship/rules/style.md", "Use explicit types.");

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

    assert!(
        !tmp.path().join("CLAUDE.md").exists(),
        "dry-run must not write any files"
    );
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
        .args(["skills", "list"])
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
        ".ship/skills/rust-expert/SKILL.md",
        "---\nname: Rust Expert\ndescription: Rust coding\n---\n\nBe a Rust expert.",
    );

    ship()
        .args(["skills", "list"])
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
        .args(["skills", "list"])
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
    write(tmp.path(), ".ship/rules/style.md", "Use explicit types.");

    ship()
        .args(["compile", "--provider", "gemini"])
        .current_dir(tmp.path())
        .assert()
        .success();

    assert!(
        tmp.path().join("GEMINI.md").exists(),
        "GEMINI.md must be written for gemini compile"
    );
    let content = fs::read_to_string(tmp.path().join("GEMINI.md")).unwrap();
    assert!(content.contains("Use explicit types."));
}

#[test]
fn compile_provider_codex_writes_agents_md() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), ".ship/rules/style.md", "Use explicit types.");

    ship()
        .args(["compile", "--provider", "codex"])
        .current_dir(tmp.path())
        .assert()
        .success();

    assert!(
        tmp.path().join("AGENTS.md").exists(),
        "AGENTS.md must be written for codex compile"
    );
    let content = fs::read_to_string(tmp.path().join("AGENTS.md")).unwrap();
    assert!(content.contains("Use explicit types."));
}

#[test]
fn compile_provider_codex_writes_codex_config_toml() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), ".ship/rules/style.md", "Use explicit types.");

    ship()
        .args(["compile", "--provider", "codex"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let path = tmp.path().join(".codex/config.toml");
    assert!(
        path.exists(),
        ".codex/config.toml must be written for codex compile"
    );
    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("mcp_servers"),
        "config.toml must contain mcp_servers"
    );
}

// ── ship status ───────────────────────────────────────────────────────────────

#[test]
fn status_shows_no_active_agent_before_use() {
    let tmp = TempDir::new().unwrap();

    ship_in(&tmp)
        .args(["init"])
        .current_dir(tmp.path())
        .assert()
        .success();

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
    write(tmp.path(), ".ship/rules/style.md", "Use explicit types.");

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

    assert_eq!(
        first, second,
        "compile output must be identical on second run"
    );
}

// ── ship install ──────────────────────────────────────────────────────────────

#[test]
fn install_no_manifest_exits_nonzero() {
    let tmp = TempDir::new().unwrap();
    ship()
        .args(["init"])
        .current_dir(tmp.path())
        .assert()
        .success();

    // .ship/ship.jsonc from `ship init` is a project-config file (no "module"),
    // so install must fail with a clear error.
    ship()
        .args(["install"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("module").or(predicate::str::contains("manifest")));
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
        ".ship/permissions.jsonc",
        r#"{ "ship-standard": { "default_mode": "acceptEdits" } }"#,
    );
    write(
        tmp.path(),
        ".ship/agents/default.jsonc",
        r#"{
  "agent": { "name": "Default", "id": "default", "providers": ["claude"] },
  "skills": { "refs": [] },
  "mcp": { "servers": [] },
  "permissions": { "preset": "ship-standard" }
}"#,
    );

    ship()
        .args(["validate", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("valid"));
}

#[test]
fn validate_reports_error_on_bad_config() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), ".ship/agents/bad.jsonc", "not valid json {{{");

    ship()
        .args(["validate", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .failure()
        .stdout(predicate::str::contains("✗"));
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

// ── E2E integration tests ────────────────────────────────────────────────────

/// Full user story: init -> write agent JSONC with permissions/skills/hooks ->
/// write rules -> write MCP config -> write permissions preset -> use agent ->
/// compile for claude -> verify all output files.
#[test]
fn e2e_init_create_agent_use_compile_full_flow() {
    let tmp = TempDir::new().unwrap();
    let p = tmp.path().to_str().unwrap();

    // 1. ship init
    ship()
        .args(["init"])
        .current_dir(tmp.path())
        .assert()
        .success();

    // 2. Write a realistic agent JSONC with permissions preset, skills ref, hooks
    write(
        tmp.path(),
        ".ship/agents/fullstack.jsonc",
        r#"{
  "agent": {
    "id": "fullstack",
    "name": "Fullstack Dev",
    "version": "0.1.0",
    "description": "Full-stack development agent",
    "providers": ["claude"]
  },
  "skills": { "refs": [] },
  "mcp": { "servers": [] },
  "permissions": {
    "preset": "ship-autonomous",
    "tools_deny": ["Bash(docker rm*)"]
  },
  "rules": {
    "inline": "Always write tests before implementation."
  },
  "hooks": {
    "stop": "ship permissions sync"
  }
}"#,
    );

    // 3. Write a rule file
    write(
        tmp.path(),
        ".ship/rules/style.md",
        "Prefer composition over inheritance. Use explicit return types.",
    );

    // 4. Write MCP config with a server
    write(
        tmp.path(),
        ".ship/mcp.jsonc",
        r#"{
  "mcp": {
    "servers": {
      "github": {
        "id": "",
        "name": "GitHub MCP",
        "command": "npx",
        "args": ["-y", "@modelcontextprotocol/server-github"],
        "scope": "project",
        "server_type": "stdio",
        "disabled": false,
        "env": {}
      }
    }
  }
}"#,
    );

    // 5. Write permissions.jsonc with ship-autonomous preset
    write(
        tmp.path(),
        ".ship/permissions.jsonc",
        r#"{
  "ship-autonomous": {
    "default_mode": "dontAsk",
    "tools_allow": ["Read", "Write", "Edit", "Glob", "Grep", "Bash(*)"],
    "tools_deny": [
      "Bash(rm -rf *)",
      "Bash(git reset --hard*)",
      "Bash(git push*)"
    ]
  }
}"#,
    );

    // 6. ship use <agent> (this compiles with the agent active)
    ship_in(&tmp)
        .args(["use", "fullstack", "--path", p])
        .assert()
        .success()
        .stdout(predicate::str::contains("activated agent 'fullstack'"));

    // 7. Verify CLAUDE.md contains rule content and inline rule (written by ship use)
    let claude_md = fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
    assert!(
        claude_md.contains("Prefer composition over inheritance"),
        "CLAUDE.md must contain rule file content"
    );
    assert!(
        claude_md.contains("Always write tests before implementation"),
        "CLAUDE.md must contain inline rule from agent"
    );

    // 8. Re-compile picks up active agent from DB state
    ship_in(&tmp)
        .args(["compile", "--provider", "claude", "--path", p])
        .assert()
        .success();

    // 9. Verify .claude/settings.json contains permissions
    let settings_path = tmp.path().join(".claude/settings.json");
    assert!(settings_path.exists(), ".claude/settings.json must exist");
    let settings: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&settings_path).unwrap()).unwrap();
    assert_eq!(
        settings["permissions"]["defaultMode"], "dontAsk",
        "defaultMode must come from ship-autonomous preset"
    );
    let deny = settings["permissions"]["deny"].as_array().unwrap();
    assert!(
        deny.iter().any(|d| d == "Bash(rm -rf *)"),
        "deny list must contain preset entries"
    );

    // 10. Verify .mcp.json contains the github server
    let mcp_json_path = tmp.path().join(".mcp.json");
    assert!(mcp_json_path.exists(), ".mcp.json must exist");
    let mcp: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&mcp_json_path).unwrap()).unwrap();
    assert!(
        mcp["mcpServers"]["github"].is_object(),
        "github server must appear in .mcp.json"
    );
}

/// Same .ship/ configuration compiles correctly to all 4 providers.
/// Verifies that rule content appears in every provider's context file
/// and that each provider produces its expected artifacts.
#[test]
fn e2e_cross_provider_same_config() {
    let tmp = TempDir::new().unwrap();
    let p = tmp.path().to_str().unwrap();

    // Set up a realistic .ship/ with agent, rules, permissions, MCP
    write(
        tmp.path(),
        ".ship/agents/cross.jsonc",
        r#"{
  "agent": {
    "id": "cross",
    "name": "Cross Provider",
    "version": "0.1.0",
    "providers": ["claude", "cursor", "gemini", "codex"]
  },
  "skills": { "refs": [] },
  "mcp": { "servers": [] },
  "permissions": {
    "preset": "ship-standard"
  },
  "rules": {}
}"#,
    );

    write(
        tmp.path(),
        ".ship/rules/architecture.md",
        "Keep modules under 300 lines. Prefer pure functions.",
    );

    write(
        tmp.path(),
        ".ship/permissions.jsonc",
        r#"{ "ship-standard": { "default_mode": "default" } }"#,
    );

    write(
        tmp.path(),
        ".ship/mcp.jsonc",
        r#"{
  "mcp": {
    "servers": {
      "test-server": {
        "id": "",
        "command": "echo",
        "args": ["hello"],
        "scope": "project",
        "server_type": "stdio",
        "disabled": false,
        "env": {}
      }
    }
  }
}"#,
    );

    let rule_text = "Keep modules under 300 lines";

    // Compile for claude
    ship()
        .args(["compile", "--provider", "claude", "--path", p])
        .assert()
        .success();
    assert!(
        tmp.path().join("CLAUDE.md").exists(),
        "CLAUDE.md must be written"
    );
    assert!(
        fs::read_to_string(tmp.path().join("CLAUDE.md"))
            .unwrap()
            .contains(rule_text),
        "CLAUDE.md must contain rule content"
    );
    assert!(
        tmp.path().join(".mcp.json").exists(),
        ".mcp.json must be written for claude"
    );

    // Compile for cursor
    ship()
        .args(["compile", "--provider", "cursor", "--path", p])
        .assert()
        .success();
    let cursor_rule = tmp.path().join(".cursor/rules/architecture.mdc");
    assert!(
        cursor_rule.exists(),
        ".cursor/rules/architecture.mdc must be written"
    );
    assert!(
        fs::read_to_string(&cursor_rule)
            .unwrap()
            .contains(rule_text),
        "cursor rule file must contain rule content"
    );

    // Compile for gemini
    ship()
        .args(["compile", "--provider", "gemini", "--path", p])
        .assert()
        .success();
    assert!(
        tmp.path().join("GEMINI.md").exists(),
        "GEMINI.md must be written"
    );
    assert!(
        fs::read_to_string(tmp.path().join("GEMINI.md"))
            .unwrap()
            .contains(rule_text),
        "GEMINI.md must contain rule content"
    );
    assert!(
        tmp.path().join(".gemini/settings.json").exists(),
        ".gemini/settings.json must be written"
    );

    // Compile for codex
    ship()
        .args(["compile", "--provider", "codex", "--path", p])
        .assert()
        .success();
    assert!(
        tmp.path().join("AGENTS.md").exists(),
        "AGENTS.md must be written"
    );
    assert!(
        fs::read_to_string(tmp.path().join("AGENTS.md"))
            .unwrap()
            .contains(rule_text),
        "AGENTS.md must contain rule content"
    );
    assert!(
        tmp.path().join(".codex/config.toml").exists(),
        ".codex/config.toml must be written"
    );
}

/// Validate that permission preset + agent-level overrides compose correctly.
/// The preset defines a deny list, the agent adds additional entries on top.
/// Both must appear in the compiled settings.json.
#[test]
fn e2e_permission_preset_plus_agent_override() {
    let tmp = TempDir::new().unwrap();
    let p = tmp.path().to_str().unwrap();

    // permissions.jsonc defines ship-autonomous with specific deny list
    write(
        tmp.path(),
        ".ship/permissions.jsonc",
        r#"{
  "ship-autonomous": {
    "default_mode": "dontAsk",
    "tools_allow": ["Read", "Write", "Edit", "Bash(*)"],
    "tools_deny": [
      "Bash(rm -rf *)",
      "Bash(git reset --hard*)",
      "Bash(git push*)"
    ]
  }
}"#,
    );

    // Agent uses preset AND adds its own tools_deny entries
    write(
        tmp.path(),
        ".ship/agents/strict-auto.jsonc",
        r#"{
  "agent": {
    "id": "strict-auto",
    "name": "Strict Autonomous",
    "version": "0.1.0",
    "providers": ["claude"]
  },
  "skills": { "refs": [] },
  "mcp": { "servers": [] },
  "permissions": {
    "preset": "ship-autonomous",
    "tools_deny": ["Bash(docker rm*)", "Bash(cargo publish*)"]
  },
  "rules": {}
}"#,
    );

    // ship use compiles with the agent active, applying preset + overrides
    ship_in(&tmp)
        .args(["use", "strict-auto", "--path", p])
        .assert()
        .success();

    let settings_path = tmp.path().join(".claude/settings.json");
    assert!(settings_path.exists(), ".claude/settings.json must exist");
    let settings: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&settings_path).unwrap()).unwrap();

    // Assert defaultMode matches the preset
    assert_eq!(
        settings["permissions"]["defaultMode"], "dontAsk",
        "defaultMode must come from ship-autonomous preset"
    );

    let deny = settings["permissions"]["deny"]
        .as_array()
        .expect("deny must be an array");

    // Assert preset deny entries are present
    assert!(
        deny.iter().any(|d| d == "Bash(rm -rf *)"),
        "preset deny entry 'Bash(rm -rf *)' must be present, got: {:?}",
        deny
    );
    assert!(
        deny.iter().any(|d| d == "Bash(git push*)"),
        "preset deny entry 'Bash(git push*)' must be present, got: {:?}",
        deny
    );

    // Assert agent-level deny additions are ALSO present
    assert!(
        deny.iter().any(|d| d == "Bash(docker rm*)"),
        "agent-level deny entry 'Bash(docker rm*)' must be present, got: {:?}",
        deny
    );
    assert!(
        deny.iter().any(|d| d == "Bash(cargo publish*)"),
        "agent-level deny entry 'Bash(cargo publish*)' must be present, got: {:?}",
        deny
    );
}

/// Test that installed dependency skills from the package cache appear in compiled output.
/// Simulates the cache directory structure that ship install would create.
#[test]
fn e2e_compile_with_dependency_skills() {
    let tmp = TempDir::new().unwrap();
    let p = tmp.path().to_str().unwrap();
    let hex = "a1b2c3d4e5f6";

    // ship.jsonc referencing a dependency
    write(
        tmp.path(),
        ".ship/ship.jsonc",
        r#"{
  "module": {
    "name": "github.com/test/my-project",
    "version": "0.1.0"
  },
  "dependencies": {
    "github.com/example/skills-pkg": "main"
  }
}"#,
    );

    // ship.lock with a matching entry
    write(
        tmp.path(),
        ".ship/ship.lock",
        &format!(
            "version = 1\n\n[[package]]\npath = \"github.com/example/skills-pkg\"\n\
             version = \"main\"\ncommit = \"{}\"\nhash = \"sha256:{}\"\n",
            "a".repeat(40),
            hex,
        ),
    );

    // Agent referencing the dep skill
    write(
        tmp.path(),
        ".ship/agents/with-deps.jsonc",
        r#"{
  "agent": {
    "id": "with-deps",
    "name": "With Deps",
    "version": "0.1.0",
    "providers": ["claude"]
  },
  "skills": {
    "refs": ["github.com/example/skills-pkg/skills/cool-skill"]
  },
  "mcp": { "servers": [] },
  "permissions": {},
  "rules": {}
}"#,
    );

    // Simulate installed package cache at $HOME/.ship/cache/objects/<hex>/
    // The dep_skills resolver uses dirs::home_dir() for the cache path.
    // Set HOME to tmp so the cache is found at tmp/.ship/cache/objects/<hex>/
    write(
        tmp.path(),
        &format!(".ship/cache/objects/{}/skills/cool-skill/SKILL.md", hex),
        "---\nname: Cool Skill\ndescription: A cool dependency skill\n---\n\nYou are a cool skill. Follow best practices.",
    );

    // Use ship_in for DB isolation and set HOME for cache resolution.
    // Both SHIP_GLOBAL_DIR (for platform.db) and HOME (for cache) point to tmp.
    ship_in(&tmp)
        .env("HOME", tmp.path())
        .args(["use", "with-deps", "--path", p])
        .assert()
        .success();

    // Verify the dep skill is written as a skill file (skills are compiled
    // into .claude/skills/<id>/SKILL.md, not inlined into CLAUDE.md)
    let skill_path = tmp
        .path()
        .join(".claude/skills/github.com/example/skills-pkg/skills/cool-skill/SKILL.md");
    assert!(
        skill_path.exists(),
        "dep skill file must be written at {:?}",
        skill_path,
    );
    let skill_content = fs::read_to_string(&skill_path).unwrap();
    assert!(
        skill_content.contains("You are a cool skill"),
        "skill file must contain dep skill content, got: {}",
        skill_content
    );
}

/// Validate catches bad config, then after fixing it, validate and compile succeed.
#[test]
fn e2e_validate_then_compile_catches_bad_config() {
    let tmp = TempDir::new().unwrap();
    let p = tmp.path().to_str().unwrap();

    // Write invalid agent JSONC (missing required `id` field in agent)
    write(
        tmp.path(),
        ".ship/agents/broken.jsonc",
        r#"{ "agent": { "name": "Broken" } }"#,
    );

    // Validate should fail
    ship().args(["validate", "--path", p]).assert().failure();

    // Fix the config by adding the required fields
    write(
        tmp.path(),
        ".ship/agents/broken.jsonc",
        r#"{
  "agent": {
    "id": "broken",
    "name": "Fixed Agent",
    "providers": ["claude"]
  },
  "skills": { "refs": [] },
  "mcp": { "servers": [] },
  "permissions": {},
  "rules": {}
}"#,
    );

    // Validate should now pass
    ship()
        .args(["validate", "--path", p])
        .assert()
        .success()
        .stdout(predicate::str::contains("valid"));

    // Compile should succeed
    ship()
        .args(["compile", "--provider", "claude", "--path", p])
        .assert()
        .success();

    // The config has no rules, so CLAUDE.md is correctly not written.
    // The MCP config is always written for the claude provider.
    assert!(
        !tmp.path().join("CLAUDE.md").exists(),
        "CLAUDE.md must NOT be written when there are no rules"
    );
    assert!(
        tmp.path().join(".mcp.json").exists(),
        ".mcp.json must be written after successful compile"
    );
}
