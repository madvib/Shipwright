use super::*;
use std::fs;
use tempfile::tempdir;

/// Helper: create a temp dir with the given files.
fn make_tree(files: &[(&str, &str)]) -> anyhow::Result<tempfile::TempDir> {
    let dir = tempdir()?;
    for (path, content) in files {
        let full = dir.path().join(path);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(full, content)?;
    }
    Ok(dir)
}

// ── compute_tree_hash ───────────────────────────────────────────────

#[test]
fn tree_hash_deterministic_on_known_files() -> anyhow::Result<()> {
    let dir = make_tree(&[("a.txt", "hello"), ("b.txt", "world")])?;
    let h1 = compute_tree_hash(dir.path())?;
    let h2 = compute_tree_hash(dir.path())?;
    assert!(h1.starts_with("sha256:"));
    assert_eq!(h1, h2, "same tree must produce same hash");
    Ok(())
}

#[test]
fn tree_hash_excludes_git_dir() -> anyhow::Result<()> {
    let dir = make_tree(&[("a.txt", "data")])?;
    let h_before = compute_tree_hash(dir.path())?;
    fs::create_dir_all(dir.path().join(".git/objects"))?;
    fs::write(dir.path().join(".git/HEAD"), "ref: refs/heads/main")?;
    fs::write(dir.path().join(".git/objects/abc"), "blob")?;
    let h_after = compute_tree_hash(dir.path())?;
    assert_eq!(h_before, h_after, ".git/ must be excluded from hash");
    Ok(())
}

#[test]
fn tree_hash_excludes_ds_store_and_thumbs_db() -> anyhow::Result<()> {
    let dir = make_tree(&[("a.txt", "data")])?;
    let h_before = compute_tree_hash(dir.path())?;
    fs::write(dir.path().join(".DS_Store"), "apple noise")?;
    fs::write(dir.path().join("Thumbs.db"), "windows noise")?;
    let h_after = compute_tree_hash(dir.path())?;
    assert_eq!(h_before, h_after, ".DS_Store and Thumbs.db must be excluded");
    Ok(())
}

#[test]
fn tree_hash_excludes_ship_lock() -> anyhow::Result<()> {
    let dir = make_tree(&[("a.txt", "data")])?;
    let h_before = compute_tree_hash(dir.path())?;
    fs::write(dir.path().join("ship.lock"), "lock content")?;
    let h_after = compute_tree_hash(dir.path())?;
    assert_eq!(h_before, h_after, "ship.lock must be excluded");
    Ok(())
}

#[test]
fn tree_hash_excludes_swp_files() -> anyhow::Result<()> {
    let dir = make_tree(&[("a.txt", "data")])?;
    let h_before = compute_tree_hash(dir.path())?;
    fs::write(dir.path().join(".a.txt.swp"), "vim swap")?;
    fs::create_dir_all(dir.path().join("sub"))?;
    fs::write(dir.path().join("sub/.b.swp"), "")?;
    let h_after = compute_tree_hash(dir.path())?;
    assert_eq!(h_before, h_after, "*.swp files must be excluded");
    Ok(())
}

#[test]
fn tree_hash_order_independent() -> anyhow::Result<()> {
    let dir1 = make_tree(&[("z.txt", "last"), ("a.txt", "first"), ("m/b.txt", "mid")])?;
    let dir2 = make_tree(&[("a.txt", "first"), ("m/b.txt", "mid"), ("z.txt", "last")])?;
    let h1 = compute_tree_hash(dir1.path())?;
    let h2 = compute_tree_hash(dir2.path())?;
    assert_eq!(h1, h2, "hash must not depend on file creation order");
    Ok(())
}

#[test]
fn tree_hash_changes_when_content_changes() -> anyhow::Result<()> {
    let dir = make_tree(&[("a.txt", "original")])?;
    let h_before = compute_tree_hash(dir.path())?;
    fs::write(dir.path().join("a.txt"), "modified")?;
    let h_after = compute_tree_hash(dir.path())?;
    assert_ne!(h_before, h_after, "content change must change hash");
    Ok(())
}

#[test]
fn tree_hash_empty_dir() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let hash = compute_tree_hash(dir.path())?;
    assert!(hash.starts_with("sha256:"), "empty tree still returns valid hash");
    Ok(())
}

// ── compute_file_hash ───────────────────────────────────────────────

#[test]
fn file_hash_known_content() -> anyhow::Result<()> {
    let dir = make_tree(&[("hello.txt", "hello")])?;
    let hash = compute_file_hash(&dir.path().join("hello.txt"))?;
    // SHA-256("hello") is a well-known constant.
    assert_eq!(
        hash,
        "sha256:2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    );
    Ok(())
}

#[test]
fn file_hash_missing_file_fails() {
    let dir = tempdir().unwrap();
    let result = compute_file_hash(&dir.path().join("nonexistent.txt"));
    assert!(result.is_err());
}

// ── compute_export_hashes ───────────────────────────────────────────

#[test]
fn export_hashes_skills_and_agents() -> anyhow::Result<()> {
    let ship_dir = make_tree(&[
        ("skills/my-skill/SKILL.md", "# My Skill"),
        ("skills/my-skill/rules.md", "some rules"),
        ("agents/my-agent.md", "# My Agent"),
    ])?;
    let result = compute_export_hashes(
        ship_dir.path(),
        &["skills/my-skill".into()],
        &["agents/my-agent.md".into()],
    )?;
    assert_eq!(result.per_export.len(), 2);
    assert!(result.per_export.contains_key("skills/my-skill"));
    assert!(result.per_export.contains_key("agents/my-agent.md"));
    for hash in result.per_export.values() {
        assert!(hash.starts_with("sha256:"));
    }
    assert!(result.combined.starts_with("sha256:"));
    Ok(())
}

#[test]
fn export_hashes_missing_skill_dir_fails() {
    let ship_dir = tempdir().unwrap();
    let result = compute_export_hashes(ship_dir.path(), &["skills/missing".into()], &[]);
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("not found"), "error should mention 'not found': {msg}");
}

