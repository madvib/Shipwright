//! `ship import <source>` — import a profile from a URL, local path, or provider config.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use crate::config::Credentials;
use crate::mcp::{McpEntry, McpFile};
use crate::profile;

/// Run the import command for the given source string.
pub fn run_import(source: &str) -> Result<()> {
    if is_getship_url(source) {
        import_from_url(source)
    } else if is_github_url(source) {
        import_from_github(source)
    } else if Path::new(source).exists() {
        import_from_path(source)
    } else if source.starts_with("http://") || source.starts_with("https://") {
        anyhow::bail!(
            "Unsupported URL: {}\nOnly getship.dev or github.com URLs are supported.",
            source
        )
    } else {
        anyhow::bail!(
            "Source not found: {}\nProvide a getship.dev URL, a github.com URL, or a local file/directory path.",
            source
        )
    }
}

/// Check whether a source string points to github.com.
///
/// Matches `https://github.com/<owner>/<repo>` with optional .git suffix or
/// additional path segments.
fn is_github_url(s: &str) -> bool {
    let s = s.trim_end_matches('/');
    if !s.starts_with("https://github.com/") && !s.starts_with("http://github.com/") {
        return false;
    }
    // Must have at least owner/repo after the host
    let after = s
        .trim_start_matches("https://github.com/")
        .trim_start_matches("http://github.com/");
    let parts: Vec<&str> = after.split('/').filter(|p| !p.is_empty()).collect();
    parts.len() >= 2
}

/// POST the GitHub URL to the Ship API and write returned artifacts to .ship/.
fn import_from_github(url: &str) -> Result<()> {
    import_from_github_with_base(url, &github_api_base())
}

fn github_api_base() -> String {
    std::env::var("SHIP_API_URL").unwrap_or_else(|_| "https://ship-studio.com".to_string())
}

