//! Agent activation — `ship use [<agent-id>]`.
//! Workspace state (active_agent, compiled_at, plugins_installed) is stored
//! in platform.db kv_state (namespace='workspace'). ship.lock is reserved for
//! the registry lockfile format.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use crate::agent_config::AgentConfig;
use crate::compile::{CompileOptions, run_compile};

const NS: &str = "workspace";
const KEY_ACTIVE_AGENT: &str = "active_agent";
const KEY_COMPILED_AT: &str = "compiled_at";
const KEY_PLUGINS_INSTALLED: &str = "plugins_installed";

// ── WorkspaceState ────────────────────────────────────────────────────────────

/// Workspace runtime state, persisted in platform.db kv_state (namespace='workspace').
/// Previously stored in .ship/ship.lock (TOML format). Migration runs on first load.
#[derive(Debug, Clone, Default)]
pub struct WorkspaceState {
    pub active_agent: Option<String>,
    pub compiled_at: Option<String>,
    pub plugins_installed: Vec<String>,
}

impl WorkspaceState {
    /// Load from platform.db.
    pub fn load(_ship_dir: &Path) -> Self {
        let mut state = WorkspaceState::default();
        if let Err(e) = runtime::db::ensure_db() {
            eprintln!("warning: could not open platform.db: {}", e);
            return state;
        }
        if let Ok(Some(v)) = runtime::db::kv::get(NS, KEY_ACTIVE_AGENT) {
            state.active_agent = v.as_str().map(str::to_string);
        }
        // Compat: also check the old key name for projects that haven't re-activated yet
        if state.active_agent.is_none()
            && let Ok(Some(v)) = runtime::db::kv::get(NS, "active_profile")
        {
            state.active_agent = v.as_str().map(str::to_string);
        }
        if let Ok(Some(v)) = runtime::db::kv::get(NS, KEY_COMPILED_AT) {
            state.compiled_at = v.as_str().map(str::to_string);
        }
        if let Ok(Some(v)) = runtime::db::kv::get(NS, KEY_PLUGINS_INSTALLED)
            && let Some(arr) = v.as_array()
        {
            state.plugins_installed = arr
                .iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect();
        }
        state
    }

    /// Persist workspace state to platform.db.
    pub fn save(&self, _ship_dir: &Path) -> Result<()> {
        runtime::db::ensure_db().context("failed to open platform.db")?;
        if let Some(ref p) = self.active_agent {
            runtime::db::kv::set(NS, KEY_ACTIVE_AGENT, &serde_json::json!(p))?;
        }
        if let Some(ref t) = self.compiled_at {
            runtime::db::kv::set(NS, KEY_COMPILED_AT, &serde_json::json!(t))?;
        }
        runtime::db::kv::set(
            NS,
            KEY_PLUGINS_INSTALLED,
            &serde_json::json!(self.plugins_installed),
        )?;
        Ok(())
    }
}

// ── Migration (removed) ──────────────────────────────────────────────────────
// ship.lock workspace-state migration was a one-time path from the old TOML
// format to platform.db. No consumers remain. Deleted 2026-03-20.

// ── Activation ────────────────────────────────────────────────────────────────

/// Activate an agent: compile + install plugins + persist workspace state to platform.db.
/// If `agent_id` is None, re-runs the active agent from platform.db.
pub fn activate_agent(
    agent_id: Option<&str>,
    project_root: &Path,
    output_root: Option<&Path>,
) -> Result<()> {
    let ship_dir = project_root.join(".ship");
    let mut state = WorkspaceState::load(&ship_dir);
    let id = agent_id
        .map(str::to_string)
        .or_else(|| state.active_agent.clone())
        .context("No agent specified and no active agent set. Run: ship use <agent-id>")?;

    let agent_path = find_agent_file(&id, project_root)
        .with_context(|| format!("Agent '{}' not found in .ship/agents/", id))?;
    let agent = AgentConfig::load(&agent_path)?;

    run_compile(CompileOptions {
        project_root,
        output_root,
        provider: None,
        dry_run: false,
        active_agent: Some(&id),
    })?;

    let now_plugins: Vec<String> = agent.plugins.install.clone();
    let prev_plugins = state.plugins_installed.clone();
    run_plugin_lifecycle(&now_plugins, &prev_plugins, &agent.plugins.scope);

    state.active_agent = Some(id.clone());
    state.compiled_at = Some(chrono::Utc::now().to_rfc3339());
    state.plugins_installed = now_plugins;
    state.save(&ship_dir)?;
    println!("✓ activated agent '{}'", id);
    Ok(())
}

fn run_plugin_lifecycle(now: &[String], prev: &[String], scope: &str) {
    for plugin in now {
        if !prev.contains(plugin) {
            let status = std::process::Command::new("claude")
                .args(["plugin", "install", plugin, "--scope", scope])
                .status();
            match status {
                Ok(s) if s.success() => println!("  + plugin {}", plugin),
                Ok(_) => eprintln!("  warning: plugin install failed for {}", plugin),
                Err(_) => eprintln!(
                    "  warning: claude CLI not found — skipping plugin install for {}",
                    plugin
                ),
            }
        }
    }
    for plugin in prev {
        if !now.contains(plugin) {
            let status = std::process::Command::new("claude")
                .args(["plugin", "uninstall", plugin])
                .status();
            match status {
                Ok(s) if s.success() => println!("  - plugin {}", plugin),
                _ => eprintln!("  warning: plugin uninstall failed for {}", plugin),
            }
        }
    }
}

