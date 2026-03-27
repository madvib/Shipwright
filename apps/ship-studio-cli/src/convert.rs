//! `ship convert <source>` — convert provider config files (CLAUDE.md, .cursor/) into .ship/ format.

use anyhow::{Context, Result};
use std::path::Path;

use crate::profile;

#[path = "convert_github.rs"]
pub(crate) mod convert_github;
#[path = "convert_mcp.rs"]
mod convert_mcp;

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
    profile::activate_agent(Some(&agent_id), &project_root, None)?;

    println!();
    println!("Create an account to sync across machines: ship login");
    Ok(())
}

/// Convert from a local file or directory path (provider config detection).
fn convert_from_path(source: &str) -> Result<()> {
    let path = Path::new(source).canonicalize().context("Invalid path")?;

    if !path.is_dir() {
        anyhow::bail!(
            "Expected a directory, got: {}\nUsage: ship convert .",
            source
        );
    }

    let detected = compiler::detect_providers(&path);
    if !detected.any() {
        anyhow::bail!(
            "No provider configs found in {}\n\
             Looking for: CLAUDE.md, .claude/, .mcp.json, .codex/, .gemini/, .cursor/",
            source
        );
    }

    let providers = detected.as_list();
    println!("Detected providers: {}", providers.join(", "));

    let library = compiler::decompile_all(&path);

    // ── Ensure .ship/ exists ─────────────────────────────────────────────────
    let ship_dir = path.join(".ship");
    std::fs::create_dir_all(&ship_dir)?;

    let mut files_written = 0;

    // ── ship.jsonc — manifest with providers and provider_defaults ────────────
    let ship_jsonc = ship_dir.join("ship.jsonc");
    if !ship_jsonc.exists() {
        let mut project = serde_json::json!({
            "providers": providers.iter().map(|p| serde_json::Value::String(p.to_string())).collect::<Vec<_>>()
        });
        if !library.provider_defaults.is_empty() {
            let defaults: serde_json::Map<String, serde_json::Value> = library
                .provider_defaults
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            project["provider_defaults"] = serde_json::Value::Object(defaults);
        }
        let manifest = serde_json::json!({
            "$schema": "../schemas/ship.schema.json",
            "project": project
        });
        let content = serde_json::to_string_pretty(&manifest)?;
        std::fs::write(&ship_jsonc, &content)?;
        println!("  wrote .ship/ship.jsonc");
        files_written += 1;
    }

    // ── mcp.jsonc — MCP server definitions ───────────────────────────────────
    if !library.mcp_servers.is_empty() {
        let mcp_jsonc = ship_dir.join("mcp.jsonc");
        let servers = convert_mcp::serialize_mcp_servers(&library.mcp_servers);
        let mcp_obj = serde_json::json!({
            "$schema": "../schemas/mcp.schema.json",
            "mcp": { "servers": servers }
        });
        std::fs::write(&mcp_jsonc, serde_json::to_string_pretty(&mcp_obj)?)?;
        println!(
            "  wrote .ship/mcp.jsonc ({} servers)",
            library.mcp_servers.len()
        );
        files_written += 1;
    }

    // ── permissions.jsonc — permission presets ────────────────────────────────
    let p = &library.permissions;
    let has_perms = !p.tools.allow.is_empty()
        || !p.tools.deny.is_empty()
        || !p.tools.ask.is_empty()
        || p.default_mode.is_some()
        || !p.additional_directories.is_empty()
        || p.agent.max_cost_per_session.is_some()
        || p.agent.max_turns.is_some();

    if has_perms {
        let perm_jsonc = ship_dir.join("permissions.jsonc");
        let mut preset = serde_json::json!({});
        if let Some(mode) = &p.default_mode {
            preset["default_mode"] = serde_json::json!(mode);
        }
        // Only emit non-default allow list
        let non_default_allow =
            !(p.tools.allow.is_empty() || (p.tools.allow.len() == 1 && p.tools.allow[0] == "*"));
        if non_default_allow {
            preset["tools_allow"] = serde_json::json!(p.tools.allow);
        }
        if !p.tools.deny.is_empty() {
            preset["tools_deny"] = serde_json::json!(p.tools.deny);
        }
        if !p.tools.ask.is_empty() {
            preset["tools_ask"] = serde_json::json!(p.tools.ask);
        }
        if !p.additional_directories.is_empty() {
            preset["additional_directories"] = serde_json::json!(p.additional_directories);
        }
        let perms_obj = serde_json::json!({
            "$schema": "../schemas/permissions.schema.json",
            "imported": preset
        });
        std::fs::write(&perm_jsonc, serde_json::to_string_pretty(&perms_obj)?)?;
        println!("  wrote .ship/permissions.jsonc");
        files_written += 1;
    }

    // ── rules/*.md — rule files ──────────────────────────────────────────────
    if !library.rules.is_empty() {
        let rules_dir = ship_dir.join("rules");
        std::fs::create_dir_all(&rules_dir)?;
        for rule in &library.rules {
            let file_name = if rule.file_name.ends_with(".md") {
                rule.file_name.clone()
            } else {
                format!("{}.md", rule.file_name)
            };
            std::fs::write(rules_dir.join(&file_name), &rule.content)?;
        }
        println!("  wrote .ship/rules/ ({} files)", library.rules.len());
        files_written += 1;
    }

    // ── .gitignore in .ship/ ─────────────────────────────────────────────────
    let gitignore = ship_dir.join(".gitignore");
    if !gitignore.exists() {
        std::fs::write(&gitignore, "secrets/\n")?;
    }

    if files_written == 0 {
        println!("  Nothing to write — providers detected but configs are empty.");
    } else {
        println!("\nConverted {} provider(s) into .ship/", providers.len());
        println!("  Run: ship use <agent-id>   to activate an agent");
        println!("  Run: ship compile          to compile back to provider configs");
    }

    Ok(())
}

#[cfg(test)]
#[path = "convert_tests.rs"]
mod tests;