/// Deserialised server response shape.
#[derive(Deserialize)]
struct ImportResult {
    library: Option<ProjectLibraryJson>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct ProjectLibraryJson {
    name: Option<String>,
    #[serde(default)]
    modes: Vec<ModeJson>,
    #[serde(default)]
    rules: Vec<RuleJson>,
    #[serde(default)]
    mcp_servers: Vec<McpServerJson>,
}

#[derive(Deserialize)]
struct ModeJson {
    name: Option<String>,
    #[serde(flatten)]
    extra: serde_json::Value,
}

#[derive(Deserialize)]
struct RuleJson {
    name: Option<String>,
    content: Option<String>,
    #[serde(flatten)]
    _extra: serde_json::Value,
}

#[derive(Deserialize)]
struct McpServerJson {
    id: Option<String>,
    name: Option<String>,
    command: Option<String>,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    env: HashMap<String, String>,
    url: Option<String>,
    #[serde(flatten)]
    _extra: serde_json::Value,
}

fn import_from_github_with_base(url: &str, base_url: &str) -> Result<()> {
    let project_root = std::env::current_dir()?;
    let ship_dir = project_root.join(".ship");
    std::fs::create_dir_all(&ship_dir)?;

    // Load token if available — public repos work without auth
    let creds = Credentials::load();
    let token = creds.token();

    let endpoint = format!("{}/api/github/import", base_url);
    let body = serde_json::json!({ "url": url });

    let mut req = ureq::post(&endpoint).header("Content-Type", "application/json");
    if let Some(tok) = token {
        req = req.header("Authorization", &format!("Bearer {}", tok));
    }

    let raw: String = req
        .send(body.to_string().as_bytes())
        .map_err(|e| {
            anyhow::anyhow!(
                "Could not reach {}. Check your internet connection.\nDetails: {}",
                base_url,
                e
            )
        })?
        .body_mut()
        .read_to_string()
        .context("Failed to read response body")?;

    let result: ImportResult = serde_json::from_str(&raw).map_err(|e| {
        anyhow::anyhow!(
            "Could not parse response from server ({}). Raw response:\n{}\nParse error: {}",
            endpoint,
            raw,
            e
        )
    })?;

    if let Some(err) = result.error {
        if err.contains("not found") || err.contains("404") {
            anyhow::bail!("Repository not found or no .ship config detected");
        }
        anyhow::bail!("Server error: {}", err);
    }

    let library = result
        .library
        .ok_or_else(|| anyhow::anyhow!("Server returned no library data. Raw response:\n{}", raw))?;

    // Extract owner/repo from URL for the summary line
    let repo_slug = extract_github_slug(url).unwrap_or_else(|| url.to_string());

    let profiles_dir = ship_dir.join("agents").join("profiles");
    let rules_dir = ship_dir.join("agents").join("rules");
    std::fs::create_dir_all(&profiles_dir)?;
    std::fs::create_dir_all(&rules_dir)?;

    let mut n_profiles = 0usize;
    for mode in &library.modes {
        let name = mode
            .name
            .clone()
            .unwrap_or_else(|| format!("profile-{}", n_profiles + 1));
        let safe_name = sanitize_filename(&name);
        let dest = profiles_dir.join(format!("{}.toml", safe_name));
        // Serialise the whole mode JSON value as TOML
        let toml_str = json_value_to_toml(&mode.extra, library.name.as_deref(), &name)?;
        std::fs::write(&dest, &toml_str)
            .with_context(|| format!("Failed to write profile {}", dest.display()))?;
        n_profiles += 1;
    }

    let mut n_rules = 0usize;
    for rule in &library.rules {
        let name = rule
            .name
            .clone()
            .unwrap_or_else(|| format!("rule-{}", n_rules + 1));
        let safe_name = sanitize_filename(&name);
        let dest = rules_dir.join(format!("{}.md", safe_name));
        let content = rule.content.clone().unwrap_or_default();
        std::fs::write(&dest, &content)
            .with_context(|| format!("Failed to write rule {}", dest.display()))?;
        n_rules += 1;
    }

    let mut n_mcp = 0usize;
    if !library.mcp_servers.is_empty() {
        let mcp_path = ship_dir.join("agents").join("mcp.toml");
        std::fs::create_dir_all(mcp_path.parent().unwrap())?;
        let mut mcp_file = McpFile::load(&mcp_path)?;

        for srv in &library.mcp_servers {
            let id = srv
                .id
                .clone()
                .unwrap_or_else(|| format!("imported-{}", n_mcp + 1));
            // Skip duplicates
            if mcp_file.servers.iter().any(|s| s.id == id) {
                continue;
            }
            mcp_file.servers.push(McpEntry {
                id: id.clone(),
                name: srv.name.clone(),
                command: srv.command.clone(),
                args: srv.args.clone(),
                env: srv.env.clone(),
                url: srv.url.clone(),
                scope: "project".into(),
                server_type: if srv.url.is_some() {
                    Some("http".into())
                } else {
                    Some("stdio".into())
                },
                disabled: false,
            });
            n_mcp += 1;
        }
        mcp_file.save(&mcp_path)?;
    }

    println!(
        "imported from {}: {} profiles, {} rules, {} MCP servers",
        repo_slug, n_profiles, n_rules, n_mcp
    );
    Ok(())
}

/// Extract `owner/repo` slug from a GitHub URL.
fn extract_github_slug(url: &str) -> Option<String> {
    let after = url
        .trim_start_matches("https://github.com/")
        .trim_start_matches("http://github.com/");
    let parts: Vec<&str> = after.split('/').filter(|p| !p.is_empty()).collect();
    if parts.len() >= 2 {
        let repo = parts[1].trim_end_matches(".git");
        Some(format!("github.com/{}/{}", parts[0], repo))
    } else {
        None
    }
}

/// Replace characters that are not safe in filenames.
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

/// Convert a JSON value to a minimal TOML profile string.
///
/// Since the server returns a full mode JSON, we do a best-effort conversion:
/// wrap in a [profile] table with the name field plus any extra fields that
/// survive serde_json → toml::Value conversion.
fn json_value_to_toml(
    extra: &serde_json::Value,
    _library_name: Option<&str>,
    mode_name: &str,
) -> Result<String> {
    // Build a minimal TOML table with at least the name
    let mut map = toml::map::Map::new();
    map.insert("name".into(), toml::Value::String(mode_name.to_string()));

    // Merge any extra fields from the server response (best effort)
    if let Some(obj) = extra.as_object() {
        for (k, v) in obj {
            if k == "name" { continue; }
            if let Ok(tv) = json_to_toml_value(v) {
                map.insert(k.clone(), tv);
            }
        }
    }

    let root = toml::Value::Table({
        let mut r = toml::map::Map::new();
        r.insert("profile".into(), toml::Value::Table(map));
        r
    });
    toml::to_string_pretty(&root).context("Failed to serialise profile to TOML")
}

fn json_to_toml_value(v: &serde_json::Value) -> Result<toml::Value> {
    match v {
        serde_json::Value::Null => Ok(toml::Value::String(String::new())),
        serde_json::Value::Bool(b) => Ok(toml::Value::Boolean(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(toml::Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(toml::Value::Float(f))
            } else {
                Ok(toml::Value::String(n.to_string()))
            }
        }
        serde_json::Value::String(s) => Ok(toml::Value::String(s.clone())),
        serde_json::Value::Array(arr) => {
            let items: Result<Vec<_>> = arr.iter().map(json_to_toml_value).collect();
            Ok(toml::Value::Array(items?))
        }
        serde_json::Value::Object(obj) => {
            let mut map = toml::map::Map::new();
            for (k, val) in obj {
                if let Ok(tv) = json_to_toml_value(val) {
                    map.insert(k.clone(), tv);
                }
            }
            Ok(toml::Value::Table(map))
        }
    }
}

/// Check whether a source string points to getship.dev.
fn is_getship_url(source: &str) -> bool {
    source.starts_with("https://getship.dev/")
        || source.starts_with("http://getship.dev/")
        || source.starts_with("https://www.getship.dev/")
}

/// Extract profile ID from a getship.dev URL path.
/// Expects paths like `/p/<id>`, `/profiles/<id>`, or just uses the last path segment.
fn extract_profile_id(url: &str) -> Result<String> {
    let path = url
        .split("getship.dev/")
        .nth(1)
        .unwrap_or("")
        .trim_end_matches('/');
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    match segments.as_slice() {
        [_, id, ..] => Ok((*id).to_string()),
        [id] => Ok((*id).to_string()),
        _ => anyhow::bail!("Could not extract profile ID from URL: {}", url),
    }
}

/// Fetch a profile from getship.dev and install it locally.
fn import_from_url(url: &str) -> Result<()> {
    let project_root = std::env::current_dir()?;
    let ship_dir = project_root.join(".ship");
    if !ship_dir.exists() {
        anyhow::bail!(".ship/ not found. Run: ship init");
    }

    let profile_id = extract_profile_id(url)?;

    println!("Fetching profile '{}' from {}...", profile_id, url);

    let body: String = ureq::get(url)
        .header("Accept", "application/json")
        .call()
        .context("Failed to connect to getship.dev. Check your network connection.")?
        .body_mut()
        .read_to_string()
        .context("Failed to read response body")?;

    // Parse JSON response into a ProjectLibrary to validate the shape
    let library: compiler::ProjectLibrary = serde_json::from_str(&body)
        .context("Invalid profile data received from getship.dev")?;

    // Serialize to TOML and write to profiles directory
    let profiles_dir = ship_dir.join("agents").join("profiles");
    std::fs::create_dir_all(&profiles_dir)?;
    let profile_path = profiles_dir.join(format!("{}.toml", profile_id));

    let toml_content = toml::to_string_pretty(&library)
        .context("Failed to serialize profile to TOML")?;
    std::fs::write(&profile_path, &toml_content)?;

    println!("  wrote {}", profile_path.display());

    // Activate the profile immediately
    profile::activate_profile(Some(&profile_id), &project_root)?;

    println!();
    println!("Create an account to sync across machines: ship login");
    Ok(())
}

/// Import from a local file or directory path (stub for provider config detection).
fn import_from_path(source: &str) -> Result<()> {
    let path = Path::new(source);
    if path.is_dir() {
        println!("[import] Local directory import from {} — not yet implemented.", source);
        println!("  This will detect existing provider configs (CLAUDE.md, .cursor/, etc.)");
        println!("  and reverse-import them into .ship/agents/.");
    } else {
        println!("[import] Local file import from {} — not yet implemented.", source);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn is_getship_url_matches_valid_urls() {
        assert!(is_getship_url("https://getship.dev/p/rust-expert"));
        assert!(is_getship_url("https://getship.dev/profiles/cli-lane"));
        assert!(is_getship_url("https://www.getship.dev/p/test"));
        assert!(!is_getship_url("https://example.com/p/test"));
        assert!(!is_getship_url("/some/local/path"));
    }

    #[test]
    fn extract_profile_id_from_url() {
        assert_eq!(
            extract_profile_id("https://getship.dev/p/rust-expert").unwrap(),
            "rust-expert"
        );
        assert_eq!(
            extract_profile_id("https://getship.dev/profiles/cli-lane").unwrap(),
            "cli-lane"
        );
    }

    #[test]
    fn extract_profile_id_single_segment() {
        assert_eq!(
            extract_profile_id("https://getship.dev/my-profile").unwrap(),
            "my-profile"
        );
    }

    #[test]
    fn extract_profile_id_empty_path_fails() {
        assert!(extract_profile_id("https://getship.dev/").is_err());
    }

    // ── GitHub URL detection ──────────────────────────────────────────────────

    #[test]
    fn is_github_url_matches_owner_repo() {
        assert!(is_github_url("https://github.com/acme/my-repo"));
        assert!(is_github_url("https://github.com/acme/my-repo.git"));
        assert!(is_github_url("https://github.com/acme/my-repo/tree/main"));
        assert!(is_github_url("http://github.com/acme/my-repo"));
    }

    #[test]
    fn is_github_url_rejects_incomplete_paths() {
        assert!(!is_github_url("https://github.com/acme"));
        assert!(!is_github_url("https://github.com/"));
        assert!(!is_github_url("https://getship.dev/p/test"));
        assert!(!is_github_url("https://gitlab.com/acme/repo"));
    }

    // ── extract_github_slug ───────────────────────────────────────────────────

    #[test]
    fn extract_github_slug_basic() {
        assert_eq!(
            extract_github_slug("https://github.com/acme/my-repo"),
            Some("github.com/acme/my-repo".to_string())
        );
    }

    #[test]
    fn extract_github_slug_strips_git_suffix() {
        assert_eq!(
            extract_github_slug("https://github.com/acme/my-repo.git"),
            Some("github.com/acme/my-repo".to_string())
        );
    }

    // ── sanitize_filename ─────────────────────────────────────────────────────

    #[test]
    fn sanitize_filename_replaces_spaces_and_slashes() {
        assert_eq!(sanitize_filename("hello world"), "hello-world");
        assert_eq!(sanitize_filename("path/name"), "path-name");
        assert_eq!(sanitize_filename("valid-name_123"), "valid-name_123");
    }

    // ── import_from_github_with_base (mocked server) ──────────────────────────

    fn run_in_tmp<F: FnOnce(&std::path::Path)>(f: F) {
        let tmp = TempDir::new().unwrap();
        let orig = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();
        f(tmp.path());
        std::env::set_current_dir(orig).unwrap();
    }

    #[test]
    fn github_import_writes_profiles_rules_and_mcp() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/api/github/import")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "library": {
                    "name": "test-lib",
                    "modes": [{"name": "rust-expert"}],
                    "rules": [{"name": "no-panics", "content": "Never use unwrap()"}],
                    "mcp_servers": [{"id": "linear", "name": "Linear", "command": "npx", "args": ["-y", "@mcp/linear"]}]
                }
            }"#)
            .create();

        run_in_tmp(|tmp| {
            import_from_github_with_base(
                "https://github.com/acme/test-repo",
                &server.url(),
            )
            .unwrap();

            assert!(tmp.join(".ship/agents/profiles/rust-expert.toml").exists(), "profile written");
            assert!(tmp.join(".ship/agents/rules/no-panics.md").exists(), "rule written");
            let rule = std::fs::read_to_string(tmp.join(".ship/agents/rules/no-panics.md")).unwrap();
            assert_eq!(rule, "Never use unwrap()");
            assert!(tmp.join(".ship/agents/mcp.toml").exists(), "mcp written");
        });

        mock.assert();
    }