#[test]
fn export_hashes_missing_agent_file_fails() {
    let ship_dir = tempdir().unwrap();
    let result = compute_export_hashes(ship_dir.path(), &[], &["agents/missing.md".into()]);
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("not found"), "error should mention 'not found': {msg}");
}

#[test]
fn export_hashes_empty_exports() -> anyhow::Result<()> {
    let ship_dir = tempdir()?;
    let result = compute_export_hashes(ship_dir.path(), &[], &[])?;
    assert!(result.per_export.is_empty());
    assert!(result.combined.starts_with("sha256:"));
    Ok(())
}

// ── Combined hash determinism ───────────────────────────────────────

#[test]
fn combined_hash_deterministic() -> anyhow::Result<()> {
    let ship_dir = make_tree(&[
        ("skills/alpha/SKILL.md", "a"),
        ("skills/beta/SKILL.md", "b"),
    ])?;
    let exports = vec!["skills/alpha".to_string(), "skills/beta".to_string()];
    let r1 = compute_export_hashes(ship_dir.path(), &exports, &[])?;
    let r2 = compute_export_hashes(ship_dir.path(), &exports, &[])?;
    assert_eq!(r1, r2, "repeated calls must produce identical results");
    Ok(())
}

#[test]
fn combined_hash_changes_when_export_added() -> anyhow::Result<()> {
    let ship_dir = make_tree(&[
        ("skills/alpha/SKILL.md", "a"),
        ("skills/beta/SKILL.md", "b"),
    ])?;
    let r1 = compute_export_hashes(ship_dir.path(), &["skills/alpha".into()], &[])?;
    let r2 = compute_export_hashes(
        ship_dir.path(),
        &["skills/alpha".into(), "skills/beta".into()],
        &[],
    )?;
    assert_ne!(r1.combined, r2.combined, "adding an export must change combined hash");
    Ok(())
}

#[test]
fn combined_hash_changes_when_export_content_changes() -> anyhow::Result<()> {
    let ship_dir = make_tree(&[("skills/alpha/SKILL.md", "original")])?;
    let r1 = compute_export_hashes(ship_dir.path(), &["skills/alpha".into()], &[])?;
    fs::write(ship_dir.path().join("skills/alpha/SKILL.md"), "changed")?;
    let r2 = compute_export_hashes(ship_dir.path(), &["skills/alpha".into()], &[])?;
    assert_ne!(
        r1.per_export["skills/alpha"], r2.per_export["skills/alpha"],
        "content change in skill must change its per-export hash"
    );
    assert_ne!(r1.combined, r2.combined, "content change must change combined hash");
    Ok(())
}

// ── Internal changes affect only their own export ───────────────────

#[test]
fn internal_change_does_not_affect_other_exports() -> anyhow::Result<()> {
    let ship_dir = make_tree(&[
        ("skills/alpha/SKILL.md", "a"),
        ("skills/beta/SKILL.md", "b"),
    ])?;
    let exports = vec!["skills/alpha".to_string(), "skills/beta".to_string()];
    let r1 = compute_export_hashes(ship_dir.path(), &exports, &[])?;
    fs::write(ship_dir.path().join("skills/alpha/SKILL.md"), "a-changed")?;
    let r2 = compute_export_hashes(ship_dir.path(), &exports, &[])?;
    assert_ne!(
        r1.per_export["skills/alpha"], r2.per_export["skills/alpha"],
        "alpha hash must change"
    );
    assert_eq!(
        r1.per_export["skills/beta"], r2.per_export["skills/beta"],
        "beta hash must NOT change when only alpha changed"
    );
    Ok(())
}

// ── should_exclude edge cases via tree_hash behavior ────────────────

#[test]
fn nested_ds_store_excluded() -> anyhow::Result<()> {
    let dir = make_tree(&[("a.txt", "data")])?;
    let h_before = compute_tree_hash(dir.path())?;
    fs::create_dir_all(dir.path().join("sub/deep"))?;
    fs::write(dir.path().join("sub/deep/.DS_Store"), "nested")?;
    let h_after = compute_tree_hash(dir.path())?;
    assert_eq!(h_before, h_after, "nested .DS_Store must be excluded");
    Ok(())
}

#[test]
fn ship_lock_only_at_root_excluded() -> anyhow::Result<()> {
    // ship.lock at root is excluded; ship.lock inside a subdirectory is NOT
    // excluded (rel_path != "ship.lock").
    let dir_without = make_tree(&[("a.txt", "data")])?;
    let h_without = compute_tree_hash(dir_without.path())?;
    let dir_with_nested = make_tree(&[("a.txt", "data"), ("sub/ship.lock", "nested lock")])?;
    let h_with_nested = compute_tree_hash(dir_with_nested.path())?;
    assert_ne!(
        h_without, h_with_nested,
        "sub/ship.lock is NOT excluded, so hash should differ"
    );
    Ok(())
}
