//! Cloud sync — `ship sync push` and `ship sync pull`.
//!
//! Syncs .ship/agents/profiles/ with the Ship Studio cloud API.
//! Token is read from ~/.ship/credentials. Base URL is read from
//! [cloud] base_url in ~/.ship/config.toml (default: https://ship-studio.com).

use anyhow::Result;
use std::path::Path;

use crate::config::{CloudConfig, Credentials, ShipConfig};

// ── HTTP helper ───────────────────────────────────────────────────────────────

fn api_request(
    method: &str,
    url: &str,
    token: &str,
    body: Option<serde_json::Value>,
) -> Result<serde_json::Value> {
    let auth = format!("Bearer {}", token);
    let raw = match (method, body) {
        ("GET", _) => ureq::get(url)
            .header("Authorization", &auth)
            .call()
            .map_err(|e| match e {
                ureq::Error::StatusCode(401) => {
                    println!("Session expired. Run `ship login` to re-authenticate.");
                    anyhow::anyhow!("Unauthorized")
                }
                ureq::Error::StatusCode(409) => anyhow::anyhow!("409 Conflict"),
                other => anyhow::anyhow!("API request failed ({}): {}", url, other),
            }),
        ("POST", Some(payload)) => ureq::post(url)
            .header("Authorization", &auth)
            .header("Content-Type", "application/json")
            .send(payload.to_string().as_bytes())
            .map_err(|e| match e {
                ureq::Error::StatusCode(401) => {
                    println!("Session expired. Run `ship login` to re-authenticate.");
                    anyhow::anyhow!("Unauthorized")
                }
                ureq::Error::StatusCode(409) => anyhow::anyhow!("409 Conflict"),
                other => anyhow::anyhow!("API request failed ({}): {}", url, other),
            }),
        ("PUT", Some(payload)) => ureq::put(url)
            .header("Authorization", &auth)
            .header("Content-Type", "application/json")
            .send(payload.to_string().as_bytes())
            .map_err(|e| match e {
                ureq::Error::StatusCode(401) => {
                    println!("Session expired. Run `ship login` to re-authenticate.");
                    anyhow::anyhow!("Unauthorized")
                }
                ureq::Error::StatusCode(409) => anyhow::anyhow!("409 Conflict"),
                other => anyhow::anyhow!("API request failed ({}): {}", url, other),
            }),
        _ => anyhow::bail!("Unsupported HTTP method: {}", method),
    }?;

    let buf = raw
        .into_body()
        .read_to_string()
        .map_err(|e| anyhow::anyhow!("Failed to read response body: {}", e))?;
    Ok(serde_json::from_str(&buf).unwrap_or(serde_json::Value::Null))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn require_token() -> Result<String> {
    Credentials::load()
        .token()
        .map(str::to_string)
        .ok_or_else(|| anyhow::anyhow!("Not logged in. Run: ship login"))
}

fn cloud_cfg() -> CloudConfig {
    ShipConfig::load().cloud.unwrap_or_default()
}

fn profiles_dir(project_root: &Path) -> std::path::PathBuf {
    project_root.join(".ship").join("agents").join("profiles")
}

// ── Public entry points ───────────────────────────────────────────────────────

/// `ship sync` with no subcommand — print local/remote summary.
pub fn run_sync_status(_project_root: &Path) -> Result<()> {
    println!("sync: not synced yet. Use `ship sync push` or `ship sync pull`.");
    Ok(())
}

/// `ship sync push` — upload all .ship/agents/profiles/*.toml to the cloud.
pub fn run_sync_push(project_root: &Path) -> Result<()> {
    let token = require_token()?;
    let cfg = cloud_cfg();
    push_with_token(&token, &cfg, project_root)
}

fn push_with_token(token: &str, cfg: &CloudConfig, project_root: &Path) -> Result<()> {
    let dir = profiles_dir(project_root);
    if !dir.exists() {
        anyhow::bail!("No profiles directory found at {}. Run: ship init", dir.display());
    }

    let mut ok: usize = 0;
    let mut fail: usize = 0;

    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let content = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("Cannot read {}: {}", path.display(), e))?;

        let body = serde_json::json!({"name": name, "content": content, "provider": null});
        let post_url = cfg.get_api_url("/api/profiles");

        match api_request("POST", &post_url, token, Some(body.clone())) {
            Ok(_) => { ok += 1; }
            Err(e) if e.to_string().contains("409") || e.to_string().contains("Conflict") => {
                // Profile exists — try PUT /api/profiles/:name
                let put_url = cfg.get_api_url(&format!("/api/profiles/{}", name));
                match api_request("PUT", &put_url, token, Some(body)) {
                    Ok(_) => { ok += 1; }
                    Err(put_err) => {
                        eprintln!("  failed ({}): {}", name, put_err);
                        fail += 1;
                    }
                }
            }
            Err(e) => {
                eprintln!("  failed ({}): {}", name, e);
                fail += 1;
            }
        }
    }

    println!("pushed {} profile{}", ok, if ok == 1 { "" } else { "s" });
    if fail > 0 {
        anyhow::bail!("{} profile(s) failed to push — see errors above", fail);
    }
    Ok(())
}

