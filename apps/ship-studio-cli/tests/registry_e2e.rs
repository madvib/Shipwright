//! End-to-end tests for the Ship registry pipeline.
//!
//! Covers the full path from dependency resolution through provider output.
//! No real network calls — local bare git repos are used as fixtures.
//!
//! Test strategy:
//! - For dep skill resolution and compile: construct ProjectLibrary in-memory,
//!   pre-populate the cache, and call the public library APIs directly.
//! - For install pipeline: call resolve_and_fetch with a pre-populated cache
//!   so no git fetch is triggered.
//! - For the git fetch path: use fetch_package_content with a file:// URL.
//!
//! File layout:
//! - `fixtures/mod.rs`       — bare git repo and cache fixture helpers
//! - `registry_e2e_deps.rs`  — dep skill and compile tests (Tests 1-4, bonus)
//! - `registry_e2e.rs`       — install and workspace tests (Tests 5-8)

mod fixtures;

use std::fs;
use tempfile::TempDir;

use compiler::lockfile::ShipLock;
use runtime::registry::cache::PackageCache;

use fixtures::{write, write_lock};

// ═══════════════════════════════════════════════════════════════════════════════
// Test 5: Idempotent install
// ═══════════════════════════════════════════════════════════════════════════════
//
// resolve_and_fetch run twice: second run must NOT rewrite the lock.

