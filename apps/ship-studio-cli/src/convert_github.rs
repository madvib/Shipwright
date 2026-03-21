//! GitHub-specific conversion: POST to Ship API, write agents/rules/MCP to .ship/.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;

use crate::config::Credentials;
use crate::mcp::{McpEntry, McpFile};

// ── Types ─────────────────────────────────────────────────────────────────────

/// Deserialised server response shape.
#[derive(Deserialize)]
pub(crate) struct ConvertResult {
    pub library: Option<ProjectLibraryJson>,
    pub error: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct ProjectLibraryJson {
    pub name: Option<String>,
    #[serde(default)]
    pub modes: Vec<ModeJson>,
    #[serde(default)]
    pub rules: Vec<RuleJson>,
    #[serde(default)]
    pub mcp_servers: Vec<McpServerJson>,
}

#[derive(Deserialize)]
pub(crate) struct ModeJson {
    pub name: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Deserialize)]
pub(crate) struct RuleJson {
    pub name: Option<String>,
    pub content: Option<String>,
    #[serde(flatten)]
    pub _extra: serde_json::Value,
}

#[derive(Deserialize)]
pub(crate) struct McpServerJson {
    pub id: Option<String>,
    pub name: Option<String>,
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    pub url: Option<String>,
    #[serde(flatten)]
    pub _extra: serde_json::Value,
}

// ── URL helpers ───────────────────────────────────────────────────────────────

/// Check whether a source string points to github.com.
pub fn is_github_url(s: &str) -> bool {
    let s = s.trim_end_matches('/');
    if !s.starts_with("https://github.com/") && !s.starts_with("http://github.com/") {
        return false;
    }
    let after = s
        .trim_start_matches("https://github.com/")
        .trim_start_matches("http://github.com/");
    let parts: Vec<&str> = after.split('/').filter(|p| !p.is_empty()).collect();
    parts.len() >= 2
}

/// Extract `owner/repo` slug from a GitHub URL.
pub fn extract_github_slug(url: &str) -> Option<String> {
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

pub fn github_api_base() -> String {
    std::env::var("SHIP_API_URL").unwrap_or_else(|_| "https://ship-studio.com".to_string())
}

// ── Shared helpers ────────────────────────────────────────────────────────────

/// Replace characters that are not safe in filenames.
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

// ── Conversion entry point ────────────────────────────────────────────────────

/// POST the GitHub URL to the Ship API and write returned artifacts to .ship/.
pub fn convert_from_github(url: &str) -> Result<()> {
    convert_from_github_with_base(url, &github_api_base())
}

pub fn convert_from_github_with_base(url: &str, base_url: &str) -> Result<()> {
    let project_root = std::env::current_dir()?;
    let ship_dir = project_root.join(".ship");
    std::fs::create_dir_all(&ship_dir)?;

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

    let result: ConvertResult = serde_json::from_str(&raw).map_err(|e| {
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

    let repo_slug = extract_github_slug(url).unwrap_or_else(|| url.to_string());

    let agents_out_dir = ship_dir.join("agents");
    let rules_dir = ship_dir.join("agents").join("rules");
    std::fs::create_dir_all(&agents_out_dir)?;
    std::fs::create_dir_all(&rules_dir)?;

    let mut n_agents = 0usize;
    for mode in &library.modes {
        let name = mode
            .name
            .clone()
            .unwrap_or_else(|| format!("agent-{}", n_agents + 1));
        let safe_name = sanitize_filename(&name);
        let dest = agents_out_dir.join(format!("{}.toml", safe_name));
        let toml_str = json_value_to_toml(&mode.extra, library.name.as_deref(), &name)?;
        std::fs::write(&dest, &toml_str)
            .with_context(|| format!("Failed to write agent {}", dest.display()))?;
        n_agents += 1;
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
        "converted from {}: {} agents, {} rules, {} MCP servers",
        repo_slug, n_agents, n_rules, n_mcp
    );
    Ok(())
}

// ── JSON-to-TOML conversion ──────────────────────────────────────────────────

/// Convert a JSON value to a minimal TOML agent string.
fn json_value_to_toml(
    extra: &serde_json::Value,
    _library_name: Option<&str>,
    mode_name: &str,
) -> Result<String> {
    let mut map = toml::map::Map::new();
    map.insert("name".into(), toml::Value::String(mode_name.to_string()));

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
        r.insert("agent".into(), toml::Value::Table(map));
        r
    });
    toml::to_string_pretty(&root).context("Failed to serialise agent to TOML")
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
