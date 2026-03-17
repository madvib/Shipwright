use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Global ~/.ship/config.toml
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShipConfig {
    pub identity: Option<Identity>,
    pub defaults: Option<Defaults>,
    pub worktrees: Option<WorktreesConfig>,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorktreesConfig {
    pub dir: Option<String>,
}

/// ~/.ship/credentials — auth token storage (separate from general config).
///
/// Format:
/// ```toml
/// [account]
/// token = "..."
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Credentials {
    pub account: Option<CredentialsAccount>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CredentialsAccount {
    pub token: Option<String>,
}

impl Credentials {
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
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
        }
        Ok(())
    }

    fn path() -> PathBuf {
        dirs::home_dir().unwrap_or_default().join(".ship").join("credentials")
    }

    pub fn token(&self) -> Option<&str> {
        self.account.as_ref()?.token.as_deref()
    }
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
        // Restrict permissions on Unix — config may contain auth tokens.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
        }
        Ok(())
    }

    fn path() -> PathBuf {
        dirs::home_dir().unwrap_or_default().join(".ship").join("config.toml")
    }

    /// Resolved base directory for git worktrees.
    ///
    /// Returns the path from `[worktrees] dir` (expanding `~`), or falls back
    /// to `~/dev/<project-name>-worktrees/` derived from `project_root`.
    pub fn worktree_base_dir(&self, project_root: &std::path::Path) -> PathBuf {
        if let Some(ref wt) = self.worktrees {
            if let Some(ref dir) = wt.dir {
                let expanded = if dir.starts_with("~/") {
                    dirs::home_dir().unwrap_or_default().join(&dir[2..])
                } else {
                    PathBuf::from(dir)
                };
                return expanded;
            }
        }
        // Fallback: ~/dev/<project>-worktrees/
        let project_name = project_root
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "project".to_string());
        dirs::home_dir()
            .unwrap_or_default()
            .join("dev")
            .join(format!("{}-worktrees", project_name))
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
            worktrees: None,
        };
        let s = toml::to_string_pretty(&cfg).unwrap();
        let back: ShipConfig = toml::from_str(&s).unwrap();
        assert_eq!(back.identity.unwrap().name, "Alice");
    }

    #[test]
    fn credentials_round_trips() {
        let creds = Credentials {
            account: Some(CredentialsAccount { token: Some("tok-abc".into()) }),
        };
        let s = toml::to_string_pretty(&creds).unwrap();
        let back: Credentials = toml::from_str(&s).unwrap();
        assert_eq!(back.token(), Some("tok-abc"));
    }

    #[test]
    fn credentials_token_none_when_empty() {
        let creds = Credentials::default();
        assert_eq!(creds.token(), None);
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

    #[test]
    fn worktree_base_dir_uses_configured_absolute_path() {
        let cfg = ShipConfig {
            worktrees: Some(WorktreesConfig { dir: Some("/custom/worktrees".into()) }),
            ..Default::default()
        };
        let root = std::path::Path::new("/home/user/myproject");
        assert_eq!(cfg.worktree_base_dir(root), std::path::PathBuf::from("/custom/worktrees"));
    }

    #[test]
    fn worktree_base_dir_expands_tilde() {
        let cfg = ShipConfig {
            worktrees: Some(WorktreesConfig { dir: Some("~/dev/worktrees".into()) }),
            ..Default::default()
        };
        let root = std::path::Path::new("/home/user/myproject");
        let result = cfg.worktree_base_dir(root);
        let s = result.to_string_lossy();
        assert!(s.ends_with("dev/worktrees"), "expected path ending in dev/worktrees, got {s}");
        assert!(!s.contains('~'), "tilde should be expanded, got {s}");
    }

    #[test]
    fn worktree_base_dir_fallback_uses_project_name() {
        let cfg = ShipConfig::default();
        let root = std::path::Path::new("/home/user/myproject");
        let result = cfg.worktree_base_dir(root);
        assert!(
            result.to_string_lossy().ends_with("myproject-worktrees"),
            "fallback should end with <project>-worktrees, got {}",
            result.display()
        );
    }
}
