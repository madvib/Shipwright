//! Preset activation — `ship use [<preset-id>]`.
//! Loads preset, compiles, installs plugins, writes ship.lock.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::compile::{CompileOptions, run_compile};
use crate::mode::Preset;

// ── ship.lock ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShipLock {
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
    pub fn load(ship_dir: &Path) -> Self {
        let path = ship_dir.join("ship.lock");
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, ship_dir: &Path) -> Result<()> {
        let path = ship_dir.join("ship.lock");
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

// ── Activation ────────────────────────────────────────────────────────────────

/// Activate a preset: compile + install plugins + write ship.lock.
/// If `preset_id` is None, re-runs the active preset from ship.lock.
pub fn activate_preset(preset_id: Option<&str>, project_root: &Path) -> Result<()> {
    let ship_dir = project_root.join(".ship");
    let mut lock = ShipLock::load(&ship_dir);

    // Resolve preset ID
    let id = preset_id
        .map(str::to_string)
        .or_else(|| lock.active_preset.clone())
        .context("No preset specified and no active preset in ship.lock. Run: ship use <preset-id>")?;

    // Locate preset file
    let preset_path = find_preset_file(&id, project_root)
        .with_context(|| format!("Preset '{}' not found in .ship/agents/presets/", id))?;

    let preset = Preset::load(&preset_path)?;

    // Compile for all providers
    run_compile(CompileOptions {
        project_root,
        provider: None,
        dry_run: false,
        active_mode: Some(&id),
    })?;

    // Plugin lifecycle
    let now_plugins: Vec<String> = preset.plugins.install.clone();
    let prev_plugins = lock.plugins.installed.clone();
    run_plugin_lifecycle(&now_plugins, &prev_plugins, &preset.plugins.scope);

    // Update ship.lock
    lock.active_preset = Some(id.clone());
    lock.compiled_at = Some(chrono::Utc::now().to_rfc3339());
    lock.plugins.installed = now_plugins;
    lock.save(&ship_dir)?;

    println!("✓ activated preset '{}'", id);
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

/// Search order: agents/presets/ (new) → modes/ (legacy), project then global.
pub fn find_preset_file(preset_id: &str, project_root: &Path) -> Option<PathBuf> {
    let ship = project_root.join(".ship");
    let file = format!("{}.toml", preset_id);

    let p = ship.join("agents").join("presets").join(&file);
    if p.exists() { return Some(p); }
    let m = ship.join("modes").join(&file);
    if m.exists() { return Some(m); }

    let home = dirs::home_dir()?;
    let gp = home.join(".ship").join("agents").join("presets").join(&file);
    if gp.exists() { return Some(gp); }
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
    fn ship_lock_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        let mut lock = ShipLock::default();
        lock.active_preset = Some("cli-lane".to_string());
        lock.plugins.installed = vec!["superpowers@official".to_string()];
        lock.save(&ship_dir).unwrap();
        let loaded = ShipLock::load(&ship_dir);
        assert_eq!(loaded.active_preset.as_deref(), Some("cli-lane"));
        assert_eq!(loaded.plugins.installed, vec!["superpowers@official"]);
    }

    #[test]
    fn ship_lock_load_missing_returns_default() {
        let tmp = TempDir::new().unwrap();
        let lock = ShipLock::load(tmp.path());
        assert!(lock.active_preset.is_none());
        assert!(lock.plugins.installed.is_empty());
    }

    #[test]
    fn find_preset_file_finds_new_location() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), ".ship/agents/presets/test.toml", "[preset]\nname=\"Test\"\nid=\"test\"\n");
        let found = find_preset_file("test", tmp.path());
        assert!(found.is_some());
        assert!(found.unwrap().to_string_lossy().contains("agents/presets"));
    }

    #[test]
    fn find_preset_file_falls_back_to_modes() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), ".ship/modes/legacy.toml", "[mode]\nname=\"Legacy\"\nid=\"legacy\"\n");
        let found = find_preset_file("legacy", tmp.path());
        assert!(found.is_some());
    }

    #[test]
    fn find_preset_file_prefers_presets_over_modes() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), ".ship/agents/presets/both.toml", "[preset]\nname=\"New\"\nid=\"both\"\n");
        write(tmp.path(), ".ship/modes/both.toml", "[mode]\nname=\"Old\"\nid=\"both\"\n");
        let found = find_preset_file("both", tmp.path()).unwrap();
        assert!(found.to_string_lossy().contains("agents/presets"));
    }
}
