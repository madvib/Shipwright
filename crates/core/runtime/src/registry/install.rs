use std::collections::{HashSet, VecDeque};
use std::path::Path;

use anyhow::Context;

use super::cache::{CachedPackage, PackageCache};
use super::constraint::parse_constraint;
// Re-export helpers used by install_tests.rs (via `use super::*`).
pub(crate) use super::install_helpers::write_lock_atomic;
use super::install_helpers::{fetch_and_store, load_lock_if_present, should_write_lock};
use super::resolver::{resolve_alias, resolve_version};
use super::types::{LockedPackage, ShipLock, ShipManifest};
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
/// Does NOT compile provider targets -- compilation is the compiler's responsibility.
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
            // No lockfile -- resolve everything.
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
                Vec::new()
            } else {
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

    let deps_to_resolve = collect_deps_to_resolve(manifest, &existing_lock, &need_resolve);
    let mut queue: VecDeque<(String, String, Vec<String>)> = deps_to_resolve
        .iter()
        .map(|(k, v)| ((*k).clone(), v.version.clone(), vec!["ship.toml".into()]))
        .collect();

    let mut visited: HashSet<String> = HashSet::new();
    let mut resolved_versions: std::collections::HashMap<String, (String, String)> =
        std::collections::HashMap::new();

    while let Some((raw_dep_path, version_str, ancestors)) = queue.pop_front() {
        let dep_path = resolve_alias(&raw_dep_path)
            .with_context(|| format!("resolving alias for {raw_dep_path}"))?;
        let requestor = ancestors.last().cloned().unwrap_or_default();

        if visited.contains(&dep_path) {
            check_version_conflict(&dep_path, &version_str, &requestor, &resolved_versions)?;
            continue;
        }
        visited.insert(dep_path.clone());
        resolved_versions.insert(dep_path.clone(), (version_str.clone(), requestor.clone()));

        let constraint = parse_constraint(&version_str)
            .with_context(|| format!("parsing constraint for {dep_path}"))?;
        let resolved = resolve_version(&dep_path, &constraint)
            .with_context(|| format!("resolving {dep_path}"))?;

        let cached = fetch_or_use_cache(cache, &dep_path, &resolved.tag, &resolved.commit, opts)?;

        // For new deps (no existing lockfile entry), verify against the
        // registry's published hash. Advisory only -- warns but does not block.
        let in_existing_lock = existing_lock
            .as_ref()
            .is_some_and(|lock| lock.package.iter().any(|p| p.path == dep_path));
        if !in_existing_lock {
            if let Some(warning) =
                check_registry_hash(&dep_path, &resolved.tag, &cached.hash, opts.offline)
            {
                eprintln!("{warning}");
            }
        }

        locked.retain(|p| p.path != dep_path);
        locked.push(LockedPackage {
            path: dep_path.clone(),
            version: resolved.tag.clone(),
            commit: resolved.commit.clone(),
            hash: cached.hash.clone(),
        });

        let mut child_ancestors = ancestors.clone();
        child_ancestors.push(dep_path.clone());
        let sub_deps = discover_transitive_deps(&cached.dir, &dep_path, &child_ancestors)?;
        for (sub_path, sub_ver) in sub_deps {
            queue.push_back((sub_path, sub_ver, child_ancestors.clone()));
        }
    }

    let packages = verify_locked_packages(&locked, cache, opts)?;

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

fn collect_deps_to_resolve<'a>(
    manifest: &'a ShipManifest,
    existing_lock: &Option<ShipLock>,
    need_resolve: &[String],
) -> Vec<(&'a String, &'a super::types::Dependency)> {
    if existing_lock.is_none() {
        manifest.dependencies.iter().collect()
    } else {
        manifest
            .dependencies
            .iter()
            .filter(|(k, _)| need_resolve.contains(k))
            .collect()
    }
}

fn check_version_conflict(
    dep_path: &str,
    version_str: &str,
    requestor: &str,
    resolved_versions: &std::collections::HashMap<String, (String, String)>,
) -> anyhow::Result<()> {
    if let Some((prev_ver, prev_requestor)) = resolved_versions.get(dep_path)
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
    Ok(())
}

fn fetch_or_use_cache(
    cache: &PackageCache,
    dep_path: &str,
    tag: &str,
    commit: &str,
    opts: &InstallOptions,
) -> anyhow::Result<CachedPackage> {
    match cache.get(dep_path, tag) {
        Some(pkg) if cache.verify(&pkg).is_ok() => Ok(pkg),
        _ => fetch_and_store(cache, dep_path, tag, commit, opts.offline),
    }
}

/// Ensure all locked packages are in the cache and verify hashes.
fn verify_locked_packages(
    locked: &[LockedPackage],
    cache: &PackageCache,
    opts: &InstallOptions,
) -> anyhow::Result<Vec<CachedPackage>> {
    let mut packages = Vec::new();
    for lp in locked {
        let cached = match cache.get(&lp.path, &lp.version) {
            Some(pkg) if cache.verify(&pkg).is_ok() => pkg,
            _ => fetch_and_store(cache, &lp.path, &lp.version, &lp.commit, opts.offline)?,
        };
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
    Ok(packages)
}

/// Check a cached package directory for a manifest with `[dependencies]`.
/// Returns `(dep_path, version_string)` pairs for transitive deps.
/// Errors if a sub-dep is already in the ancestor chain (cycle).
fn discover_transitive_deps(
    cached_dir: &Path,
    parent_path: &str,
    ancestors: &[String],
) -> anyhow::Result<Vec<(String, String)>> {
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

#[cfg(test)]
#[path = "install_tests.rs"]
mod tests;
