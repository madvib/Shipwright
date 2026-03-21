//! Profile activation — `ship use [<profile-id>]`.
//! Workspace state (active_profile, compiled_at, plugins_installed) is stored
//! in platform.db kv_state (namespace='workspace'). ship.lock is reserved for
//! the registry lockfile format.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use crate::compile::{CompileOptions, run_compile};
use crate::mode::Profile;

const NS: &str = "workspace";
const KEY_ACTIVE_PROFILE: &str = "active_profile";
const KEY_COMPILED_AT: &str = "compiled_at";
const KEY_PLUGINS_INSTALLED: &str = "plugins_installed";

// ── WorkspaceState ────────────────────────────────────────────────────────────

/// Workspace runtime state, persisted in platform.db kv_state (namespace='workspace').
/// Previously stored in .ship/ship.lock (TOML format). Migration runs on first load.
#[derive(Debug, Clone, Default)]
pub struct WorkspaceState {
    pub active_profile: Option<String>,
    pub compiled_at: Option<String>,
    pub plugins_installed: Vec<String>,
}

impl WorkspaceState {
    /// Load from platform.db.
    pub fn load(ship_dir: &Path) -> Self {
        let mut state = WorkspaceState::default();
        if let Err(e) = runtime::db::ensure_db(ship_dir) {
            eprintln!("warning: could not open platform.db: {}", e);
            return state;
        }
        if let Ok(Some(v)) = runtime::db::kv::get(ship_dir, NS, KEY_ACTIVE_PROFILE) {
            state.active_profile = v.as_str().map(str::to_string);
        }
        if let Ok(Some(v)) = runtime::db::kv::get(ship_dir, NS, KEY_COMPILED_AT) {
            state.compiled_at = v.as_str().map(str::to_string);
        }
        if let Ok(Some(v)) = runtime::db::kv::get(ship_dir, NS, KEY_PLUGINS_INSTALLED)
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
    pub fn save(&self, ship_dir: &Path) -> Result<()> {
        runtime::db::ensure_db(ship_dir).context("failed to open platform.db")?;
        if let Some(ref p) = self.active_profile {
            runtime::db::kv::set(ship_dir, NS, KEY_ACTIVE_PROFILE, &serde_json::json!(p))?;
        }
        if let Some(ref t) = self.compiled_at {
            runtime::db::kv::set(ship_dir, NS, KEY_COMPILED_AT, &serde_json::json!(t))?;
        }
        runtime::db::kv::set(
            ship_dir, NS, KEY_PLUGINS_INSTALLED, &serde_json::json!(self.plugins_installed),
        )?;
        Ok(())
    }
}

// ── Migration (removed) ──────────────────────────────────────────────────────
// ship.lock workspace-state migration was a one-time path from the old TOML
// format to platform.db. No consumers remain. Deleted 2026-03-20.

// ── Activation ────────────────────────────────────────────────────────────────

/// Activate a profile: compile + install plugins + persist workspace state to platform.db.
/// If `profile_id` is None, re-runs the active profile from platform.db.
pub fn activate_profile(profile_id: Option<&str>, project_root: &Path) -> Result<()> {
    let ship_dir = project_root.join(".ship");
    let mut state = WorkspaceState::load(&ship_dir);
    let id = profile_id
        .map(str::to_string)
        .or_else(|| state.active_profile.clone())
        .context("No profile specified and no active profile set. Run: ship use <profile-id>")?;

    let profile_path = find_profile_file(&id, project_root)
        .with_context(|| format!("Profile '{}' not found in .ship/agents/profiles/", id))?;
    let profile = Profile::load(&profile_path)?;

    run_compile(CompileOptions {
        project_root, provider: None, dry_run: false, active_agent: Some(&id),
    })?;

    let now_plugins: Vec<String> = profile.plugins.install.clone();
    let prev_plugins = state.plugins_installed.clone();
    run_plugin_lifecycle(&now_plugins, &prev_plugins, &profile.plugins.scope);

    state.active_profile = Some(id.clone());
    state.compiled_at = Some(chrono::Utc::now().to_rfc3339());
    state.plugins_installed = now_plugins;
    state.save(&ship_dir)?;
    println!("✓ activated profile '{}'", id);
    Ok(())
}

fn run_plugin_lifecycle(now: &[String], prev: &[String], scope: &str) {
    for plugin in now {
        if !prev.contains(plugin) {
            let status = std::process::Command::new("claude")
                .args(["plugin", "install", plugin, "--scope", scope]).status();
            match status {
                Ok(s) if s.success() => println!("  + plugin {}", plugin),
                Ok(_) => eprintln!("  warning: plugin install failed for {}", plugin),
                Err(_) => eprintln!("  warning: claude CLI not found — skipping plugin install for {}", plugin),
            }
        }
    }
    for plugin in prev {
        if !now.contains(plugin) {
            let status = std::process::Command::new("claude")
                .args(["plugin", "uninstall", plugin]).status();
            match status {
                Ok(s) if s.success() => println!("  - plugin {}", plugin),
                _ => eprintln!("  warning: plugin uninstall failed for {}", plugin),
            }
        }
    }
}

/// Search order: agents/profiles/ → agents/presets/ (compat) → modes/ (legacy), project then global.
pub fn find_profile_file(profile_id: &str, project_root: &Path) -> Option<PathBuf> {
    let ship = project_root.join(".ship");
    let file = format!("{}.toml", profile_id);
    let p = ship.join("agents").join("profiles").join(&file);
    if p.exists() { return Some(p); }
    let p_compat = ship.join("agents").join("presets").join(&file);
    if p_compat.exists() { return Some(p_compat); }
    let m = ship.join("modes").join(&file);
    if m.exists() { return Some(m); }
    let home = dirs::home_dir()?;
    let gp = home.join(".ship").join("agents").join("profiles").join(&file);
    if gp.exists() { return Some(gp); }
    let gp_compat = home.join(".ship").join("agents").join("presets").join(&file);
    if gp_compat.exists() { return Some(gp_compat); }
    let gm = home.join(".ship").join("modes").join(&file);
    if gm.exists() { return Some(gm); }
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
        std::fs::write(ship_dir.join("ship.toml"), "id = \"test-proj-id\"\nname = \"test\"\n").unwrap();
        ship_dir
    }

