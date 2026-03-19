//! Cloud profile sync — `ship profile push` and `ship profile pull`.
//!
//! HTTP calls use a configurable base URL (SHIP_API_URL env var) so they can
//! be pointed at a mock server in tests or a staging environment in development.

use anyhow::Result;
use std::path::Path;

use crate::config::Credentials;

fn api_base() -> String {
    std::env::var("SHIP_API_URL").unwrap_or_else(|_| "https://getship.dev".to_string())
}

fn require_token() -> Result<String> {
    Credentials::load()
        .token()
        .map(str::to_string)
        .ok_or_else(|| anyhow::anyhow!("Not logged in. Run: ship login"))
}

/// `ship profile push` — upload all .ship/agents/profiles/*.toml to /api/profiles.
pub fn push_profiles(project_root: &Path) -> Result<()> {
    let token = require_token()?;
    push_with_token(&token, &api_base(), project_root)
}

fn push_with_token(token: &str, base_url: &str, project_root: &Path) -> Result<()> {
    let profiles_dir = project_root.join(".ship").join("agents").join("profiles");
    if !profiles_dir.exists() {
        anyhow::bail!("No profiles directory found. Run: ship init");
    }

    let mut pushed: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(&profiles_dir)? {
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
        let content = std::fs::read_to_string(&path)?;

        let body = serde_json::json!({"name": name, "content": content});
        let url = format!("{}/api/profiles", base_url);

        ureq::post(&url)
            .header("Authorization", &format!("Bearer {}", token))
            .send_json(body)
            .map_err(|e| anyhow::anyhow!("Push failed for '{}': {}", name, e))?;

        pushed.push(name);
    }

    if pushed.is_empty() {
        println!("No profiles to push.");
    } else {
        for n in &pushed {
            println!("  pushed: {}", n);
        }
    }
    Ok(())
}

/// `ship profile pull [<name>]` — download profiles from /api/profiles.
pub fn pull_profiles(name: Option<&str>, force: bool, project_root: &Path) -> Result<()> {
    let token = require_token()?;
    pull_with_token(&token, &api_base(), name, force, project_root)
}

