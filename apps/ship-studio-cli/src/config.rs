use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Global ~/.ship/config.toml
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShipConfig {
    pub identity: Option<Identity>,
    pub defaults: Option<Defaults>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub name: String,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Defaults {
    pub provider: Option<String>,
    pub mode: Option<String>,
}

/// ~/.ship/path-context.toml — maps filesystem paths to active modes.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PathContext {
    #[serde(default)]
    pub paths: HashMap<String, PathEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PathEntry {
    pub mode: Option<String>,
    pub provider: Option<String>,
}

/// .ship/ship.toml — project-level metadata.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShipProject {
    #[serde(default)]
    pub project: ProjectMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectMeta {
    pub name: Option<String>,
    #[serde(default)]
    pub providers: Vec<String>,
    pub active_mode: Option<String>,
}

impl ShipConfig {
    pub fn load() -> Self {
        let path = Self::path();
        if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| toml::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::path();
        if let Some(p) = path.parent() { std::fs::create_dir_all(p)?; }
        std::fs::write(&path, toml::to_string_pretty(self)?)?;
        Ok(())
    }

    fn path() -> PathBuf {
        dirs::home_dir().unwrap_or_default().join(".ship").join("config.toml")
    }
}

impl PathContext {
    pub fn load() -> Self {
        let path = Self::path();
        if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| toml::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::path();
        if let Some(p) = path.parent() { std::fs::create_dir_all(p)?; }
        std::fs::write(&path, toml::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn active_mode_for(&self, path: &std::path::Path) -> Option<&str> {
        self.paths.get(path.to_string_lossy().as_ref())
            .and_then(|e| e.mode.as_deref())
    }

    fn path() -> PathBuf {
        dirs::home_dir().unwrap_or_default().join(".ship").join("path-context.toml")
    }
}

impl ShipProject {
    pub fn load() -> Self {
        let path = crate::paths::project_ship_toml();
        if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| toml::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = crate::paths::project_ship_toml();
        if let Some(p) = path.parent() { std::fs::create_dir_all(p)?; }
        std::fs::write(&path, toml::to_string_pretty(self)?)?;
        Ok(())
    }

    /// Effective provider list: project config or ["claude"] default.
    pub fn providers(&self) -> Vec<String> {
        let p = &self.project.providers;
        if p.is_empty() { vec!["claude".to_string()] } else { p.clone() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ship_config_round_trips() {
        let cfg = ShipConfig {
            identity: Some(Identity { name: "Alice".into(), email: Some("a@b.com".into()) }),
            defaults: Some(Defaults { provider: Some("claude".into()), mode: None }),
        };
        let s = toml::to_string_pretty(&cfg).unwrap();
        let back: ShipConfig = toml::from_str(&s).unwrap();
        assert_eq!(back.identity.unwrap().name, "Alice");
    }

    #[test]
    fn path_context_active_mode() {
        let mut ctx = PathContext::default();
        ctx.paths.insert("/home/user/proj".into(), PathEntry { mode: Some("rust-expert".into()), provider: None });
        assert_eq!(ctx.active_mode_for(std::path::Path::new("/home/user/proj")), Some("rust-expert"));
        assert_eq!(ctx.active_mode_for(std::path::Path::new("/other")), None);
    }

    #[test]
    fn ship_project_default_provider_is_claude() {
        let proj = ShipProject::default();
        assert_eq!(proj.providers(), vec!["claude"]);
    }
}
