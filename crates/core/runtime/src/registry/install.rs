use std::path::Path;

use anyhow::Context;

use super::cache::{CachedPackage, PackageCache};
use super::constraint::parse_constraint;
use super::fetch::fetch_package_content;
use super::resolver::resolve_version;
use super::types::{ShipLock, ShipManifest, LockedPackage, parse_ship_lock, serialize_ship_lock};

/// Options passed to `resolve_and_fetch`.
#[derive(Debug, Default)]
pub struct InstallOptions {
    /// If `true`, fail when the lockfile would change rather than updating it.
    pub frozen: bool,
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

    for (dep_path, dep) in deps_to_resolve {
        let constraint = parse_constraint(&dep.version)
            .with_context(|| format!("parsing constraint for {dep_path}"))?;

        let resolved = resolve_version(dep_path, &constraint)
            .with_context(|| format!("resolving {dep_path}"))?;

        // Fetch if not cached.
        let cached = match cache.get(dep_path, &resolved.tag) {
            Some(pkg) => {
                // Verify before trusting the cache.
                if cache.verify(&pkg).is_ok() {
                    pkg
                } else {
                    fetch_and_store(cache, dep_path, &resolved.tag, &resolved.commit)?
                }
            }
            None => fetch_and_store(cache, dep_path, &resolved.tag, &resolved.commit)?,
        };

        // Record in lock.
        locked.retain(|p| p.path != *dep_path);
        locked.push(LockedPackage {
            path: dep_path.clone(),
            version: resolved.tag.clone(),
            commit: resolved.commit.clone(),
            hash: cached.hash.clone(),
        });
    }

    // For deps already in lock (unchanged), ensure they are in the cache.
    let mut packages: Vec<CachedPackage> = Vec::new();
    for lp in &locked {
        let cached = match cache.get(&lp.path, &lp.version) {
            Some(pkg) if cache.verify(&pkg).is_ok() => pkg,
            _ => {
                fetch_and_store(cache, &lp.path, &lp.version, &lp.commit)?
            }
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
        write_lock_atomic(lock_path, &new_lock)
            .context("writing ship.lock")?;
    }

    Ok(InstallResult { packages, lockfile_written })
}

fn load_lock_if_present(path: &Path) -> anyhow::Result<Option<ShipLock>> {
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?;
    let lock = parse_ship_lock(&content)
        .with_context(|| format!("parsing {}", path.display()))?;
    Ok(Some(lock))
}

fn fetch_and_store(
    cache: &PackageCache,
    dep_path: &str,
    version: &str,
    commit: &str,
) -> anyhow::Result<CachedPackage> {
    let tmp = tempfile::tempdir()
        .context("creating tempdir for package fetch")?;
    let git_url = format!("https://{}.git", dep_path);
    fetch_package_content(&git_url, commit, tmp.path())
        .with_context(|| format!("fetching {dep_path} @ {commit}"))?;
    cache
        .store(dep_path, version, commit, tmp.path())
        .with_context(|| format!("storing {dep_path}@{version} in cache"))
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
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::tempdir;

    use super::super::types::Dependency;

    fn make_cache() -> anyhow::Result<(tempfile::TempDir, PackageCache)> {
        let dir = tempdir()?;
        let cache = PackageCache::with_root(dir.path().to_path_buf());
        Ok((dir, cache))
    }

    #[test]
    fn test_frozen_fails_on_added_dep() -> anyhow::Result<()> {
        let (_cache_dir, cache) = make_cache()?;
        let lock_dir = tempdir()?;
        let lock_path = lock_dir.path().join("ship.lock");

        // Write an empty lock.
        let empty_lock = ShipLock { version: 1, package: vec![] };
        write_lock_atomic(&lock_path, &empty_lock)?;

        // Manifest has a dep not in the lock.
        let mut manifest = ShipManifest::default();
        manifest.dependencies.insert(
            "github.com/owner/pkg".into(),
            Dependency { version: "main".into(), grant: vec![] },
        );

        let opts = InstallOptions { frozen: true };
        let result = resolve_and_fetch(&manifest, &lock_path, &cache, &opts);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("--frozen") || msg.contains("out of sync"), "got: {msg}");
        Ok(())
    }

    #[test]
    fn test_no_lock_no_deps_writes_empty_lock() -> anyhow::Result<()> {
        let (_cache_dir, cache) = make_cache()?;
        let lock_dir = tempdir()?;
        let lock_path = lock_dir.path().join("ship.lock");

        let manifest = ShipManifest::default();
        let opts = InstallOptions::default();
        let result = resolve_and_fetch(&manifest, &lock_path, &cache, &opts)?;

        assert!(result.lockfile_written);
        assert!(lock_path.exists());
        let content = std::fs::read_to_string(&lock_path)?;
        assert!(content.contains("version = 1"));
        Ok(())
    }

    #[test]
    fn test_in_sync_lock_no_deps_no_write() -> anyhow::Result<()> {
        let (_cache_dir, cache) = make_cache()?;
        let lock_dir = tempdir()?;
        let lock_path = lock_dir.path().join("ship.lock");

        // Write an already-correct lock.
        let lock = ShipLock { version: 1, package: vec![] };
        write_lock_atomic(&lock_path, &lock)?;
        let mtime_before = std::fs::metadata(&lock_path)?.modified()?;

        let manifest = ShipManifest::default();
        let opts = InstallOptions::default();
        let result = resolve_and_fetch(&manifest, &lock_path, &cache, &opts)?;

        assert!(!result.lockfile_written);
        let mtime_after = std::fs::metadata(&lock_path)?.modified()?;
        assert_eq!(mtime_before, mtime_after, "lock file must not be rewritten");
        Ok(())
    }
}
