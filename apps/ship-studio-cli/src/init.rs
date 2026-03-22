//! `ship init` — scaffold .ship/ in a project or configure ~/.ship/ globally.

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::paths;

pub fn run(global: bool, provider: Option<String>, from: Option<String>) -> Result<()> {
    if global {
        run_global()
    } else if let Some(url) = from {
        run_from_url(&url)
    } else {
        run_project(provider)
    }
}

// ── --from URL scaffolding ───────────────────────────────────────────────────

/// JSON config bundle shape expected from `--from <url>`.
#[derive(Debug, Deserialize)]
struct ConfigBundle {
    #[serde(default)]
    agents: Vec<BundleAgent>,
    #[serde(default)]
    skills: Vec<BundleSkill>,
    #[serde(default)]
    permissions: Option<BundlePermissions>,
}

#[derive(Debug, Deserialize)]
struct BundleAgent {
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    model: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BundleSkill {
    id: String,
    #[serde(default)]
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BundlePermissions {
    #[serde(default)]
    preset: Option<String>,
}

/// Fetch a JSON config bundle from a URL and scaffold .ship/ from it.
fn run_from_url(url: &str) -> Result<()> {
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
        let safe_name = sanitize_filename(&agent.name);
        let dest = paths::agents_dir().join(format!("{}.toml", safe_name));
        let model = agent.model.as_deref().unwrap_or("sonnet");
        let description = agent.description.as_deref().unwrap_or("");
        let mut toml_content = format!(
            "[agent]\nname = {name}\nid = {id}\nversion = \"0.1.0\"\n\
             description = {desc}\nproviders = [\"claude\"]\n",
            name = quote_toml(&agent.name),
            id = quote_toml(&safe_name),
            desc = quote_toml(description),
        );
        toml_content.push_str(&format!("\n# model hint: {}\n", model));
        std::fs::write(&dest, &toml_content)
            .with_context(|| format!("Failed to write agent {}", dest.display()))?;
        n_agents += 1;
    }

    // Write skills
    for skill in &bundle.skills {
        let safe_id = sanitize_filename(&skill.id);
        let dest = paths::skills_dir().join(format!("{}.md", safe_id));
        let content = skill.content.as_deref().unwrap_or("");
        std::fs::write(&dest, content)
            .with_context(|| format!("Failed to write skill {}", dest.display()))?;
        n_skills += 1;
    }

    // Write permissions
    if let Some(ref perms) = bundle.permissions
        && let Some(ref preset) = perms.preset {
            let dest = paths::project_dir().join("permissions.toml");
            let content = format!("[permissions]\npreset = {}\n", quote_toml(preset));
            std::fs::write(&dest, &content)
                .with_context(|| format!("Failed to write {}", dest.display()))?;
        }

    let preset_msg = bundle.permissions.as_ref()
        .and_then(|p| p.preset.as_deref())
        .map(|p| format!(", preset: {}", p))
        .unwrap_or_default();
    println!("initialized .ship/ from {}: {} agents, {} skills{}", url, n_agents, n_skills, preset_msg);
    println!("\nNext steps:");
    println!("  ship use <agent-id>     activate an agent");
    Ok(())
}

/// Quote a string for TOML output.
fn quote_toml(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Replace characters that are not safe in filenames.
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

// ── Normal init paths ────────────────────────────────────────────────────────

fn run_global() -> Result<()> {
    paths::ensure_global_dirs()?;
    let gdir = paths::global_dir();
    let cfg_path = gdir.join("config.toml");
    if !cfg_path.exists() {
        std::fs::write(&cfg_path, "# Ship global configuration\n\n[identity]\nname = \"\"\n")?;
    }
    if !gdir.join("README.md").exists() {
        std::fs::write(gdir.join("README.md"), "# Ship\n\nSee https://getship.dev\n")?;
    }
    println!("initialized global config at ~/.ship/");
    println!("  Edit ~/.ship/config.toml to set your identity");
    Ok(())
}

fn run_project(provider: Option<String>) -> Result<()> {
    paths::ensure_project_dirs()?;
    let ship_jsonc = paths::project_ship_jsonc();
    // Scaffold .ship/ship.jsonc when no config exists (prefer JSONC over TOML)
    if !ship_jsonc.exists() && !paths::project_ship_toml().exists() {
        let prov = provider.as_deref().unwrap_or("claude");
        std::fs::write(&ship_jsonc, format!(
            "{{\n  \"$schema\": \"../schemas/ship.schema.json\",\n  \"project\": {{\n    \"providers\": [\"{prov}\"],\n  }},\n}}"
        ))?;
    }
    let pdir = paths::project_dir();
    if !pdir.join(".gitignore").exists() {
        std::fs::write(pdir.join(".gitignore"),
            "# Ship compiled artifacts\n/secrets/\nCLAUDE.md\nGEMINI.md\nAGENTS.md\n.mcp.json\n.codex/\n.gemini/\n.cursor/\n")?;
    }
    if !pdir.join("README.md").exists() {
        std::fs::write(pdir.join("README.md"),
            "# Ship Configuration\n\nManaged by [Ship](https://getship.dev). Run `ship use <agent-id>` to activate.\n")?;
    }
    println!("initialized .ship/ in current directory");
    println!("  providers: {}", provider.as_deref().unwrap_or("claude"));
    println!("\nNext steps:");
    println!("  ship use <agent-id>     activate an agent");
    println!("  ship compile            re-compile current agent");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_in_tmp<F: FnOnce(&std::path::Path)>(f: F) {
        let tmp = tempfile::TempDir::new().unwrap();
        let orig = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();
        f(tmp.path());
        std::env::set_current_dir(orig).unwrap();
    }

    #[test]
    fn init_from_flag_parsed() {
        use clap::Parser;
        use crate::cli::Cli;
        let cli = Cli::parse_from(["ship", "init", "--from", "https://example.com/c.json"]);
        match cli.command {
            Some(crate::cli::Commands::Init { from, .. }) => {
                assert_eq!(from.as_deref(), Some("https://example.com/c.json"));
            }
            other => panic!("expected Init, got {:?}", other),
        }
    }

    #[test]
    fn init_without_from_flag_parsed() {
        use clap::Parser;
        use crate::cli::Cli;
        let cli = Cli::parse_from(["ship", "init"]);
        match cli.command {
            Some(crate::cli::Commands::Init { from, .. }) => assert!(from.is_none()),
            other => panic!("expected Init, got {:?}", other),
        }
    }

    #[test]
    fn sanitize_and_quote() {
        assert_eq!(sanitize_filename("hello world"), "hello-world");
        assert_eq!(sanitize_filename("path/name"), "path-name");
        assert_eq!(quote_toml("simple"), "\"simple\"");
        assert_eq!(quote_toml("a\"b"), "\"a\\\"b\"");
    }

    #[test]
    fn config_bundle_parses() {
        let full: ConfigBundle = serde_json::from_str(r#"{
            "agents": [{"name": "default", "model": "sonnet"}],
            "skills": [{"id": "tdd", "content": "..."}],
            "permissions": {"preset": "elevated"}
        }"#).unwrap();
        assert_eq!(full.agents.len(), 1);
        assert_eq!(full.permissions.unwrap().preset.as_deref(), Some("elevated"));

        let empty: ConfigBundle = serde_json::from_str("{}").unwrap();
        assert!(empty.agents.is_empty());
    }

    #[test]
    fn run_from_url_scaffolds_files() {
        let mut server = mockito::Server::new();
        let _m = server.mock("GET", "/c.json").with_status(200)
            .with_body(r#"{
                "agents": [{"name": "default", "description": "Main", "model": "sonnet"}],
                "skills": [{"id": "tdd", "content": "Write tests first"}],
                "permissions": {"preset": "elevated"}
            }"#).create();

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
        let _m = server.mock("GET", "/bad").with_status(200).with_body("not json").create();
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