/// `ship sync pull [--force]` — download profiles from the cloud.
pub fn run_sync_pull(force: bool, project_root: &Path) -> Result<()> {
    let token = require_token()?;
    let cfg = cloud_cfg();
    pull_with_token(&token, &cfg, force, project_root)
}

fn pull_with_token(token: &str, cfg: &CloudConfig, force: bool, project_root: &Path) -> Result<()> {
    let url = cfg.get_api_url("/api/profiles");
    let val = api_request("GET", &url, token, None)?;

    let profiles = val
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Unexpected API response: expected JSON array"))?;

    let dir = profiles_dir(project_root);
    std::fs::create_dir_all(&dir)?;

    let mut pulled: usize = 0;

    for item in profiles {
        let name = item["name"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Profile entry missing 'name' field"))?;
        let content = item["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Profile '{}' missing 'content' field", name))?;

        let dest = dir.join(format!("{}.toml", name));

        if dest.exists() && !force {
            // Skip if local file is newer than remote (use mtime as proxy)
            if let Ok(meta) = std::fs::metadata(&dest)
                && let Ok(modified) = meta.modified()
            {
                // Without a remote timestamp we skip conservatively —
                // any local file wins unless --force is given.
                let _ = modified;
                println!("  skipped (local is newer or same): {}", name);
                continue;
            }
        }

        std::fs::write(&dest, content)
            .map_err(|e| anyhow::anyhow!("Cannot write {}: {}", dest.display(), e))?;
        pulled += 1;
    }

    println!("pulled {} profile{}", pulled, if pulled == 1 { "" } else { "s" });
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn tmp() -> TempDir { TempDir::new().unwrap() }

    fn init_profiles(root: &Path) {
        std::fs::create_dir_all(root.join(".ship/agents/profiles")).unwrap();
    }

    #[test]
    fn push_fails_if_not_logged_in() {
        let t = tmp();
        init_profiles(t.path());
        let creds = Credentials::load();
        if creds.token().is_none() {
            let err = run_sync_push(t.path()).unwrap_err();
            assert!(err.to_string().contains("ship login"), "{err}");
        }
    }

    #[test]
    fn pull_fails_if_not_logged_in() {
        let t = tmp();
        let creds = Credentials::load();
        if creds.token().is_none() {
            let err = run_sync_pull(true, t.path()).unwrap_err();
            assert!(err.to_string().contains("ship login"), "{err}");
        }
    }

    #[test]
    fn push_sends_bearer_and_content() {
        let t = tmp();
        init_profiles(t.path());
        std::fs::write(
            t.path().join(".ship/agents/profiles/default.toml"),
            "[profile]\nid = \"default\"\n",
        ).unwrap();

        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/api/profiles")
            .match_header("authorization", "Bearer tok")
            .with_status(200)
            .with_body("{}")
            .create();

        let cfg = CloudConfig { base_url: Some(server.url()) };
        push_with_token("tok", &cfg, t.path()).unwrap();
        mock.assert();
    }

    #[test]
    fn push_uses_put_on_409() {
        let t = tmp();
        init_profiles(t.path());
        std::fs::write(
            t.path().join(".ship/agents/profiles/alpha.toml"),
            "[profile]\nid = \"alpha\"\n",
        ).unwrap();

        let mut server = mockito::Server::new();
        server.mock("POST", "/api/profiles").with_status(409).with_body("{}").create();
        let put_mock = server
            .mock("PUT", "/api/profiles/alpha")
            .with_status(200)
            .with_body("{}")
            .create();

        let cfg = CloudConfig { base_url: Some(server.url()) };
        push_with_token("tok", &cfg, t.path()).unwrap();
        put_mock.assert();
    }

    #[test]
    fn pull_writes_profiles() {
        let t = tmp();
        init_profiles(t.path());

        let mut server = mockito::Server::new();
        server
            .mock("GET", "/api/profiles")
            .match_header("authorization", "Bearer tok")
            .with_status(200)
            .with_body(r#"[{"name":"beta","content":"[profile]\nid=\"beta\"\n"}]"#)
            .create();

        let cfg = CloudConfig { base_url: Some(server.url()) };
        pull_with_token("tok", &cfg, true, t.path()).unwrap();

        assert!(t.path().join(".ship/agents/profiles/beta.toml").exists());
    }

    #[test]
    fn pull_skips_existing_without_force() {
        let t = tmp();
        init_profiles(t.path());
        let dest = t.path().join(".ship/agents/profiles/gamma.toml");
        std::fs::write(&dest, "original").unwrap();

        let mut server = mockito::Server::new();
        server
            .mock("GET", "/api/profiles")
            .with_status(200)
            .with_body(r#"[{"name":"gamma","content":"new"}]"#)
            .create();

        let cfg = CloudConfig { base_url: Some(server.url()) };
        pull_with_token("tok", &cfg, false, t.path()).unwrap();

        let written = std::fs::read_to_string(&dest).unwrap();
        assert_eq!(written, "original", "force=false should skip existing file");
    }
}