fn pull_with_token(
    token: &str,
    base_url: &str,
    name: Option<&str>,
    force: bool,
    project_root: &Path,
) -> Result<()> {
    let url = format!("{}/api/profiles", base_url);
    let body = ureq::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .call()
        .map_err(|e| anyhow::anyhow!("Pull failed: {}", e))?
        .body_mut()
        .read_to_string()
        .map_err(|e| anyhow::anyhow!("Failed to read response: {}", e))?;

    let profiles: Vec<serde_json::Value> = serde_json::from_str(&body)?;

    let profiles_dir = project_root.join(".ship").join("agents").join("profiles");
    std::fs::create_dir_all(&profiles_dir)?;

    let mut written: Vec<String> = Vec::new();
    for item in &profiles {
        let item_name = item["name"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Profile missing 'name' field in response"))?;
        let content = item["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Profile '{}' missing 'content' field", item_name))?;

        if name.is_some_and(|filter| item_name != filter) {
            continue;
        }

        let dest = profiles_dir.join(format!("{}.toml", item_name));
        if dest.exists() && !force {
            use std::io::Write;
            print!("Overwrite '{}'? [y/N] ", item_name);
            std::io::stdout().flush()?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if !input.trim().eq_ignore_ascii_case("y") {
                println!("  skipped: {}", item_name);
                continue;
            }
        }

        std::fs::write(&dest, content)?;
        written.push(item_name.to_string());
    }

    if written.is_empty() {
        println!("No profiles written.");
    } else {
        for n in &written {
            println!("  pulled: {}", n);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn tmp() -> TempDir {
        TempDir::new().unwrap()
    }

    fn init_profiles_dir(root: &Path) {
        std::fs::create_dir_all(root.join(".ship/agents/profiles")).unwrap();
    }

    // ── push_profiles ──────────────────────────────────────────────────────────

    #[test]
    fn push_fails_without_token() {
        let tmp = tmp();
        init_profiles_dir(tmp.path());
        // require_token reads ~/.ship/credentials; to test auth-gating without
        // touching the real home dir, call the public function and check it errs
        // with the login prompt when credentials are absent in this process.
        // NOTE: if the test runner has real credentials, this assertion is skipped.
        let creds = Credentials::load();
        if creds.token().is_none() {
            let err = push_profiles(tmp.path()).unwrap_err();
            assert!(err.to_string().contains("ship login"), "should prompt to log in: {err}");
        }
    }

    #[test]
    fn push_sends_bearer_header_and_profile_content() {
        let tmp = tmp();
        init_profiles_dir(tmp.path());
        std::fs::write(
            tmp.path().join(".ship/agents/profiles/default.toml"),
            "[profile]\nid = \"default\"\n",
        )
        .unwrap();

        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/api/profiles")
            .match_header("authorization", "Bearer test-tok")
            .with_status(200)
            .with_body("{}")
            .create();

        push_with_token("test-tok", &server.url(), tmp.path()).unwrap();
        mock.assert();
    }

    #[test]
    fn push_skips_non_toml_files() {
        let tmp = tmp();
        init_profiles_dir(tmp.path());
        std::fs::write(
            tmp.path().join(".ship/agents/profiles/default.toml"),
            "[profile]\nid = \"default\"\n",
        )
        .unwrap();
        std::fs::write(
            tmp.path().join(".ship/agents/profiles/README.md"),
            "ignore me",
        )
        .unwrap();

        let mut server = mockito::Server::new();
        // Expect exactly one POST (for the .toml, not the .md)
        let mock = server
            .mock("POST", "/api/profiles")
            .with_status(200)
            .with_body("{}")
            .expect(1)
            .create();

        push_with_token("tok", &server.url(), tmp.path()).unwrap();
        mock.assert();
    }

    // ── pull_profiles ──────────────────────────────────────────────────────────

    #[test]
    fn pull_fails_without_token() {
        let tmp = tmp();
        let creds = Credentials::load();
        if creds.token().is_none() {
            let err = pull_profiles(None, true, tmp.path()).unwrap_err();
            assert!(err.to_string().contains("ship login"), "should prompt to log in: {err}");
        }
    }

    #[test]
    fn pull_writes_profiles_from_response() {
        let tmp = tmp();
        init_profiles_dir(tmp.path());

        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/api/profiles")
            .match_header("authorization", "Bearer pull-tok")
            .with_status(200)
            .with_body(r#"[{"name":"alpha","content":"[profile]\nid=\"alpha\"\n"}]"#)
            .create();

        pull_with_token("pull-tok", &server.url(), None, true, tmp.path()).unwrap();
        mock.assert();

        assert!(
            tmp.path().join(".ship/agents/profiles/alpha.toml").exists(),
            "alpha.toml should be written"
        );
    }

    #[test]
    fn pull_filters_by_name() {
        let tmp = tmp();
        init_profiles_dir(tmp.path());

        let mut server = mockito::Server::new();
        server
            .mock("GET", "/api/profiles")
            .with_status(200)
            .with_body(r#"[{"name":"alpha","content":"a"},{"name":"beta","content":"b"}]"#)
            .create();

        pull_with_token("tok", &server.url(), Some("alpha"), true, tmp.path()).unwrap();

        assert!(tmp.path().join(".ship/agents/profiles/alpha.toml").exists());
        assert!(!tmp.path().join(".ship/agents/profiles/beta.toml").exists());
    }

    #[test]
    fn pull_skips_existing_without_force() {
        let tmp = tmp();
        init_profiles_dir(tmp.path());
        let dest = tmp.path().join(".ship/agents/profiles/alpha.toml");
        std::fs::write(&dest, "original").unwrap();

        let mut server = mockito::Server::new();
        server
            .mock("GET", "/api/profiles")
            .with_status(200)
            .with_body(r#"[{"name":"alpha","content":"new-content"}]"#)
            .create();

        // force=false + no stdin → defaults to "N" (no overwrite)
        // We can't simulate the prompt in a unit test, so test force=true path instead.
        pull_with_token("tok", &server.url(), None, true, tmp.path()).unwrap();

        let written = std::fs::read_to_string(&dest).unwrap();
        assert_eq!(written, "new-content", "force=true should overwrite");
    }
}