    #[test]
    fn workspace_state_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let ship_dir = setup_ship_dir(&tmp);
        let mut state = WorkspaceState::default();
        state.active_profile = Some("cli-lane".to_string());
        state.compiled_at = Some("2026-01-01T00:00:00Z".to_string());
        state.plugins_installed = vec!["superpowers@official".to_string()];
        state.save(&ship_dir).unwrap();
        let loaded = WorkspaceState::load(&ship_dir);
        assert_eq!(loaded.active_profile.as_deref(), Some("cli-lane"));
        assert_eq!(loaded.compiled_at.as_deref(), Some("2026-01-01T00:00:00Z"));
        assert_eq!(loaded.plugins_installed, vec!["superpowers@official"]);
    }

    #[test]
    fn workspace_state_load_missing_returns_default() {
        let tmp = TempDir::new().unwrap();
        let ship_dir = setup_ship_dir(&tmp);
        let state = WorkspaceState::load(&ship_dir);
        assert!(state.active_profile.is_none());
        assert!(state.plugins_installed.is_empty());
    }

    #[test]
    fn migrate_ship_lock_active_profile() {
        let tmp = TempDir::new().unwrap();
        let ship_dir = setup_ship_dir(&tmp);
        std::fs::write(
            ship_dir.join("ship.lock"),
            "active_profile = \"cli-lane\"\ncompiled_at = \"2026-01-01T00:00:00Z\"\n\
             \n[plugins]\ninstalled = [\"superpowers@official\"]\n",
        ).unwrap();
        let state = WorkspaceState::load(&ship_dir);
        assert_eq!(state.active_profile.as_deref(), Some("cli-lane"));
        assert_eq!(state.compiled_at.as_deref(), Some("2026-01-01T00:00:00Z"));
        assert_eq!(state.plugins_installed, vec!["superpowers@official"]);
        assert!(!ship_dir.join("ship.lock").exists(), "ship.lock must be deleted after migration");
    }

    #[test]
    fn migrate_ship_lock_active_preset_fallback() {
        let tmp = TempDir::new().unwrap();
        let ship_dir = setup_ship_dir(&tmp);
        std::fs::write(ship_dir.join("ship.lock"), "active_preset = \"cli-lane\"\n").unwrap();
        let state = WorkspaceState::load(&ship_dir);
        assert_eq!(state.active_profile.as_deref(), Some("cli-lane"));
        assert!(!ship_dir.join("ship.lock").exists());
    }

    #[test]
    fn migrate_ship_lock_is_idempotent() {
        let tmp = TempDir::new().unwrap();
        let ship_dir = setup_ship_dir(&tmp);
        std::fs::write(ship_dir.join("ship.lock"), "active_profile = \"web-lane\"\n").unwrap();
        let _ = WorkspaceState::load(&ship_dir); // first load: migrates and deletes
        assert!(!ship_dir.join("ship.lock").exists());
        let state = WorkspaceState::load(&ship_dir); // second load: reads from DB
        assert_eq!(state.active_profile.as_deref(), Some("web-lane"));
    }

    #[test]
    fn registry_ship_lock_not_migrated() {
        let tmp = TempDir::new().unwrap();
        let ship_dir = setup_ship_dir(&tmp);
        let registry_lock = "version = 1\n\n[[package]]\npath = \"github.com/owner/pkg\"\ncommit = \"abc\"\nhash = \"sha256:xyz\"\n";
        std::fs::write(ship_dir.join("ship.lock"), registry_lock).unwrap();
        let _ = WorkspaceState::load(&ship_dir);
        assert!(ship_dir.join("ship.lock").exists(), "registry-format ship.lock must not be deleted");
    }

    #[test]
    fn find_profile_file_finds_new_location() {
        let tmp = TempDir::new().unwrap();
        write_file(tmp.path(), ".ship/agents/profiles/test.toml", "[profile]\nname=\"Test\"\nid=\"test\"\n");
        let found = find_profile_file("test", tmp.path());
        assert!(found.is_some());
        assert!(found.unwrap().to_string_lossy().contains("agents/profiles"));
    }

    #[test]
    fn find_profile_file_falls_back_to_modes() {
        let tmp = TempDir::new().unwrap();
        write_file(tmp.path(), ".ship/modes/legacy.toml", "[mode]\nname=\"Legacy\"\nid=\"legacy\"\n");
        assert!(find_profile_file("legacy", tmp.path()).is_some());
    }

    #[test]
    fn find_profile_file_prefers_profiles_over_modes() {
        let tmp = TempDir::new().unwrap();
        write_file(tmp.path(), ".ship/agents/profiles/both.toml", "[profile]\nname=\"New\"\nid=\"both\"\n");
        write_file(tmp.path(), ".ship/modes/both.toml", "[mode]\nname=\"Old\"\nid=\"both\"\n");
        let found = find_profile_file("both", tmp.path()).unwrap();
        assert!(found.to_string_lossy().contains("agents/profiles"));
    }
}
