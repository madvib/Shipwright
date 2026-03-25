//! Internal helpers for the install pipeline.
//!
//! Split from `install.rs` to stay under the 300-line file cap.

use std::path::Path;

use anyhow::Context;

use super::cache::{CachedPackage, PackageCache};
use super::fetch::fetch_package_content;
use super::tracking::track_install;
use super::types::{ShipLock, parse_ship_lock, serialize_ship_lock};

pub fn load_lock_if_present(path: &Path) -> anyhow::Result<Option<ShipLock>> {
    if !path.exists() {
        return Ok(None);
    }
    let content =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let lock = parse_ship_lock(&content).with_context(|| format!("parsing {}", path.display()))?;
    Ok(Some(lock))
}

pub fn fetch_and_store(
    cache: &PackageCache,
    dep_path: &str,
    version: &str,
    commit: &str,
    offline: bool,
) -> anyhow::Result<CachedPackage> {
    let tmp = tempfile::tempdir().context("creating tempdir for package fetch")?;
    let git_url = format!("https://{}.git", dep_path);
    fetch_package_content(&git_url, commit, tmp.path())
        .with_context(|| format!("fetching {dep_path} @ {commit}"))?;

    // Ship-native packages land in tmp/.ship/; root-manifest packages land in tmp/.
    let ship_dir = tmp.path().join(".ship");
    let content_dir = if ship_dir.is_dir() {
        ship_dir.as_path()
    } else {
        tmp.path()
    };

    // Security scan: block packages with critical hidden Unicode characters.
    let findings = crate::security::scan_dir(content_dir);
    if crate::security::has_critical(&findings) {
        let critical_findings: Vec<_> = findings
            .iter()
            .filter(|f| f.severity == crate::security::Severity::Critical)
            .collect();
        let details: Vec<String> = critical_findings.iter().map(|f| f.to_string()).collect();
        anyhow::bail!(
            "security scan blocked {dep_path}@{version}: {} critical finding(s) \
             (hidden Unicode characters that may be prompt injection vectors):\n  {}",
            critical_findings.len(),
            details.join("\n  ")
        );
    }

    let cached = cache
        .store(dep_path, version, commit, content_dir)
        .with_context(|| format!("storing {dep_path}@{version} in cache"))?;

    // Fire-and-forget install tracking for freshly downloaded packages.
    track_install(dep_path, offline);

    Ok(cached)
}

pub fn should_write_lock(existing: &Option<ShipLock>, new: &ShipLock) -> bool {
    match existing {
        None => true,
        Some(old) => {
            let mut old_pkgs = old.package.clone();
            let mut new_pkgs = new.package.clone();
            old_pkgs.sort_by(|a, b| a.path.cmp(&b.path));
            new_pkgs.sort_by(|a, b| a.path.cmp(&b.path));
            old_pkgs != new_pkgs
        }
    }
}

pub fn write_lock_atomic(path: &Path, lock: &ShipLock) -> anyhow::Result<()> {
    let content = serialize_ship_lock(lock)?;
    let tmp_path = path.with_extension("lock.tmp");
    std::fs::write(&tmp_path, &content)
        .with_context(|| format!("writing tmp lockfile {}", tmp_path.display()))?;
    std::fs::rename(&tmp_path, path)
        .with_context(|| format!("renaming lockfile into place at {}", path.display()))?;
    Ok(())
}
