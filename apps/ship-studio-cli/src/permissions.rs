//! `ship permissions sync` — import session permission decisions back into the active agent.
//!
//! Claude Code accumulates "Allow always" and "Deny" decisions in
//! `.claude/settings.local.json` during a session. `ship use` overwrites the
//! compiled config and those decisions are lost. This command reads the session
//! decisions, diffs them against the agent's compiled allow/deny lists, and
//! writes the delta back into the agent TOML so they survive the next `ship use`.
//!
//! Safety rule: deny rules that would shadow a tool already in the agent's
//! allow list are flagged as warnings rather than silently imported. They may
//! represent accidental clicks.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

use crate::profile::{WorkspaceState, find_agent_file};

// ── Claude session settings format ────────────────────────────────────────────

/// Subset of `.claude/settings.local.json` we care about.
#[derive(Debug, Deserialize, Default)]
struct ClaudeLocalSettings {
    #[serde(default)]
    permissions: ClaudeLocalPermissions,
}

#[derive(Debug, Deserialize, Default)]
struct ClaudeLocalPermissions {
    #[serde(default)]
    allow: Vec<String>,
    #[serde(default)]
    deny: Vec<String>,
}

// ── Agent TOML permission section ─────────────────────────────────────────────

/// Minimal parse of the agent TOML to read and update permission lists.
/// We do a targeted text patch so we don't lose comments or formatting.
#[derive(Debug, Deserialize)]
struct ProfileForSync {
    #[serde(default)]
    permissions: ProfilePermissionsForSync,
}

