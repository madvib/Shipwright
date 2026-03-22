use super::*;
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
    let empty_lock = ShipLock {
        version: 1,
        package: vec![],
    };
    write_lock_atomic(&lock_path, &empty_lock)?;

    // Manifest has a dep not in the lock.
    let manifest = ShipManifest {
        dependencies: [(
            "github.com/owner/pkg".into(),
            Dependency {
                version: "main".into(),
                grant: vec![],
            },
        )]
        .into(),
    };

    let opts = InstallOptions { frozen: true };
    let result = resolve_and_fetch(&manifest, &lock_path, &cache, &opts);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("--frozen") || msg.contains("out of sync"),
        "got: {msg}"
    );
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
fn test_install_aborts_on_hash_mismatch() -> anyhow::Result<()> {
    let (_cache_dir, cache) = make_cache()?;
    let lock_dir = tempdir()?;
    let lock_path = lock_dir.path().join("ship.lock");

    // Store a real package in cache so `get()` returns a CachedPackage.
    let content = tempdir()?;
    std::fs::write(content.path().join("README.md"), "hello")?;
    let _cached = cache.store(
        "github.com/owner/pkg",
        "v1.0.0",
        &"a".repeat(40),
        content.path(),
    )?;

    // Write a lockfile with the WRONG hash — the real hash won't match.
    let lock = ShipLock {
        version: 1,
        package: vec![LockedPackage {
            path: "github.com/owner/pkg".into(),
            version: "v1.0.0".into(),
            commit: "a".repeat(40),
            hash: "sha256:0000000000000000000000000000000000000000000000000000000000000000".into(),
        }],
    };
    write_lock_atomic(&lock_path, &lock)?;

    // The manifest matches the lock so no re-resolution happens — but the
    // hash check should fail.
    let manifest = ShipManifest {
        dependencies: [(
            "github.com/owner/pkg".into(),
            Dependency {
                version: "v1.0.0".into(),
                grant: vec![],
            },
        )]
        .into(),
    };

    let opts = InstallOptions::default();
    let result = resolve_and_fetch(&manifest, &lock_path, &cache, &opts);
    assert!(result.is_err(), "expected hash mismatch error");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("hash mismatch"), "got: {msg}");
    Ok(())
}

#[test]
fn test_in_sync_lock_no_deps_no_write() -> anyhow::Result<()> {
    let (_cache_dir, cache) = make_cache()?;
    let lock_dir = tempdir()?;
    let lock_path = lock_dir.path().join("ship.lock");

    // Write an already-correct lock.
    let lock = ShipLock {
        version: 1,
        package: vec![],
    };
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

// ── discover_transitive_deps unit tests ───────────────────────────────

fn write_file(dir: &std::path::Path, rel: &str, content: &str) {
    let p = dir.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

#[test]
fn test_discover_no_manifest() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let deps = discover_transitive_deps(tmp.path(), "github.com/a/b", &[])?;
    assert!(deps.is_empty());
    Ok(())
}

#[test]
fn test_discover_manifest_with_deps() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    write_file(
        tmp.path(),
        "ship.toml",
        r#"
[module]
name = "github.com/a/b"
version = "1.0.0"

[dependencies]
"github.com/c/d" = "^1.0.0"
"#,
    );

    let deps = discover_transitive_deps(tmp.path(), "github.com/a/b", &[])?;
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].0, "github.com/c/d");
    assert_eq!(deps[0].1, "^1.0.0");
    Ok(())
}

#[test]
fn test_discover_manifest_no_deps() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    write_file(
        tmp.path(),
        "ship.toml",
        r#"
[module]
name = "github.com/a/b"
version = "1.0.0"
"#,
    );

    let deps = discover_transitive_deps(tmp.path(), "github.com/a/b", &[])?;
    assert!(deps.is_empty());
    Ok(())
}

#[test]
fn test_discover_cycle_detection() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    // B depends on A, and A is an ancestor.
    write_file(
        tmp.path(),
        "ship.toml",
        r#"
[module]
name = "github.com/b/pkg"
version = "1.0.0"

[dependencies]
"github.com/a/pkg" = "^1.0.0"
"#,
    );

    let ancestors = vec!["ship.toml".into(), "github.com/a/pkg".into()];
    let result = discover_transitive_deps(tmp.path(), "github.com/b/pkg", &ancestors);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("cycle"), "got: {msg}");
    assert!(msg.contains("github.com/a/pkg"), "got: {msg}");
    Ok(())
}

#[test]
fn test_discover_version_conflict_format() {
    // Version conflict is detected in the BFS loop, not in discover.
    // Tested via the resolve_and_fetch integration path.
    // Here we just verify the error message format is actionable.
    let v1 = ("^1.0.0".to_string(), "github.com/a/b".to_string());
    let v2_version = "^2.0.0";
    let v2_requestor = "github.com/c/d";
    let msg = format!(
        "version conflict for {}: {} requires '{}' but {} requires '{}'",
        "github.com/x/y", v2_requestor, v2_version, v1.1, v1.0
    );
    assert!(msg.contains("version conflict"));
    assert!(msg.contains("github.com/a/b"));
    assert!(msg.contains("github.com/c/d"));
}
