use std::path::Path;

use super::*;
use compiler::lockfile::{LockPackage, ShipLock};
use compiler::{Skill, SkillSource};
use tempfile::TempDir;

// ── is_dep_ref ────────────────────────────────────────────────────────

#[test]
fn is_dep_ref_github_prefix() {
    assert!(is_dep_ref("github.com/owner/pkg/skills/name"));
    assert!(is_dep_ref("github.com/a/b/skills/c"));
}

#[test]
fn is_dep_ref_local_refs_are_false() {
    assert!(!is_dep_ref("my-skill"));
    assert!(!is_dep_ref("review-pr"));
    assert!(!is_dep_ref(""));
    assert!(!is_dep_ref("skills/foo"));
}

// ── parse_dep_ref ─────────────────────────────────────────────────────

#[test]
fn parse_dep_ref_splits_correctly() {
    let (pkg, within) = parse_dep_ref("github.com/owner/pkg/skills/name").unwrap();
    assert_eq!(pkg, "github.com/owner/pkg");
    assert_eq!(within, "skills/name");
}

#[test]
fn parse_dep_ref_nested_within_path() {
    let (pkg, within) = parse_dep_ref("github.com/owner/repo/skills/foo/bar").unwrap();
    assert_eq!(pkg, "github.com/owner/repo");
    assert_eq!(within, "skills/foo/bar");
}

#[test]
fn parse_dep_ref_too_short_returns_none() {
    // Only package path, no within-path
    assert!(parse_dep_ref("github.com/owner/pkg").is_none());
    // No owner slash — only "github.com/skills/name" has 3 segments but
    // within_path is after the third segment, so with only two it's None.
    assert!(parse_dep_ref("github.com/only-two-segments").is_none());
}

#[test]
fn parse_dep_ref_non_github_returns_none() {
    assert!(parse_dep_ref("my-skill").is_none());
}

// ── hash_from_lock ────────────────────────────────────────────────────

fn make_lock(path: &str, hash: &str) -> ShipLock {
    ShipLock {
        version: 1,
        packages: vec![LockPackage {
            path: path.to_string(),
            version: "main".to_string(),
            commit: "a".repeat(40),
            hash: hash.to_string(),
        }],
    }
}

#[test]
fn hash_from_lock_found() {
    let lock = make_lock("github.com/owner/pkg", "sha256:abc123");
    let hex = hash_from_lock(&lock, "github.com/owner/pkg").unwrap();
    assert_eq!(hex, "abc123");
}

#[test]
fn hash_from_lock_missing_dep_errors() {
    let lock = make_lock("github.com/owner/pkg", "sha256:abc");
    let err = hash_from_lock(&lock, "github.com/other/repo").unwrap_err();
    assert!(err.to_string().contains("not in cache"), "got: {err}");
    assert!(err.to_string().contains("ship install"), "got: {err}");
}

#[test]
fn hash_from_lock_malformed_hash_errors() {
    let lock = make_lock("github.com/owner/pkg", "noshaeprefix");
    let err = hash_from_lock(&lock, "github.com/owner/pkg").unwrap_err();
    assert!(err.to_string().contains("malformed hash"), "got: {err}");
}

// ── cache_skill_path ──────────────────────────────────────────────────

#[test]
fn cache_skill_path_builds_correct_path() {
    let root = Path::new("/home/user/.ship/cache");
    let path = cache_skill_path(root, "abcdef", "skills/my-skill");
    assert_eq!(
        path,
        Path::new("/home/user/.ship/cache/objects/abcdef/skills/my-skill")
    );
}

// ── resolve_dep_skill (integration with temp filesystem) ─────────────

