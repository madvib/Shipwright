use std::collections::{HashSet, VecDeque};
use std::path::Path;

use anyhow::Context;

use super::cache::{CachedPackage, PackageCache};
use super::constraint::parse_constraint;
use super::fetch::fetch_package_content;
use super::resolver::{resolve_alias, resolve_version};
use super::tracking::track_install;
use super::types::{LockedPackage, ShipLock, ShipManifest, parse_ship_lock, serialize_ship_lock};
use super::verify::check_registry_hash;

/// Options passed to `resolve_and_fetch`.
#[derive(Debug, Default)]
pub struct InstallOptions {
    /// If `true`, fail when the lockfile would change rather than updating it.
    pub frozen: bool,
    /// If `true`, skip install tracking (no network POST to registry).
    pub offline: bool,
}

/// Returned by `resolve_and_fetch`.
#[derive(Debug)]
pub struct InstallResult {
    pub packages: Vec<CachedPackage>,
    pub lockfile_written: bool,
}

/// Resolve all dependencies in `manifest`, populate the cache, and return the
/// resolved packages.  Writes `lock_path` when the lockfile changes unless
/// `opts.frozen` is set.
///
/// Does NOT compile provider targets — compilation is the compiler's responsibility.
pub fn resolve_and_fetch(
    manifest: &ShipManifest,
    lock_path: &Path,
    cache: &PackageCache,
    opts: &InstallOptions,
) -> anyhow::Result<InstallResult> {
    let existing_lock = load_lock_if_present(lock_path)?;

    // Determine which deps need (re-)resolution.
    let need_resolve: Vec<String> = match &existing_lock {
        None => {
            // No lockfile — resolve everything.
            manifest.dependencies.keys().cloned().collect()
        }
        Some(lock) => {
            let sync = lock.check_sync(manifest);
            if opts.frozen && !sync.is_in_sync() {
                anyhow::bail!(
                    "lockfile is out of sync with ship.toml (--frozen):\n  added: {:?}\n  removed: {:?}\nRun `ship install` to update.",
                    sync.added,
                    sync.removed
                );
            }
            if sync.is_in_sync() {
                // Nothing changed — only fetch what's missing from cache.
                Vec::new()
            } else {
                // Re-resolve only the changed deps.
                sync.added.clone()
            }
        }
    };

    // Start from the existing lock entries (for deps that haven't changed).
    let mut locked: Vec<LockedPackage> = match &existing_lock {
        Some(lock) => {
            let sync = lock.check_sync(manifest);
            lock.package
                .iter()
                .filter(|p| !sync.removed.contains(&p.path))
                .cloned()
                .collect()
        }
        None => Vec::new(),
    };

    // Resolve newly added / all deps depending on whether a lock existed.
    let deps_to_resolve: Vec<(&String, &super::types::Dependency)> = if existing_lock.is_none() {
        manifest.dependencies.iter().collect()
    } else {
        manifest
            .dependencies
            .iter()
            .filter(|(k, _)| need_resolve.contains(k))
            .collect()
    };

    // BFS queue: (dep_path, version_constraint, ancestor_chain).
    // ancestor_chain tracks the dependency path for cycle detection.
    let mut queue: VecDeque<(String, String, Vec<String>)> = deps_to_resolve
        .iter()
        .map(|(k, v)| ((*k).clone(), v.version.clone(), vec!["ship.toml".into()]))
        .collect();

    // Track already-resolved dep paths to avoid duplicate work.
    let mut visited: HashSet<String> = HashSet::new();
    // Track resolved versions per dep path for conflict detection.
    let mut resolved_versions: std::collections::HashMap<String, (String, String)> =
        std::collections::HashMap::new();

    while let Some((raw_dep_path, version_str, ancestors)) = queue.pop_front() {
        // Resolve scoped aliases (@owner/repo → github.com/owner/repo).
        let dep_path = resolve_alias(&raw_dep_path)
            .with_context(|| format!("resolving alias for {raw_dep_path}"))?;
        let requestor = ancestors.last().cloned().unwrap_or_default();

        if visited.contains(&dep_path) {
            // Check for version conflicts.
            if let Some((prev_ver, prev_requestor)) = resolved_versions.get(&dep_path)
                && *prev_ver != version_str
            {
                anyhow::bail!(
                    "version conflict for {}: {} requires '{}' but {} requires '{}'",
                    dep_path,
                    requestor,
                    version_str,
                    prev_requestor,
                    prev_ver
                );
            }
            continue;
        }
        visited.insert(dep_path.clone());
        resolved_versions.insert(dep_path.clone(), (version_str.clone(), requestor.clone()));

        let constraint = parse_constraint(&version_str)
            .with_context(|| format!("parsing constraint for {dep_path}"))?;

        let resolved = resolve_version(&dep_path, &constraint)
            .with_context(|| format!("resolving {dep_path}"))?;

        // Fetch if not cached.
        let cached = match cache.get(&dep_path, &resolved.tag) {
            Some(pkg) => {
                if cache.verify(&pkg).is_ok() {
                    pkg
                } else {
                    fetch_and_store(cache, &dep_path, &resolved.tag, &resolved.commit, opts.offline)?
                }
            }
            None => fetch_and_store(cache, &dep_path, &resolved.tag, &resolved.commit, opts.offline)?,
        };

        // For new deps (no existing lockfile entry), verify against the
        // registry's published hash. Advisory only — warns but does not block.
        let in_existing_lock = existing_lock
            .as_ref()
            .is_some_and(|lock| lock.package.iter().any(|p| p.path == dep_path));
        if !in_existing_lock {
            if let Some(warning) = check_registry_hash(
                &dep_path,
                &resolved.tag,
                &cached.hash,
                opts.offline,
            ) {
                eprintln!("{warning}");
            }
        }

        // Record in lock.
        locked.retain(|p| p.path != dep_path);
        locked.push(LockedPackage {
            path: dep_path.clone(),
            version: resolved.tag.clone(),
            commit: resolved.commit.clone(),
            hash: cached.hash.clone(),
        });

        // Discover transitive deps from cached package's ship.toml.
        let mut child_ancestors = ancestors.clone();
        child_ancestors.push(dep_path.clone());
        let sub_deps = discover_transitive_deps(&cached.dir, &dep_path, &child_ancestors)?;
        for (sub_path, sub_ver) in sub_deps {
            queue.push_back((sub_path, sub_ver, child_ancestors.clone()));
        }
    }

    // For deps already in lock (unchanged), ensure they are in the cache.
    let mut packages: Vec<CachedPackage> = Vec::new();
    for lp in &locked {
        let cached = match cache.get(&lp.path, &lp.version) {
            Some(pkg) if cache.verify(&pkg).is_ok() => pkg,
            _ => fetch_and_store(cache, &lp.path, &lp.version, &lp.commit, opts.offline)?,
        };

        // Enforce hash matches what's in the lockfile.
        if cached.hash != lp.hash {
            anyhow::bail!(
                "hash mismatch for {}@{}: lockfile={} cache={}",
                lp.path,
                lp.version,
                lp.hash,
                cached.hash
            );
        }

        packages.push(cached);
    }

    // Write updated lockfile if necessary.
    let new_lock = ShipLock {
        version: 1,
        package: locked,
    };

    let lockfile_written = should_write_lock(&existing_lock, &new_lock);
    if lockfile_written {
        write_lock_atomic(lock_path, &new_lock).context("writing ship.lock")?;
    }

    Ok(InstallResult {
        packages,
        lockfile_written,
    })
}

