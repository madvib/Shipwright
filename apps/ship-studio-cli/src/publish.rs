use anyhow::{Context, Result};
use std::path::Path;

use compiler::manifest::ShipManifest;
use runtime::registry::hash::compute_tree_hash;

use crate::config::Credentials;

/// `ship publish --dry-run` output.
fn dry_run(manifest: &ShipManifest, tree_hash: &str) {
    println!("Package:  {}", manifest.module.name);
    println!("Version:  {}", manifest.module.version);
    println!("Hash:     {}", tree_hash);
    if let Some(ref desc) = manifest.module.description {
        println!("Description: {}", desc);
    }
    if let Some(ref lic) = manifest.module.license {
        println!("License:  {}", lic);
    }
    if !manifest.module.authors.is_empty() {
        println!("Authors:  {}", manifest.module.authors.join(", "));
    }
    println!("\n(dry run — no files uploaded)");
}

/// Build the JSON payload for the publish API.
fn build_payload(manifest: &ShipManifest, tree_hash: &str, tag: Option<&str>) -> serde_json::Value {
    let mut payload = serde_json::json!({
        "name": manifest.module.name,
        "version": manifest.module.version,
        "hash": tree_hash,
    });
    if let Some(ref desc) = manifest.module.description {
        payload["description"] = serde_json::Value::String(desc.clone());
    }
    if let Some(ref lic) = manifest.module.license {
        payload["license"] = serde_json::Value::String(lic.clone());
    }
    if !manifest.module.authors.is_empty() {
        payload["authors"] = serde_json::json!(manifest.module.authors);
    }
    if let Some(t) = tag {
        payload["tag"] = serde_json::Value::String(t.to_string());
    }
    if !manifest.exports.skills.is_empty() {
        payload["exports_skills"] = serde_json::json!(manifest.exports.skills);
    }
    if !manifest.exports.agents.is_empty() {
        payload["exports_agents"] = serde_json::json!(manifest.exports.agents);
    }
    payload
}

/// Publish the package at `root` to the Ship registry.
pub fn run_publish(root: &Path, is_dry_run: bool, tag: Option<&str>) -> Result<()> {
    let manifest_path = root.join(".ship").join("ship.toml");
    let manifest = ShipManifest::from_file(&manifest_path)
        .context("reading .ship/ship.toml — is this a Ship project?")?;

    let ship_dir = root.join(".ship");
    let tree_hash = compute_tree_hash(&ship_dir).context("computing content hash for .ship/")?;

    if is_dry_run {
        dry_run(&manifest, &tree_hash);
        return Ok(());
    }

    let creds = Credentials::load();
    let token = creds.token().ok_or_else(|| {
        anyhow::anyhow!("Not authenticated. Run `ship login` first, then retry `ship publish`.")
    })?;

    let payload = build_payload(&manifest, &tree_hash, tag);

    let mut resp = ureq::post("https://getship.dev/api/registry/publish")
        .header("Authorization", &format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .send(payload.to_string().as_bytes())
        .map_err(|e| anyhow::anyhow!("Publish request failed: {}", e))?;

    let status = resp.status();
    let body: String = resp.body_mut().read_to_string().unwrap_or_default();

    if status != 200 && status != 201 {
        let detail = serde_json::from_str::<serde_json::Value>(&body)
            .ok()
            .and_then(|v| v.get("error").and_then(|e| e.as_str()).map(String::from))
            .unwrap_or(body);
        anyhow::bail!("Publish failed ({}): {}", status, detail);
    }

    println!(
        "Published {}@{} ({})",
        manifest.module.name, manifest.module.version, tree_hash
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn dry_run_reads_manifest_and_hashes() -> Result<()> {
        let dir = tempdir()?;
        let ship_dir = dir.path().join(".ship");
        std::fs::create_dir_all(&ship_dir)?;
        std::fs::write(
            ship_dir.join("ship.toml"),
            r#"
[module]
name = "github.com/test/pkg"
version = "0.1.0"
description = "Test package"
license = "MIT"
authors = ["Test Author"]

[exports]
skills = ["agents/skills/my-skill"]
"#,
        )?;
        // Create a dummy skill so the hash is non-empty
        let skill_dir = ship_dir.join("agents").join("skills").join("my-skill");
        std::fs::create_dir_all(&skill_dir)?;
        std::fs::write(skill_dir.join("SKILL.md"), "# My Skill")?;

        // dry_run should succeed without network
        run_publish(dir.path(), true, None)?;
        Ok(())
    }

    #[test]
    fn publish_without_token_errors() -> Result<()> {
        let dir = tempdir()?;
        let ship_dir = dir.path().join(".ship");
        std::fs::create_dir_all(&ship_dir)?;
        std::fs::write(
            ship_dir.join("ship.toml"),
            r#"
[module]
name = "github.com/test/pkg"
version = "0.1.0"
"#,
        )?;

        let err = run_publish(dir.path(), false, None).unwrap_err();
        assert!(
            err.to_string().contains("ship login"),
            "expected auth error, got: {err}"
        );
        Ok(())
    }

    #[test]
    fn build_payload_includes_all_fields() {
        let manifest = ShipManifest::from_toml_str(
            r#"
[module]
name = "github.com/test/pkg"
version = "1.0.0"
description = "A test"
license = "MIT"
authors = ["Alice"]

[exports]
skills = ["agents/skills/foo"]
agents = ["agents/profiles/bar.toml"]
"#,
        )
        .unwrap();

        let payload = build_payload(&manifest, "sha256:abc123", Some("beta"));
        assert_eq!(payload["name"], "github.com/test/pkg");
        assert_eq!(payload["version"], "1.0.0");
        assert_eq!(payload["hash"], "sha256:abc123");
        assert_eq!(payload["description"], "A test");
        assert_eq!(payload["license"], "MIT");
        assert_eq!(payload["tag"], "beta");
        assert_eq!(payload["authors"][0], "Alice");
        assert_eq!(payload["exports_skills"][0], "agents/skills/foo");
    }
}