fn write_file(dir: &Path, rel: &str, content: &str) {
    let p = dir.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

#[test]
fn resolve_dep_skill_reads_skill_md() {
    let tmp = TempDir::new().unwrap();
    let hex = "deadbeef";
    write_file(
        tmp.path(),
        &format!("objects/{hex}/skills/my-skill/SKILL.md"),
        "---\nname: My Skill\ndescription: Does stuff\n---\n\nInstructions.",
    );

    let lock = make_lock("github.com/owner/pkg", &format!("sha256:{hex}"));
    let skill =
        resolve_dep_skill("github.com/owner/pkg/skills/my-skill", &lock, tmp.path()).unwrap();

    assert_eq!(skill.id, "github.com/owner/pkg/skills/my-skill");
    assert_eq!(skill.name, "My Skill");
    assert_eq!(skill.description.as_deref(), Some("Does stuff"));
    assert_eq!(skill.content, "Instructions.");
}

#[test]
fn resolve_dep_skill_missing_skill_lists_available() {
    let tmp = TempDir::new().unwrap();
    let hex = "deadbeef";
    // Create a different skill so the package exists
    write_file(
        tmp.path(),
        &format!("objects/{hex}/skills/other-skill/SKILL.md"),
        "content",
    );

    let lock = make_lock("github.com/owner/pkg", &format!("sha256:{hex}"));
    let err =
        resolve_dep_skill("github.com/owner/pkg/skills/missing", &lock, tmp.path()).unwrap_err();

    let msg = err.to_string();
    assert!(msg.contains("not found"), "got: {msg}");
    assert!(msg.contains("other-skill"), "got: {msg}");
}

#[test]
fn resolve_dep_skill_cache_miss_errors_with_install_hint() {
    let tmp = TempDir::new().unwrap();
    let lock = ShipLock { version: 1, packages: vec![] };

    let err =
        resolve_dep_skill("github.com/owner/pkg/skills/name", &lock, tmp.path()).unwrap_err();

    let msg = err.to_string();
    assert!(msg.contains("not in cache"), "got: {msg}");
    assert!(msg.contains("ship install"), "got: {msg}");
}

// ── resolve_dep_skills (batch) ────────────────────────────────────────

#[test]
fn resolve_dep_skills_skips_local_refs() {
    let tmp = TempDir::new().unwrap();
    // Lock file must exist but won't be read since there are no dep refs
    let lock_path = tmp.path().join("ship.lock");
    std::fs::write(&lock_path, "version = 1\n").unwrap();

    let refs = vec!["local-skill".to_string(), "another-local".to_string()];
    let result = resolve_dep_skills(&refs, &[], &lock_path, Some(tmp.path())).unwrap();
    assert!(result.is_empty(), "local refs must produce no dep skills");
}

#[test]
fn resolve_dep_skills_deduplicates_against_local() {
    let tmp = TempDir::new().unwrap();
    let hex = "cafebabe";
    write_file(
        tmp.path(),
        &format!("objects/{hex}/skills/my-skill/SKILL.md"),
        "content",
    );

    let lock_path = tmp.path().join("ship.lock");
    std::fs::write(
        &lock_path,
        format!(
            "version = 1\n\n[[package]]\npath = \"github.com/owner/pkg\"\n\
             version = \"main\"\ncommit = \"{}\"\nhash = \"sha256:{hex}\"\n",
            "a".repeat(40)
        ),
    )
    .unwrap();

    // Simulate a local skill that already has the dep ref as its id
    let existing = Skill {
        id: "github.com/owner/pkg/skills/my-skill".to_string(),
        name: "Local".to_string(),
        description: None,
        version: None,
        author: None,
        content: "local content".to_string(),
        source: SkillSource::Custom,
    };

    let refs = vec!["github.com/owner/pkg/skills/my-skill".to_string()];
    let result = resolve_dep_skills(&refs, &[existing], &lock_path, Some(tmp.path())).unwrap();
    assert!(result.is_empty(), "already-present skill must not be duplicated");
}

#[test]
fn resolve_dep_skills_merges_dep_with_local() {
    let tmp = TempDir::new().unwrap();
    let hex = "cafebabe";
    write_file(
        tmp.path(),
        &format!("objects/{hex}/skills/cool-skill/SKILL.md"),
        "---\nname: Cool\n---\n\nCool content.",
    );

    let lock_path = tmp.path().join("ship.lock");
    std::fs::write(
        &lock_path,
        format!(
            "version = 1\n\n[[package]]\npath = \"github.com/owner/pkg\"\n\
             version = \"main\"\ncommit = \"{}\"\nhash = \"sha256:{hex}\"\n",
            "a".repeat(40)
        ),
    )
    .unwrap();

    let refs = vec![
        "local-skill".to_string(),
        "github.com/owner/pkg/skills/cool-skill".to_string(),
    ];
    let result = resolve_dep_skills(&refs, &[], &lock_path, Some(tmp.path())).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, "github.com/owner/pkg/skills/cool-skill");
    assert_eq!(result[0].content, "Cool content.");
}