#[derive(Debug, Deserialize, Default)]
struct ProfilePermissionsForSync {
    #[serde(default)]
    tools_allow: Vec<String>,
    #[serde(default)]
    tools_deny: Vec<String>,
    /// Preset tier shorthand — used to detect what baseline the agent applies.
    preset: Option<String>,
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn run_permissions_sync(project_root: &Path) -> Result<()> {
    let ship_dir = project_root.join(".ship");

    // 1. Find the active agent.
    let state = WorkspaceState::load(&ship_dir);
    let agent_id = state.active_agent
        .as_deref()
        .context("No active agent. Run: ship use <agent-id>")?;

    let profile_path = find_agent_file(agent_id, project_root)
        .with_context(|| format!("Agent '{}' not found in .ship/agents/", agent_id))?;

    // 2. Read session decisions from .claude/settings.local.json
    let local_path = project_root.join(".claude").join("settings.local.json");
    let session = load_local_settings(&local_path)?;

    if session.permissions.allow.is_empty() && session.permissions.deny.is_empty() {
        println!("No session permission decisions found in {}", local_path.display());
        return Ok(());
    }

    // 3. Read the agent's current permission lists.
    let profile_raw = std::fs::read_to_string(&profile_path)
        .with_context(|| format!("Cannot read agent at {}", profile_path.display()))?;
    let profile: ProfileForSync = toml::from_str(&profile_raw)
        .with_context(|| format!("Invalid TOML in {}", profile_path.display()))?;

    let current_allow: std::collections::HashSet<String> =
        profile.permissions.tools_allow.iter().cloned().collect();
    let current_deny: std::collections::HashSet<String> =
        profile.permissions.tools_deny.iter().cloned().collect();

    // 4. Compute the delta — only new entries not already in the agent.
    let new_allows: Vec<String> = session.permissions.allow.iter()
        .filter(|a| !current_allow.contains(*a))
        .cloned()
        .collect();

    let mut new_denies: Vec<String> = Vec::new();
    let mut warned_denies: Vec<String> = Vec::new();

    for deny in &session.permissions.deny {
        if current_deny.contains(deny) {
            continue; // already present
        }
        // Warn if this deny would shadow something explicitly allowed in the agent.
        if current_allow.contains(deny) || is_shadowing_allow(deny, &profile.permissions.tools_allow) {
            warned_denies.push(deny.clone());
        } else {
            new_denies.push(deny.clone());
        }
    }

    if new_allows.is_empty() && new_denies.is_empty() && warned_denies.is_empty() {
        println!("Agent '{}' already contains all session decisions — nothing to do.", agent_id);
        return Ok(());
    }

    // 5. Print what we found.
    if !new_allows.is_empty() {
        println!("New allow rules to import ({}):", new_allows.len());
        for a in &new_allows { println!("  + allow: {}", a); }
    }
    if !new_denies.is_empty() {
        println!("New deny rules to import ({}):", new_denies.len());
        for d in &new_denies { println!("  + deny: {}", d); }
    }
    if !warned_denies.is_empty() {
        println!("WARNING: {} deny rule(s) shadow tools in the agent's allow list.", warned_denies.len());
        println!("These were NOT imported — review and add manually if intentional:");
        for d in &warned_denies { println!("  ! deny: {}", d); }
    }

    // 6. Patch the agent TOML in place.
    let updated = patch_profile_toml(&profile_raw, &new_allows, &new_denies)?;
    std::fs::write(&profile_path, &updated)
        .with_context(|| format!("Cannot write agent at {}", profile_path.display()))?;

    println!("✓ updated agent '{}' at {}", agent_id, profile_path.display());
    println!("  Run 'ship use' to recompile the updated agent.");

    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn load_local_settings(path: &Path) -> Result<ClaudeLocalSettings> {
    if !path.exists() {
        return Ok(ClaudeLocalSettings::default());
    }
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("Cannot read {}", path.display()))?;
    serde_json::from_str(&raw)
        .with_context(|| format!("Invalid JSON in {}", path.display()))
}

/// Returns true if `deny_pattern` is a substring-match or exact match for any
/// entry in the agent's allow list. Catches patterns like "Bash" matching "Bash(*)".
fn is_shadowing_allow(deny_pattern: &str, allow_list: &[String]) -> bool {
    allow_list.iter().any(|a| {
        a == deny_pattern
            || a.starts_with(&format!("{}(", deny_pattern))
            || deny_pattern.starts_with(&format!("{}(", a))
    })
}

/// Patch an agent TOML string by appending new allow/deny entries.
///
/// Strategy: parse the existing tools_allow and tools_deny arrays (if present)
/// and rebuild the [permissions] section. If no [permissions] section exists,
/// append one. Idempotent — calling twice with the same delta is a no-op
/// (delta is empty on the second call since values are now in the agent).
fn patch_profile_toml(
    original: &str,
    new_allows: &[String],
    new_denies: &[String],
) -> Result<String> {
    if new_allows.is_empty() && new_denies.is_empty() {
        return Ok(original.to_string());
    }

    // Re-parse to get full current lists for reconstruction.
    let profile: ProfileForSync = toml::from_str(original)?;

    let mut allow_list: Vec<String> = profile.permissions.tools_allow.clone();
    let mut deny_list: Vec<String> = profile.permissions.tools_deny.clone();

    for a in new_allows {
        if !allow_list.contains(a) { allow_list.push(a.clone()); }
    }
    for d in new_denies {
        if !deny_list.contains(d) { deny_list.push(d.clone()); }
    }

    // Build the new [permissions] section.
    let new_perms_block = build_permissions_block(
        profile.permissions.preset.as_deref(),
        &allow_list,
        &deny_list,
    );

    // Replace or append the [permissions] section.
    if original.contains("[permissions]") {
        replace_permissions_section(original, &new_perms_block)
    } else {
        Ok(format!("{}\n{}", original.trim_end(), new_perms_block))
    }
}

fn build_permissions_block(
    preset: Option<&str>,
    allow: &[String],
    deny: &[String],
) -> String {
    let mut lines = vec!["[permissions]".to_string()];
    if let Some(p) = preset {
        lines.push(format!("preset = {:?}", p));
    }
    if !allow.is_empty() {
        let vals: Vec<String> = allow.iter().map(|s| format!("{:?}", s)).collect();
        lines.push(format!("tools_allow = [{}]", vals.join(", ")));
    }
    if !deny.is_empty() {
        let vals: Vec<String> = deny.iter().map(|s| format!("{:?}", s)).collect();
        lines.push(format!("tools_deny = [{}]", vals.join(", ")));
    }
    lines.join("\n") + "\n"
}

/// Replace the `[permissions]` section in a TOML string with new content.
/// Assumes [permissions] is a top-level section (not nested).
fn replace_permissions_section(original: &str, new_block: &str) -> Result<String> {
    let mut before = String::new();
    let mut after = String::new();
    let mut in_permissions = false;
    let mut found = false;

    for line in original.lines() {
        if line.trim() == "[permissions]" {
            in_permissions = true;
            found = true;
            continue;
        }
        if in_permissions {
            // Another top-level section starts — stop consuming permissions.
            if line.starts_with('[') && !line.starts_with("[[") {
                in_permissions = false;
                after.push_str(line);
                after.push('\n');
            }
            // Skip lines that belong to the old [permissions] block.
            continue;
        }
        if found {
            after.push_str(line);
            after.push('\n');
        } else {
            before.push_str(line);
            before.push('\n');
        }
    }

    Ok(format!("{}\n{}{}", before.trim_end(), new_block, after))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write(dir: &Path, rel: &str, content: &str) {
        let p = dir.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, content).unwrap();
    }

    #[test]
    fn patch_adds_new_allows_to_empty_permissions() {
        let toml = r#"[agent]
name = "Test"
id = "test"
providers = ["claude"]
"#;
        let result = patch_profile_toml(toml, &["Bash(cargo test*)".to_string()], &[]).unwrap();
        let parsed: ProfileForSync = toml::from_str(&result).unwrap();
        assert!(parsed.permissions.tools_allow.contains(&"Bash(cargo test*)".to_string()));
    }

