//! OpenCode agent decompilation tests.

use std::io::Write;

use serde_json::json;
use tempfile::TempDir;

use super::opencode::decompile_opencode;

fn write_file(dir: &TempDir, path: &str, content: &str) {
    let full = dir.path().join(path);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let mut f = std::fs::File::create(&full).unwrap();
    f.write_all(content.as_bytes()).unwrap();
}

// ── Agent with model, description, prompt ────────────────────────────────────

#[test]
fn opencode_agent_basic_fields() {
    let tmp = TempDir::new().unwrap();
    let config = json!({
        "agent": {
            "build": {
                "model": "anthropic/claude-sonnet-4-20250514",
                "description": "Build agent",
                "prompt": "You are a build agent. Follow TDD."
            }
        }
    });
    write_file(
        &tmp,
        "opencode.json",
        &serde_json::to_string(&config).unwrap(),
    );

    let lib = decompile_opencode(tmp.path());

    assert_eq!(lib.agent_profiles.len(), 1);
    let agent = &lib.agent_profiles[0];
    assert_eq!(agent.profile.id, "build");
    assert_eq!(agent.profile.name, "build");
    assert_eq!(agent.profile.description.as_deref(), Some("Build agent"));
    assert_eq!(
        agent.rules.inline.as_deref(),
        Some("You are a build agent. Follow TDD.")
    );

    // Model stored in provider_settings.opencode
    let oc = agent.provider_settings.get("opencode").unwrap();
    assert_eq!(
        oc.get("model").and_then(|v| v.as_str()),
        Some("anthropic/claude-sonnet-4-20250514")
    );
}

// ── Agent with permission object ─────────────────────────────────────────────

#[test]
fn opencode_agent_permissions_simple() {
    let tmp = TempDir::new().unwrap();
    let config = json!({
        "agent": {
            "coder": {
                "permission": {
                    "edit": "allow",
                    "read": "allow",
                    "bash": "deny",
                    "grep": "ask"
                }
            }
        }
    });
    write_file(
        &tmp,
        "opencode.json",
        &serde_json::to_string(&config).unwrap(),
    );

    let lib = decompile_opencode(tmp.path());

    assert_eq!(lib.agent_profiles.len(), 1);
    let perms = &lib.agent_profiles[0].permissions;

    // edit → Edit/Write in allow, read → Read in allow
    assert!(perms.tools_allow.contains(&"Edit".to_string()));
    assert!(perms.tools_allow.contains(&"Write".to_string()));
    assert!(perms.tools_allow.contains(&"Read".to_string()));

    // bash → Bash in deny
    assert!(perms.tools_deny.contains(&"Bash".to_string()));

    // grep → Grep in ask
    assert!(perms.tools_ask.contains(&"Grep".to_string()));
}

#[test]
fn opencode_agent_permissions_granular_bash() {
    let tmp = TempDir::new().unwrap();
    let config = json!({
        "agent": {
            "ops": {
                "permission": {
                    "bash": {
                        "git *": "allow",
                        "rm *": "deny",
                        "cargo test": "ask"
                    }
                }
            }
        }
    });
    write_file(
        &tmp,
        "opencode.json",
        &serde_json::to_string(&config).unwrap(),
    );

    let lib = decompile_opencode(tmp.path());

    let perms = &lib.agent_profiles[0].permissions;
    assert!(perms.tools_allow.contains(&"Bash(git *)".to_string()));
    assert!(perms.tools_deny.contains(&"Bash(rm *)".to_string()));
    assert!(perms.tools_ask.contains(&"Bash(cargo test)".to_string()));
}

// ── Agent with unknown fields preserved ──────────────────────────────────────

#[test]
fn opencode_agent_unknown_fields_in_provider_settings() {
    let tmp = TempDir::new().unwrap();
    let config = json!({
        "agent": {
            "reviewer": {
                "mode": "primary",
                "temperature": 0.7,
                "steps": 50,
                "hidden": true,
                "color": "#ff0000"
            }
        }
    });
    write_file(
        &tmp,
        "opencode.json",
        &serde_json::to_string(&config).unwrap(),
    );

    let lib = decompile_opencode(tmp.path());

    assert_eq!(lib.agent_profiles.len(), 1);
    let oc = lib.agent_profiles[0]
        .provider_settings
        .get("opencode")
        .unwrap();
    assert_eq!(oc.get("mode").and_then(|v| v.as_str()), Some("primary"));
    assert_eq!(oc.get("temperature").and_then(|v| v.as_float()), Some(0.7));
    assert_eq!(oc.get("steps").and_then(|v| v.as_integer()), Some(50));
    assert_eq!(oc.get("hidden").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(oc.get("color").and_then(|v| v.as_str()), Some("#ff0000"));
}

// ── Multiple agents ──────────────────────────────────────────────────────────

#[test]
fn opencode_multiple_agents() {
    let tmp = TempDir::new().unwrap();
    let config = json!({
        "agent": {
            "build": {
                "description": "Build agent",
                "model": "anthropic/claude-sonnet-4-20250514"
            },
            "review": {
                "description": "Review agent",
                "model": "openai/gpt-4o"
            },
            "deploy": {
                "description": "Deploy agent"
            }
        }
    });
    write_file(
        &tmp,
        "opencode.json",
        &serde_json::to_string(&config).unwrap(),
    );

    let lib = decompile_opencode(tmp.path());

    assert_eq!(lib.agent_profiles.len(), 3);
    let ids: Vec<&str> = lib
        .agent_profiles
        .iter()
        .map(|a| a.profile.id.as_str())
        .collect();
    assert!(ids.contains(&"build"));
    assert!(ids.contains(&"review"));
    assert!(ids.contains(&"deploy"));
}

// ── Agent key added to KNOWN_CONFIG_KEYS ─────────────────────────────────────

#[test]
fn opencode_agent_key_not_in_provider_defaults() {
    let tmp = TempDir::new().unwrap();
    let config = json!({
        "model": "gpt-4o",
        "agent": {
            "build": {
                "description": "Build agent"
            }
        }
    });
    write_file(
        &tmp,
        "opencode.json",
        &serde_json::to_string(&config).unwrap(),
    );

    let lib = decompile_opencode(tmp.path());

    // The "agent" key should NOT appear in provider_defaults
    if let Some(defaults) = lib.provider_defaults.get("opencode") {
        assert!(
            defaults.get("agent").is_none(),
            "agent key should be in KNOWN_CONFIG_KEYS, not provider_defaults"
        );
    }
}
