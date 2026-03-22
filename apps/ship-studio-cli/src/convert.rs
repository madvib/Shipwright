//! `ship convert <source>` — convert provider config files (CLAUDE.md, .cursor/) into .ship/ format.

use anyhow::{Context, Result};
use std::path::Path;

use crate::profile;

#[path = "convert_github.rs"]
pub(crate) mod convert_github;

/// Run the convert command for the given source string.
pub fn run_convert(source: &str) -> Result<()> {
    if is_getship_url(source) {
        convert_from_url(source)
    } else if convert_github::is_github_url(source) {
        convert_github::convert_from_github(source)
    } else if Path::new(source).exists() {
        convert_from_path(source)
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

// ── getship.dev URL helpers ──────────────────────────────────────────────────

/// Check whether a source string points to getship.dev.
fn is_getship_url(source: &str) -> bool {
    source.starts_with("https://getship.dev/")
        || source.starts_with("http://getship.dev/")
        || source.starts_with("https://www.getship.dev/")
}

/// Extract agent ID from a getship.dev URL path.
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
        _ => anyhow::bail!("Could not extract agent ID from URL: {}", url),
    }
}

/// Fetch an agent from getship.dev and install it locally.
fn convert_from_url(url: &str) -> Result<()> {
    let project_root = std::env::current_dir()?;
    let ship_dir = project_root.join(".ship");
    if !ship_dir.exists() {
        anyhow::bail!(".ship/ not found. Run: ship init");
    }

    let agent_id = extract_profile_id(url)?;

    println!("Fetching agent '{}' from {}...", agent_id, url);

    let body: String = ureq::get(url)
        .header("Accept", "application/json")
        .call()
        .context("Failed to connect to getship.dev. Check your network connection.")?
        .body_mut()
        .read_to_string()
        .context("Failed to read response body")?;

    // Parse JSON response into a ProjectLibrary to validate the shape
    let library: compiler::ProjectLibrary =
        serde_json::from_str(&body).context("Invalid agent data received from getship.dev")?;

    // Serialize to TOML and write to agents directory
    let agents_out_dir = ship_dir.join("agents");
    std::fs::create_dir_all(&agents_out_dir)?;
    let agent_path = agents_out_dir.join(format!("{}.toml", agent_id));

    let toml_content =
        toml::to_string_pretty(&library).context("Failed to serialize agent to TOML")?;
    std::fs::write(&agent_path, &toml_content)?;

    println!("  wrote {}", agent_path.display());

    // Activate the agent immediately
    profile::activate_agent(Some(&agent_id), &project_root)?;

    println!();
    println!("Create an account to sync across machines: ship login");
    Ok(())
}

/// Convert from a local file or directory path (provider config detection).
fn convert_from_path(source: &str) -> Result<()> {
    let path = Path::new(source);
    if path.is_dir() {
        println!(
            "[convert] Local directory conversion from {} — not yet implemented.",
            source
        );
        println!("  This will detect existing provider configs (CLAUDE.md, .cursor/, etc.)");
        println!("  and convert them into .ship/agents/.");
    } else {
        println!(
            "[convert] Local file conversion from {} — not yet implemented.",
            source
        );
    }
    Ok(())
}

#[cfg(test)]
#[path = "convert_tests.rs"]
mod tests;