    #[test]
    fn patch_adds_new_denies() {
        let toml = r#"[agent]
name = "Test"
id = "test"
providers = ["claude"]

[permissions]
tools_allow = ["Read"]
"#;
        let result = patch_profile_toml(toml, &[], &["Bash(rm -rf *)".to_string()]).unwrap();
        let parsed: ProfileForSync = toml::from_str(&result).unwrap();
        assert!(parsed.permissions.tools_deny.contains(&"Bash(rm -rf *)".to_string()));
        // Existing allow must survive.
        assert!(parsed.permissions.tools_allow.contains(&"Read".to_string()));
    }

    #[test]
    fn patch_is_idempotent() {
        let toml = r#"[agent]
name = "Test"
id = "test"
providers = ["claude"]

[permissions]
tools_allow = ["Bash(cargo test*)"]
"#;
        // Delta is now empty — already in agent.
        let result = patch_profile_toml(toml, &[], &[]).unwrap();
        let parsed: ProfileForSync = toml::from_str(&result).unwrap();
        // Existing allow must survive unchanged.
        assert_eq!(parsed.permissions.tools_allow, vec!["Bash(cargo test*)"]);
    }

    #[test]
    fn is_shadowing_allow_detects_prefix_match() {
        let allow = vec!["Bash(cargo*)".to_string()];
        // "Bash" shadows "Bash(cargo*)"
        assert!(is_shadowing_allow("Bash", &allow));
        // "Read" does not shadow "Bash(cargo*)"
        assert!(!is_shadowing_allow("Read", &allow));
    }

    #[test]
    fn load_local_settings_returns_default_when_missing() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join(".claude").join("settings.local.json");
        let settings = load_local_settings(&path).unwrap();
        assert!(settings.permissions.allow.is_empty());
        assert!(settings.permissions.deny.is_empty());
    }

    #[test]
    fn load_local_settings_parses_allow_deny() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("settings.local.json");
        write(tmp.path(), "settings.local.json", r#"
{
  "permissions": {
    "allow": ["Bash(cargo test*)"],
    "deny": ["Bash(rm -rf *)"]
  }
}
"#);
        let settings = load_local_settings(&path).unwrap();
        assert_eq!(settings.permissions.allow, vec!["Bash(cargo test*)"]);
        assert_eq!(settings.permissions.deny, vec!["Bash(rm -rf *)"]);
    }

    #[test]
    fn sync_writes_new_allows_to_agent() {
        let tmp = TempDir::new().unwrap();
        let ship_dir = tmp.path().join(".ship");
        write(tmp.path(), ".ship/ship.toml", "id = \"test-proj\"\nname = \"test\"\n");
        // Write workspace state with active agent via platform.db
        let mut state = WorkspaceState::default();
        state.active_agent = Some("test".to_string());
        state.save(&ship_dir).unwrap();
        // Write the agent
        write(tmp.path(), ".ship/agents/test.toml", r#"[agent]
name = "Test"
id = "test"
providers = ["claude"]
"#);
        // Write local session decisions
        write(tmp.path(), ".claude/settings.local.json", r#"
{
  "permissions": {
    "allow": ["Bash(cargo build*)"],
    "deny": []
  }
}
"#);
        run_permissions_sync(tmp.path()).unwrap();
        let updated = std::fs::read_to_string(
            tmp.path().join(".ship/agents/test.toml")
        ).unwrap();
        let parsed: ProfileForSync = toml::from_str(&updated).unwrap();
        assert!(parsed.permissions.tools_allow.contains(&"Bash(cargo build*)".to_string()));
    }

    #[test]
    fn sync_warns_deny_that_shadows_allow_but_does_not_import() {
        let tmp = TempDir::new().unwrap();
        let ship_dir = tmp.path().join(".ship");
        write(tmp.path(), ".ship/ship.toml", "id = \"test-proj\"\nname = \"test\"\n");
        // Write workspace state with active agent via platform.db
        let mut state = WorkspaceState::default();
        state.active_agent = Some("test".to_string());
        state.save(&ship_dir).unwrap();
        write(tmp.path(), ".ship/agents/test.toml", r#"[agent]
name = "Test"
id = "test"
providers = ["claude"]

[permissions]
tools_allow = ["Bash(cargo*)"]
"#);
        // Session tried to deny "Bash" which would shadow "Bash(cargo*)"
        write(tmp.path(), ".claude/settings.local.json", r#"
{
  "permissions": {
    "allow": [],
    "deny": ["Bash"]
  }
}
"#);
        // Should succeed without panicking; the deny must NOT be imported.
        run_permissions_sync(tmp.path()).unwrap();
        let updated = std::fs::read_to_string(
            tmp.path().join(".ship/agents/test.toml")
        ).unwrap();
        let parsed: ProfileForSync = toml::from_str(&updated).unwrap();
        assert!(!parsed.permissions.tools_deny.contains(&"Bash".to_string()),
            "shadowing deny must not be imported");
    }
}