#[test]
fn install_idempotent_second_run_does_not_rewrite_lock() {
    use runtime::registry::install::{InstallOptions, resolve_and_fetch};
    use runtime::registry::types::{Dependency, ShipManifest as RegistryManifest};

    let tmp = TempDir::new().unwrap();
    let cache_tmp = TempDir::new().unwrap();

    let content_dir = tmp.path().join("pkg-content");
    write(&content_dir, "skills/s/SKILL.md", "# S\nStay the same.");

    let cache = PackageCache::with_root(cache_tmp.path().to_path_buf());
    let commit = "c".repeat(40);
    let pkg = cache
        .store("github.com/owner/idempkg", &commit, &commit, &content_dir)
        .expect("cache store must succeed");

    let lock_path = tmp.path().join("ship.lock");
    write_lock(&lock_path, "github.com/owner/idempkg", &commit, &pkg.hash);

    let mtime_before = fs::metadata(&lock_path).unwrap().modified().unwrap();

    let mut manifest = RegistryManifest::default();
    manifest.dependencies.insert(
        "github.com/owner/idempkg".to_string(),
        Dependency {
            version: commit.clone(),
            grant: vec![],
        },
    );

    let opts = InstallOptions::default();

    // First run: cache hit + lock in sync = no rewrite.
    let r1 = resolve_and_fetch(&manifest, &lock_path, &cache, &opts)
        .expect("first install must succeed");
    assert!(
        !r1.lockfile_written,
        "first run with matching lock+cache must be a no-op"
    );

    let mtime_after_first = fs::metadata(&lock_path).unwrap().modified().unwrap();
    assert_eq!(
        mtime_before, mtime_after_first,
        "lock mtime must not change on first run"
    );

    // Second run: same result.
    let r2 = resolve_and_fetch(&manifest, &lock_path, &cache, &opts)
        .expect("second install must succeed");
    assert!(!r2.lockfile_written, "second run must also be a no-op");

    let mtime_after_second = fs::metadata(&lock_path).unwrap().modified().unwrap();
    assert_eq!(
        mtime_before, mtime_after_second,
        "lock mtime unchanged after second run"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test 6: Frozen install fails when lock would change
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn frozen_install_fails_when_lock_is_stale() {
    use runtime::registry::install::{InstallOptions, resolve_and_fetch};
    use runtime::registry::types::{Dependency, ShipManifest as RegistryManifest};

    let tmp = TempDir::new().unwrap();
    let cache_tmp = TempDir::new().unwrap();
    let cache = PackageCache::with_root(cache_tmp.path().to_path_buf());

    // Write empty lock (no packages).
    let lock_path = tmp.path().join("ship.lock");
    ShipLock::default().write_atomic(&lock_path).unwrap();

    // Manifest declares a dep not in the lock.
    let mut manifest = RegistryManifest::default();
    manifest.dependencies.insert(
        "github.com/owner/pkg".to_string(),
        Dependency {
            version: "main".to_string(),
            grant: vec![],
        },
    );

    let opts = InstallOptions {
        frozen: true,
        offline: false,
    };
    let err = resolve_and_fetch(&manifest, &lock_path, &cache, &opts).unwrap_err();
    let chain = format!("{:#}", err);
    assert!(
        chain.contains("frozen") || chain.contains("out of sync"),
        "error must mention frozen/out-of-sync; got:\n{chain}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test 7: Install writes lock with correct package entry
// ═══════════════════════════════════════════════════════════════════════════════
//
// resolve_and_fetch with a cache-prepopulated package (commit SHA constraint).
// Verifies that the lock is written with the correct entry and hash.

#[test]
fn install_writes_lock_with_correct_package_entry() {
    use runtime::registry::install::{InstallOptions, resolve_and_fetch};
    use runtime::registry::types::{Dependency, ShipManifest as RegistryManifest};

    let tmp = TempDir::new().unwrap();
    let cache_tmp = TempDir::new().unwrap();

    // Pre-populate cache — install will find a cache hit and skip git fetch.
    let content_dir = tmp.path().join("pkg-content");
    write(
        &content_dir,
        "skills/my-skill/SKILL.md",
        "# My Skill\nBe excellent.",
    );

    let cache = PackageCache::with_root(cache_tmp.path().to_path_buf());
    let commit = "d".repeat(40);
    let pkg = cache
        .store("github.com/owner/pkg", &commit, &commit, &content_dir)
        .expect("cache store must succeed");

    // Commit SHA constraint bypasses git ls-remote.
    let mut manifest = RegistryManifest::default();
    manifest.dependencies.insert(
        "github.com/owner/pkg".to_string(),
        Dependency {
            version: commit.clone(),
            grant: vec![],
        },
    );

    let lock_path = tmp.path().join("ship.lock");
    let result = resolve_and_fetch(&manifest, &lock_path, &cache, &InstallOptions::default())
        .expect("install must succeed");

    assert!(
        result.lockfile_written,
        "lock must be written on first install"
    );
    assert!(lock_path.exists(), "ship.lock must exist after install");

    let lock = ShipLock::from_file(&lock_path).expect("lock must be parseable");
    assert_eq!(lock.version, 1);
    assert_eq!(lock.packages.len(), 1);

    let lp = &lock.packages[0];
    assert_eq!(lp.path, "github.com/owner/pkg");
    assert_eq!(lp.commit, commit);
    assert_eq!(lp.hash, pkg.hash, "hash in lock must match cache entry");
    assert!(
        lp.hash.starts_with("sha256:"),
        "hash must have sha256: prefix; got: {}",
        lp.hash
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test 8: WorkspaceState round trip via runtime::db::kv
// ═══════════════════════════════════════════════════════════════════════════════
//
// active_profile written to platform.db, read back correctly.
// Second write is idempotent.

#[test]
fn workspace_state_active_profile_round_trip() {
    use runtime::db;
    use runtime::project::init_project;

    let tmp = TempDir::new().unwrap();
    // Use init_project so ship.toml gets a stable unique nanoid — avoids
    // the "tmp-test-test" slug collision that would occur with a hardcoded id.
    let ship_dir = init_project(tmp.path().to_path_buf()).expect("init_project must succeed");

    db::ensure_db().expect("ensure_db must succeed");

    // First write.
    db::kv::set(
        "workspace",
        "active_profile",
        &serde_json::json!("my-profile"),
    )
    .expect("kv set must succeed");

    let val = db::kv::get("workspace", "active_profile")
        .expect("kv get must succeed")
        .expect("value must be present");
    assert_eq!(
        val.as_str(),
        Some("my-profile"),
        "active_profile must round-trip through platform.db"
    );

    // Second write — same value, idempotent.
    db::kv::set(
        "workspace",
        "active_profile",
        &serde_json::json!("my-profile"),
    )
    .expect("second kv set must succeed");

    let val2 = db::kv::get("workspace", "active_profile")
        .expect("kv get after second write must succeed")
        .expect("value must still be present");
    assert_eq!(
        val2.as_str(),
        Some("my-profile"),
        "active_profile must be stable after idempotent write"
    );
}
