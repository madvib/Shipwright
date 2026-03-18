use super::*;
use std::fs;
use tempfile::tempdir;

fn make_content_dir(content: &[(&str, &str)]) -> anyhow::Result<tempfile::TempDir> {
    let dir = tempdir()?;
    for (name, data) in content {
        fs::write(dir.path().join(name), data)?;
    }
    Ok(dir)
}

#[test]
fn test_url_encode() {
    assert_eq!(url_encode("github.com/owner/repo"), "github.com%2Fowner%2Frepo");
    assert_eq!(url_encode("simple"), "simple");
    assert_eq!(url_encode("feat/branch"), "feat%2Fbranch");
}

#[test]
fn test_store_and_get_round_trip() -> anyhow::Result<()> {
    let cache_root = tempdir()?;
    let cache = PackageCache::with_root(cache_root.path().to_path_buf());

    let content = make_content_dir(&[("SKILL.md", "# Hello"), ("readme.txt", "world")])?;
    let stored = cache.store(
        "github.com/owner/pkg",
        "v1.0.0",
        "a".repeat(40).as_str(),
        content.path(),
    )?;

    assert!(stored.hash.starts_with("sha256:"));
    assert!(stored.dir.is_dir());

    let retrieved = cache.get("github.com/owner/pkg", "v1.0.0").expect("cache hit");
    assert_eq!(retrieved.hash, stored.hash);
    assert_eq!(retrieved.commit, "a".repeat(40));
    Ok(())
}

#[test]
fn test_cache_miss_returns_none() -> anyhow::Result<()> {
    let cache_root = tempdir()?;
    let cache = PackageCache::with_root(cache_root.path().to_path_buf());

    let result = cache.get("github.com/missing/pkg", "v1.0.0");
    assert!(result.is_none());
    Ok(())
}

#[test]
fn test_verify_passes_on_clean_cache() -> anyhow::Result<()> {
    let cache_root = tempdir()?;
    let cache = PackageCache::with_root(cache_root.path().to_path_buf());

    let content = make_content_dir(&[("a.txt", "data")])?;
    let stored =
        cache.store("github.com/owner/pkg", "v1.0.0", &"b".repeat(40), content.path())?;

    cache.verify(&stored)?;
    Ok(())
}

#[test]
fn test_verify_detects_corruption() -> anyhow::Result<()> {
    let cache_root = tempdir()?;
    let cache = PackageCache::with_root(cache_root.path().to_path_buf());

    let content = make_content_dir(&[("a.txt", "original")])?;
    let stored =
        cache.store("github.com/owner/pkg", "v1.0.0", &"c".repeat(40), content.path())?;

    // Corrupt the stored file.
    fs::write(stored.dir.join("a.txt"), "tampered")?;

    let result = cache.verify(&stored);
    assert!(result.is_err());

    // Index entry should be deleted; next get() returns None.
    let hit = cache.get("github.com/owner/pkg", "v1.0.0");
    assert!(hit.is_none(), "corrupted entry should be evicted from index");
    Ok(())
}

#[test]
fn test_store_idempotent() -> anyhow::Result<()> {
    let cache_root = tempdir()?;
    let cache = PackageCache::with_root(cache_root.path().to_path_buf());

    let content = make_content_dir(&[("f.txt", "data")])?;
    let commit = "d".repeat(40);

    let s1 = cache.store("github.com/owner/pkg", "v1.0.0", &commit, content.path())?;
    let s2 = cache.store("github.com/owner/pkg", "v1.0.0", &commit, content.path())?;

    assert_eq!(s1.hash, s2.hash);
    Ok(())
}