    #[test]
    fn github_import_handles_server_error_field() {
        let mut server = mockito::Server::new();
        server
            .mock("POST", "/api/github/import")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"error": "not found"}"#)
            .create();

        run_in_tmp(|_| {
            let err = import_from_github_with_base(
                "https://github.com/acme/missing-repo",
                &server.url(),
            )
            .unwrap_err();
            assert!(
                err.to_string().contains("not found") || err.to_string().contains("Repository"),
                "unexpected error: {err}"
            );
        });
    }

    #[test]
    fn github_import_skips_duplicate_mcp_servers() {
        let mut server = mockito::Server::new();
        server
            .mock("POST", "/api/github/import")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "library": {
                    "mcp_servers": [{"id": "linear", "command": "npx", "args": []}]
                }
            }"#)
            .expect(2)
            .create();

        run_in_tmp(|tmp| {
            let base = server.url();
            import_from_github_with_base("https://github.com/acme/repo", &base).unwrap();
            // Second call should not duplicate the MCP entry
            import_from_github_with_base("https://github.com/acme/repo", &base).unwrap();

            let mcp_path = tmp.join(".ship/agents/mcp.toml");
            let mcp = McpFile::load(&mcp_path).unwrap();
            assert_eq!(mcp.servers.len(), 1, "duplicate MCP entry should be skipped");
        });
    }
}
