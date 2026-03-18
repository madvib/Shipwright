//! Profile activation — `ship use [<profile-id>]`.
//! Loads profile, compiles, installs plugins, writes ship.state.
//!
//! NOTE: ship.state (workspace state) is distinct from ship.lock (registry deps).
//! ship.state contains: active_profile, compiled_at, plugins.
//! ship.lock contains: version=1, [[package]] entries from the registry.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::compile::{CompileOptions, run_compile};
use crate::mode::Profile;

// ── ship.state ────────────────────────────────────────────────────────────────

/// Workspace state written to `.ship/ship.state`.
/// Tracks the active profile, last compile time, and installed plugins.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShipLock {
    pub active_profile: Option<String>,
    /// Legacy field — migrated to `active_profile` on first load.
    #[serde(default, skip_serializing)]
    pub active_preset: Option<String>,
    pub compiled_at: Option<String>,
    #[serde(default)]
    pub plugins: LockPlugins,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LockPlugins {
    #[serde(default)]
    pub installed: Vec<String>,
}

impl ShipLock {
    /// Load workspace state from `.ship/ship.state`.
    ///
    /// Migration: if `ship.state` is absent but `ship.lock` exists and contains
    /// `active_profile`, the file is renamed to `ship.state` automatically.
    pub fn load(ship_dir: &Path) -> Self {
        let state_path = ship_dir.join("ship.state");
        let lock_path = ship_dir.join("ship.lock");

        // Auto-migrate: ship.lock (workspace state) → ship.state
        // Only migrate when ship.lock looks like workspace state (has active_profile key)
        // and ship.state doesn't yet exist.
        if !state_path.exists() && lock_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&lock_path) {
                let is_workspace_state = content.contains("active_profile")
                    || content.contains("active_preset");
                if is_workspace_state {
                    if let Ok(()) = std::fs::rename(&lock_path, &state_path) {
                        println!("migrated .ship/ship.lock -> .ship/ship.state");
                    }
                }
            }
        }

        let mut lock: ShipLock = std::fs::read_to_string(&state_path)
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default();
        // Migration: active_preset → active_profile
        if lock.active_profile.is_none() {
            if let Some(legacy) = lock.active_preset.take() {
                lock.active_profile = Some(legacy);
            }
        }
        lock
    }

    pub fn save(&self, ship_dir: &Path) -> Result<()> {
        let path = ship_dir.join("ship.state");
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

// ── Activation ────────────────────────────────────────────────────────────────

/// Activate a profile: compile + install plugins + write ship.state.
/// If `profile_id` is None, re-runs the active profile from ship.state.
pub fn activate_profile(profile_id: Option<&str>, project_root: &Path) -> Result<()> {
    let ship_dir = project_root.join(".ship");
    let mut lock = ShipLock::load(&ship_dir);

    // Resolve profile ID
    let id = profile_id
        .map(str::to_string)
        .or_else(|| lock.active_profile.clone())
        .context("No profile specified and no active profile in ship.state. Run: ship use <profile-id>")?;

    // Locate profile file
    let profile_path = find_profile_file(&id, project_root)
        .with_context(|| format!("Profile '{}' not found in .ship/agents/profiles/", id))?;

    let profile = Profile::load(&profile_path)?;

    // Compile for all providers
    run_compile(CompileOptions {
        project_root,
        provider: None,
        dry_run: false,
        active_mode: Some(&id),
    })?;

    // Plugin lifecycle
    let now_plugins: Vec<String> = profile.plugins.install.clone();
    let prev_plugins = lock.plugins.installed.clone();
    run_plugin_lifecycle(&now_plugins, &prev_plugins, &profile.plugins.scope);

    // Update ship.lock
    lock.active_profile = Some(id.clone());
    lock.compiled_at = Some(chrono::Utc::now().to_rfc3339());
    lock.plugins.installed = now_plugins;
    lock.save(&ship_dir)?;

    println!("✓ activated profile '{}'", id);
    Ok(())
}

/// Delta plugin installs/uninstalls via `claude plugin` CLI.
fn run_plugin_lifecycle(now: &[String], prev: &[String], scope: &str) {
    for plugin in now {
        if !prev.contains(plugin) {
            let status = std::process::Command::new("claude")
                .args(["plugin", "install", plugin, "--scope", scope])
                .status();
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
                .args(["plugin", "uninstall", plugin])
                .status();
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

    fn write(dir: &Path, rel: &str, content: &str) {
        let p = dir.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, content).unwrap();
    }

    #[test]
    fn ship_state_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        let mut lock = ShipLock::default();
        lock.active_profile = Some("cli-lane".to_string());
        lock.plugins.installed = vec!["superpowers@official".to_string()];
        lock.save(&ship_dir).unwrap();
        // Verify it wrote to ship.state not ship.lock
        assert!(ship_dir.join("ship.state").exists());
        assert!(!ship_dir.join("ship.lock").exists());
        let loaded = ShipLock::load(&ship_dir);
        assert_eq!(loaded.active_profile.as_deref(), Some("cli-lane"));
        assert_eq!(loaded.plugins.installed, vec!["superpowers@official"]);
    }

    #[test]
    fn ship_state_load_missing_returns_default() {
        let tmp = TempDir::new().unwrap();
        let lock = ShipLock::load(tmp.path());
        assert!(lock.active_profile.is_none());
        assert!(lock.plugins.installed.is_empty());
    }

    #[test]
    fn ship_state_migrates_active_preset_to_active_profile() {
        let tmp = TempDir::new().unwrap();
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        // Write old-format ship.state with active_preset
        std::fs::write(
            ship_dir.join("ship.state"),
            "active_preset = \"cli-lane\"\n",
        ).unwrap();
        let loaded = ShipLock::load(&ship_dir);
        assert_eq!(loaded.active_profile.as_deref(), Some("cli-lane"));
    }

    #[test]
    fn ship_state_migrates_from_old_ship_lock_file() {
        let tmp = TempDir::new().unwrap();
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        // Write old ship.lock with workspace state content
        std::fs::write(
            ship_dir.join("ship.lock"),
            "active_profile = \"my-profile\"\ncompiled_at = \"2026-01-01T00:00:00Z\"\n",
        ).unwrap();
        // ship.state does not exist yet
        assert!(!ship_dir.join("ship.state").exists());

        let loaded = ShipLock::load(&ship_dir);
        assert_eq!(loaded.active_profile.as_deref(), Some("my-profile"));
        // ship.lock should have been renamed to ship.state
        assert!(ship_dir.join("ship.state").exists());
        assert!(!ship_dir.join("ship.lock").exists());
    }

    #[test]
    fn ship_state_does_not_migrate_registry_ship_lock() {
        let tmp = TempDir::new().unwrap();
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        // Write a registry ship.lock (no active_profile key)
        std::fs::write(
            ship_dir.join("ship.lock"),
            "version = 1\n\n[[package]]\npath = \"github.com/a/b\"\nversion = \"v1.0.0\"\ncommit = \"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"\nhash = \"sha256:abc\"\n",
        ).unwrap();
        // ship.state does not exist yet
        assert!(!ship_dir.join("ship.state").exists());

        let loaded = ShipLock::load(&ship_dir);
        // Should be default (no profile) — ship.lock was not migrated
        assert!(loaded.active_profile.is_none());
        // Registry ship.lock should remain untouched
        assert!(ship_dir.join("ship.lock").exists());
        assert!(!ship_dir.join("ship.state").exists());
    }

    #[test]
    fn find_profile_file_finds_new_location() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), ".ship/agents/profiles/test.toml", "[profile]\nname=\"Test\"\nid=\"test\"\n");
        let found = find_profile_file("test", tmp.path());
        assert!(found.is_some());
        assert!(found.unwrap().to_string_lossy().contains("agents/profiles"));
    }

    #[test]
    fn find_profile_file_falls_back_to_modes() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), ".ship/modes/legacy.toml", "[mode]\nname=\"Legacy\"\nid=\"legacy\"\n");
        let found = find_profile_file("legacy", tmp.path());
        assert!(found.is_some());
    }

    #[test]
    fn find_profile_file_prefers_presets_over_modes() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), ".ship/agents/profiles/both.toml", "[profile]\nname=\"New\"\nid=\"both\"\n");
        write(tmp.path(), ".ship/modes/both.toml", "[mode]\nname=\"Old\"\nid=\"both\"\n");
        let found = find_profile_file("both", tmp.path()).unwrap();
        assert!(found.to_string_lossy().contains("agents/profiles"));
    }
}