/// Search order: agents/ → agents/profiles/ (compat) → agents/presets/ (compat) → modes/ (legacy), project then global.
/// Within each directory, `.jsonc` is checked before `.toml`.
pub fn find_agent_file(agent_id: &str, project_root: &Path) -> Option<PathBuf> {
    let ship = project_root.join(".ship");
    let jsonc_file = format!("{}.jsonc", agent_id);
    let toml_file = format!("{}.toml", agent_id);

    // Helper: check jsonc then toml in a directory
    let check_dir = |dir: PathBuf| -> Option<PathBuf> {
        let j = dir.join(&jsonc_file);
        if j.exists() {
            return Some(j);
        }
        let t = dir.join(&toml_file);
        if t.exists() {
            return Some(t);
        }
        None
    };

    // Primary: .ship/agents/<id>.{jsonc,toml}
    if let Some(p) = check_dir(ship.join("agents")) {
        return Some(p);
    }
    // Compat: .ship/agents/profiles/<id>.{jsonc,toml}
    if let Some(p) = check_dir(ship.join("agents").join("profiles")) {
        return Some(p);
    }
    // Compat: .ship/agents/presets/<id>.{jsonc,toml}
    if let Some(p) = check_dir(ship.join("agents").join("presets")) {
        return Some(p);
    }
    // Legacy: .ship/modes/<id>.{jsonc,toml}
    if let Some(p) = check_dir(ship.join("modes")) {
        return Some(p);
    }
    // Global dirs
    let home = dirs::home_dir()?;
    let global_ship = home.join(".ship");
    if let Some(p) = check_dir(global_ship.join("agents")) {
        return Some(p);
    }
    if let Some(p) = check_dir(global_ship.join("agents").join("profiles")) {
        return Some(p);
    }
    if let Some(p) = check_dir(global_ship.join("agents").join("presets")) {
        return Some(p);
    }
    if let Some(p) = check_dir(global_ship.join("modes")) {
        return Some(p);
    }
    None
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_file(dir: &Path, rel: &str, content: &str) {
        let p = dir.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, content).unwrap();
    }

    fn setup_ship_dir(tmp: &TempDir) -> PathBuf {
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        std::fs::write(
            ship_dir.join("ship.toml"),
            "id = \"test-proj-id\"\nname = \"test\"\n",
        )
        .unwrap();
        ship_dir
    }

    #[test]
    fn workspace_state_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let ship_dir = setup_ship_dir(&tmp);
        let state = WorkspaceState {
            active_agent: Some("cli-lane".to_string()),
            compiled_at: Some("2026-01-01T00:00:00Z".to_string()),
            plugins_installed: vec!["superpowers@official".to_string()],
        };
        state.save(&ship_dir).unwrap();
        let loaded = WorkspaceState::load(&ship_dir);
        assert_eq!(loaded.active_agent.as_deref(), Some("cli-lane"));
        assert_eq!(loaded.compiled_at.as_deref(), Some("2026-01-01T00:00:00Z"));
        assert_eq!(loaded.plugins_installed, vec!["superpowers@official"]);
    }

    #[test]
    fn workspace_state_load_missing_returns_default() {
        let tmp = TempDir::new().unwrap();
        let ship_dir = setup_ship_dir(&tmp);
        let state = WorkspaceState::load(&ship_dir);
        assert!(state.active_agent.is_none());
        assert!(state.plugins_installed.is_empty());
    }

    // ship.lock workspace-state migration tests removed — migration code deleted 2026-03-20.

    #[test]
    fn find_agent_file_finds_new_location() {
        let tmp = TempDir::new().unwrap();
        write_file(
            tmp.path(),
            ".ship/agents/test.toml",
            "[agent]\nname=\"Test\"\nid=\"test\"\n",
        );
        let found = find_agent_file("test", tmp.path());
        assert!(found.is_some());
        assert!(
            found
                .unwrap()
                .to_string_lossy()
                .contains("agents/test.toml")
        );
    }

    #[test]
    fn find_agent_file_compat_profiles_location() {
        let tmp = TempDir::new().unwrap();
        write_file(
            tmp.path(),
            ".ship/agents/profiles/test.toml",
            "[agent]\nname=\"Test\"\nid=\"test\"\n",
        );
        let found = find_agent_file("test", tmp.path());
        assert!(found.is_some());
        assert!(found.unwrap().to_string_lossy().contains("agents/profiles"));
    }

    #[test]
    fn find_agent_file_returns_none_for_missing() {
        let tmp = TempDir::new().unwrap();
        assert!(find_agent_file("nonexistent", tmp.path()).is_none());
    }

    #[test]
    fn find_agent_file_prefers_jsonc() {
        let tmp = TempDir::new().unwrap();
        write_file(
            tmp.path(),
            ".ship/agents/test.toml",
            "[agent]\nname=\"Test\"\n",
        );
        write_file(
            tmp.path(),
            ".ship/agents/test.jsonc",
            r#"{ "agent": { "name": "Test" } }"#,
        );
        let found = find_agent_file("test", tmp.path());
        assert!(found.is_some());
        assert!(found.unwrap().to_string_lossy().ends_with(".jsonc"));
    }

    #[test]
    fn find_agent_file_finds_jsonc_only() {
        let tmp = TempDir::new().unwrap();
        write_file(
            tmp.path(),
            ".ship/agents/test.jsonc",
            r#"{ "agent": { "name": "Test" } }"#,
        );
        let found = find_agent_file("test", tmp.path());
        assert!(found.is_some());
        assert!(found.unwrap().to_string_lossy().ends_with(".jsonc"));
    }
}
