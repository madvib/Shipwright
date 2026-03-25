//! `ship init --from <url>` — scaffold .ship/ from a remote JSON config bundle.

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::paths;

/// JSON config bundle shape expected from `--from <url>`.
#[derive(Debug, Deserialize)]
pub(crate) struct ConfigBundle {
    #[serde(default)]
    pub agents: Vec<BundleAgent>,
    #[serde(default)]
    pub skills: Vec<BundleSkill>,
    #[serde(default)]
    pub permissions: Option<BundlePermissions>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BundleAgent {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BundleSkill {
    pub id: String,
    #[serde(default)]
    pub content: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BundlePermissions {
    #[serde(default)]
    pub preset: Option<String>,
}

/// Fetch a JSON config bundle from a URL and scaffold .ship/ from it.
pub fn run_from_url(url: &str) -> Result<()> {
    // Fetch the URL
    let body: String = ureq::get(url)
        .call()
        .map_err(|e| {
            anyhow::anyhow!(
                "Could not reach {}. Check the URL and your network connection.\nDetails: {}",
                url,
                e
            )
        })?
        .body_mut()
        .read_to_string()
        .context("Failed to read response body")?;

    let bundle: ConfigBundle = serde_json::from_str(&body).map_err(|e| {
        anyhow::anyhow!(
            "Invalid JSON from {}.\nExpected a config bundle with agents/skills/permissions.\nParse error: {}",
            url,
            e
        )
    })?;

    // Ensure .ship/ directory structure exists
    paths::ensure_project_dirs()?;

    let mut n_agents = 0usize;
    let mut n_skills = 0usize;

    // Write agents
    for agent in &bundle.agents {
        let safe_name = crate::init::sanitize_filename(&agent.name);
        let dest = paths::agents_dir().join(format!("{}.toml", safe_name));
        let model = agent.model.as_deref().unwrap_or("sonnet");
        let description = agent.description.as_deref().unwrap_or("");
        let mut toml_content = format!(
            "[agent]\nname = {name}\nid = {id}\nversion = \"0.1.0\"\n\
             description = {desc}\nproviders = [\"claude\"]\n",
            name = crate::init::quote_toml(&agent.name),
            id = crate::init::quote_toml(&safe_name),
            desc = crate::init::quote_toml(description),
        );
        toml_content.push_str(&format!("\n# model hint: {}\n", model));
        std::fs::write(&dest, &toml_content)
            .with_context(|| format!("Failed to write agent {}", dest.display()))?;
        n_agents += 1;
    }

    // Write skills
    for skill in &bundle.skills {
        let safe_id = crate::init::sanitize_filename(&skill.id);
        let dest = paths::skills_dir().join(format!("{}.md", safe_id));
        let content = skill.content.as_deref().unwrap_or("");
        std::fs::write(&dest, content)
            .with_context(|| format!("Failed to write skill {}", dest.display()))?;
        n_skills += 1;
    }

    // Write permissions
    if let Some(ref perms) = bundle.permissions
        && let Some(ref preset) = perms.preset
    {
        let dest = paths::project_dir().join("permissions.toml");
        let content = format!(
            "[permissions]\npreset = {}\n",
            crate::init::quote_toml(preset),
        );
        std::fs::write(&dest, &content)
            .with_context(|| format!("Failed to write {}", dest.display()))?;
    }

    let preset_msg = bundle
        .permissions
        .as_ref()
        .and_then(|p| p.preset.as_deref())
        .map(|p| format!(", preset: {}", p))
        .unwrap_or_default();
    println!(
        "initialized .ship/ from {}: {} agents, {} skills{}",
        url, n_agents, n_skills, preset_msg
    );
    println!("\nNext steps:");
    println!("  ship use <agent-id>     activate an agent");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    static CWD_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn run_in_tmp<F: FnOnce(&std::path::Path)>(f: F) {
        let _guard = CWD_LOCK.lock().unwrap();
        let tmp = tempfile::TempDir::new().unwrap();
        let orig = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();
        f(tmp.path());
        std::env::set_current_dir(orig).unwrap();
    }

    #[test]
    fn config_bundle_parses() {
        let full: ConfigBundle = serde_json::from_str(
            r#"{
            "agents": [{"name": "default", "model": "sonnet"}],
            "skills": [{"id": "tdd", "content": "..."}],
            "permissions": {"preset": "elevated"}
        }"#,
        )
        .unwrap();
        assert_eq!(full.agents.len(), 1);
        assert_eq!(
            full.permissions.unwrap().preset.as_deref(),
            Some("elevated")
        );

        let empty: ConfigBundle = serde_json::from_str("{}").unwrap();
        assert!(empty.agents.is_empty());
    }

    #[test]
    fn run_from_url_scaffolds_files() {
        let mut server = mockito::Server::new();
        let _m = server
            .mock("GET", "/c.json")
            .with_status(200)
            .with_body(
                r#"{
                "agents": [{"name": "default", "description": "Main", "model": "sonnet"}],
                "skills": [{"id": "tdd", "content": "Write tests first"}],
                "permissions": {"preset": "elevated"}
            }"#,
            )
            .create();

        run_in_tmp(|tmp| {
            run_from_url(&format!("{}/c.json", server.url())).unwrap();
            assert!(tmp.join(".ship/agents/default.toml").exists());
            let agent = std::fs::read_to_string(tmp.join(".ship/agents/default.toml")).unwrap();
            assert!(agent.contains("name = \"default\""));
            assert!(tmp.join(".ship/skills/tdd.md").exists());
            let perms = std::fs::read_to_string(tmp.join(".ship/permissions.toml")).unwrap();
            assert!(perms.contains("preset = \"elevated\""));
        });
    }

    #[test]
    fn run_from_url_errors_on_bad_json() {
        let mut server = mockito::Server::new();
        let _m = server
            .mock("GET", "/bad")
            .with_status(200)
            .with_body("not json")
            .create();
        run_in_tmp(|_| {
            let err = run_from_url(&format!("{}/bad", server.url())).unwrap_err();
            assert!(err.to_string().contains("Invalid JSON"), "got: {err}");
        });
    }

    #[test]
    fn run_from_url_errors_on_unreachable() {
        run_in_tmp(|_| {
            let err = run_from_url("http://127.0.0.1:1/x").unwrap_err();
            assert!(err.to_string().contains("Could not reach"), "got: {err}");
        });
    }
}