/// Check a cached package directory for a manifest with `[dependencies]`.
/// Returns `(dep_path, version_string)` pairs for transitive deps.
/// Errors if a sub-dep is already in the ancestor chain (cycle).
fn discover_transitive_deps(
    cached_dir: &Path,
    parent_path: &str,
    ancestors: &[String],
) -> anyhow::Result<Vec<(String, String)>> {
    // Try JSONC first, then TOML for backward compat.
    let manifest_path = if cached_dir.join("ship.jsonc").exists() {
        cached_dir.join("ship.jsonc")
    } else if cached_dir.join("ship.toml").exists() {
        cached_dir.join("ship.toml")
    } else {
        return Ok(vec![]);
    };
    let sub_manifest = compiler::manifest::ShipManifest::from_file(&manifest_path)
        .with_context(|| format!("parsing manifest in transitive dep {parent_path}"))?;
    let mut deps = Vec::new();
    for (path, dep_val) in sub_manifest.dependencies {
        if ancestors.contains(&path) {
            let chain: Vec<&str> = ancestors.iter().map(|s| s.as_str()).collect();
            anyhow::bail!(
                "dependency cycle detected: {} -> {} (chain: {} -> {})",
                parent_path,
                path,
                chain.join(" -> "),
                path
            );
        }
        let version = dep_val.into_dep().version;
        deps.push((path, version));
    }
    Ok(deps)
}

fn load_lock_if_present(path: &Path) -> anyhow::Result<Option<ShipLock>> {
    if !path.exists() {
        return Ok(None);
    }
    let content =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let lock = parse_ship_lock(&content).with_context(|| format!("parsing {}", path.display()))?;
    Ok(Some(lock))
}

fn fetch_and_store(
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

fn should_write_lock(existing: &Option<ShipLock>, new: &ShipLock) -> bool {
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

fn write_lock_atomic(path: &Path, lock: &ShipLock) -> anyhow::Result<()> {
    let content = serialize_ship_lock(lock)?;
    let tmp_path = path.with_extension("lock.tmp");
    std::fs::write(&tmp_path, &content)
        .with_context(|| format!("writing tmp lockfile {}", tmp_path.display()))?;
    std::fs::rename(&tmp_path, path)
        .with_context(|| format!("renaming lockfile into place at {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
#[path = "install_tests.rs"]
mod tests;
