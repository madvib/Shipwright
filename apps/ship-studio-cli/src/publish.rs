use anyhow::{Context, Result};
use std::path::Path;

use compiler::manifest::ShipManifest;
use runtime::registry::hash::{ExportHashes, compute_export_hashes};

use crate::config::Credentials;

/// Filter manifest exports to a single export path.
///
/// Returns the filtered (skills, agents) vectors. Errors if the path is not
/// found in either list.
fn filter_exports(
    manifest: &ShipManifest,
    export_path: &str,
) -> Result<(Vec<String>, Vec<String>)> {
    if manifest.exports.skills.iter().any(|s| s == export_path) {
        return Ok((vec![export_path.to_string()], vec![]));
    }
    if manifest.exports.agents.iter().any(|a| a == export_path) {
        return Ok((vec![], vec![export_path.to_string()]));
    }
    anyhow::bail!(
        "export '{}' not found in manifest — available exports:\n  skills: {:?}\n  agents: {:?}",
        export_path,
        manifest.exports.skills,
        manifest.exports.agents,
    );
}

/// Compute per-export hashes from the manifest exports, optionally filtered.
fn resolve_hashes(
    ship_dir: &Path,
    manifest: &ShipManifest,
    export_path: Option<&str>,
) -> Result<ExportHashes> {
    let (skills, agents) = match export_path {
        Some(path) => filter_exports(manifest, path)?,
        None => (
            manifest.exports.skills.clone(),
            manifest.exports.agents.clone(),
        ),
    };
    compute_export_hashes(ship_dir, &skills, &agents).context("computing per-export content hashes")
}

/// `ship publish --dry-run` output.
fn dry_run(manifest: &ShipManifest, hashes: &ExportHashes, export_path: Option<&str>) {
    println!("Package:  {}", manifest.module.name);
    println!("Version:  {}", manifest.module.version);
    if let Some(path) = export_path {
        println!("Export:   {}", path);
    }
    println!("Hash:     {}", hashes.combined);
    if let Some(ref desc) = manifest.module.description {
        println!("Description: {}", desc);
    }
    if let Some(ref lic) = manifest.module.license {
        println!("License:  {}", lic);
    }
    if !manifest.module.authors.is_empty() {
        println!("Authors:  {}", manifest.module.authors.join(", "));
    }
    if !hashes.per_export.is_empty() {
        println!("\nExport hashes:");
        for (path, hash) in &hashes.per_export {
            println!("  {}: {}", path, hash);
        }
    }
    println!("\n(dry run — no files uploaded)");
}

/// Build the JSON payload for the publish API.
///
/// When `export_path` is set, only the matching skill/agent export and its
/// hash are included in the payload.
fn build_payload(
    manifest: &ShipManifest,
    hashes: &ExportHashes,
    tag: Option<&str>,
    export_path: Option<&str>,
) -> serde_json::Value {
    let mut payload = serde_json::json!({
        "name": manifest.module.name,
        "version": manifest.module.version,
        "hash": hashes.combined,
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

    // When filtering to a single export, only include that one.
    let (skills, agents) = match export_path {
        Some(path) => {
            let skills: Vec<&String> = manifest
                .exports
                .skills
                .iter()
                .filter(|s| s.as_str() == path)
                .collect();
            let agents: Vec<&String> = manifest
                .exports
                .agents
                .iter()
                .filter(|a| a.as_str() == path)
                .collect();
            (skills, agents)
        }
        None => {
            let skills: Vec<&String> = manifest.exports.skills.iter().collect();
            let agents: Vec<&String> = manifest.exports.agents.iter().collect();
            (skills, agents)
        }
    };

    if !skills.is_empty() {
        payload["exports_skills"] = serde_json::json!(skills);
    }
    if !agents.is_empty() {
        payload["exports_agents"] = serde_json::json!(agents);
    }
    // Per-export hashes: primary integrity mechanism.
    if !hashes.per_export.is_empty() {
        payload["export_hashes"] = serde_json::json!(hashes.per_export);
    }
    payload
}

/// Publish the package at `root` to the Ship registry.
///
/// When `export_path` is `Some`, only that single export is hashed and
/// included in the publish payload.
pub fn run_publish(
    root: &Path,
    export_path: Option<&str>,
    is_dry_run: bool,
    tag: Option<&str>,
) -> Result<()> {
    let ship_dir = root.join(".ship");
    let manifest_path = ship_dir.join("ship.jsonc");
    let manifest = ShipManifest::from_file(&manifest_path)
        .context("reading .ship/ship.jsonc — is this a Ship project?")?;
    let hashes = resolve_hashes(&ship_dir, &manifest, export_path)?;

    if is_dry_run {
        dry_run(&manifest, &hashes, export_path);
        return Ok(());
    }

    let creds = Credentials::load();
    let token = creds.token().ok_or_else(|| {
        anyhow::anyhow!("Not authenticated. Run `ship login` first, then retry `ship publish`.")
    })?;

    let payload = build_payload(&manifest, &hashes, tag, export_path);

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

    let label = match export_path {
        Some(path) => format!(
            "{}@{} [{}]",
            manifest.module.name, manifest.module.version, path
        ),
        None => format!("{}@{}", manifest.module.name, manifest.module.version),
    };
    println!("Published {} ({})", label, hashes.combined);
    Ok(())
}

#[cfg(test)]
#[path = "publish_tests.rs"]
mod tests;
