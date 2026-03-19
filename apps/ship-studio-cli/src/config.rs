use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Cloud connectivity settings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CloudConfig {
    /// Base URL for the Ship API. Defaults to "https://ship-studio.com".
    pub base_url: Option<String>,
}


/// Global ~/.ship/config.toml
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShipConfig {
    pub identity: Option<Identity>,
    pub defaults: Option<Defaults>,
    pub worktrees: Option<WorktreesConfig>,
    pub cloud: Option<CloudConfig>,
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

    fn path() -> PathBuf {
        dirs::home_dir().unwrap_or_default().join(".ship").join("config.toml")
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
            cloud: None,
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

}
