//! `ship import <source>` — import a profile from a URL, local path, or provider config.

use anyhow::{Context, Result};
use std::path::Path;

use crate::profile;

/// Run the import command for the given source string.
pub fn run_import(source: &str) -> Result<()> {
    if is_getship_url(source) {
        import_from_url(source)
    } else if Path::new(source).exists() {
        import_from_path(source)
    } else if source.starts_with("http://") || source.starts_with("https://") {
        anyhow::bail!(
            "Unsupported URL: {}\nOnly getship.dev URLs are supported (e.g. https://getship.dev/p/<id>)",
            source
        )
    } else {
        anyhow::bail!(
            "Source not found: {}\nProvide a getship.dev URL or a local file/directory path.",
            source
        )
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
}
