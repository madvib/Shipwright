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
fn is_dep_ref_any_git_host() {
    assert!(is_dep_ref("gitlab.com/owner/pkg/skills/name"));
    assert!(is_dep_ref("codeberg.org/owner/pkg/skills/name"));
    assert!(is_dep_ref("git.example.com/owner/pkg/skill-name"));
}

#[test]
fn is_dep_ref_local_refs_are_false() {
    assert!(!is_dep_ref("my-skill"));
    assert!(!is_dep_ref("review-pr"));
    assert!(!is_dep_ref(""));
    assert!(!is_dep_ref("skills/foo"));
    // No dot in first segment — not a host
    assert!(!is_dep_ref("owner/pkg/skill"));
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

#[test]
fn parse_dep_ref_gitlab() {
    let (pkg, within) = parse_dep_ref("gitlab.com/owner/pkg/skills/name").unwrap();
    assert_eq!(pkg, "gitlab.com/owner/pkg");
    assert_eq!(within, "skills/name");
}

#[test]
fn parse_dep_ref_codeberg() {
    let (pkg, within) = parse_dep_ref("codeberg.org/owner/pkg/skills/name").unwrap();
    assert_eq!(pkg, "codeberg.org/owner/pkg");
    assert_eq!(within, "skills/name");
}

#[test]
fn parse_dep_ref_self_hosted() {
    let (pkg, within) = parse_dep_ref("git.example.com/owner/pkg/skill-name").unwrap();
    assert_eq!(pkg, "git.example.com/owner/pkg");
    assert_eq!(within, "skill-name");
}

#[test]
fn parse_dep_ref_no_dot_in_host_returns_none() {
    // First segment has no dot — not a valid host
    assert!(parse_dep_ref("owner/pkg/skill").is_none());
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
            export_hashes: Default::default(),
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
    let skills =
        resolve_dep_skill("github.com/owner/pkg/skills/my-skill", &lock, tmp.path()).unwrap();

    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].id, "github.com/owner/pkg/skills/my-skill");
    assert_eq!(skills[0].name, "My Skill");
    assert_eq!(skills[0].description.as_deref(), Some("Does stuff"));
    assert_eq!(skills[0].content, "Instructions.");
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
fn resolve_dep_skill_expands_namespace_to_leaf_skills() {
    // Real cache layout: better-auth/ is a namespace containing sub-skills,
    // NOT a skill itself (no better-auth/SKILL.md).
    let tmp = TempDir::new().unwrap();
    let hex = "83fb025b";
    write_file(
        tmp.path(),
        &format!("objects/{hex}/better-auth/best-practices/SKILL.md"),
        "---\nname: Best Practices\n---\n\nBP content.",
    );
    write_file(
        tmp.path(),
        &format!("objects/{hex}/better-auth/create-auth/SKILL.md"),
        "---\nname: Create Auth\n---\n\nCA content.",
    );
    write_file(
        tmp.path(),
        &format!("objects/{hex}/better-auth/emailAndPassword/SKILL.md"),
        "---\nname: Email and Password\n---\n\nEP content.",
    );
    // Also create a sibling namespace to verify we only expand the target
    write_file(
        tmp.path(),
        &format!("objects/{hex}/security/hardening/SKILL.md"),
        "---\nname: Hardening\n---\n\nH content.",
    );

    let lock = make_lock("github.com/better-auth/skills", &format!("sha256:{hex}"));
    let skills = resolve_dep_skill(
        "github.com/better-auth/skills/better-auth",
        &lock,
        tmp.path(),
    )
    .unwrap();

    assert_eq!(skills.len(), 3, "should expand to 3 leaf skills");
    // Sorted by sub-name
    assert_eq!(
        skills[0].id,
        "github.com/better-auth/skills/better-auth/best-practices"
    );
    assert_eq!(skills[0].name, "Best Practices");
    assert_eq!(
        skills[1].id,
        "github.com/better-auth/skills/better-auth/create-auth"
    );
    assert_eq!(
        skills[2].id,
        "github.com/better-auth/skills/better-auth/emailAndPassword"
    );
}

#[test]
fn resolve_dep_skill_missing_namespace_lists_available() {
    // When within_path doesn't match any directory at all, error lists siblings.
    let tmp = TempDir::new().unwrap();
    let hex = "deadbeef";
    write_file(
        tmp.path(),
        &format!("objects/{hex}/better-auth/create-auth/SKILL.md"),
        "content",
    );
    write_file(
        tmp.path(),
        &format!("objects/{hex}/security/hardening/SKILL.md"),
        "content",
    );

    let lock = make_lock("github.com/better-auth/skills", &format!("sha256:{hex}"));
    let err = resolve_dep_skill(
        "github.com/better-auth/skills/missing-ns",
        &lock,
        tmp.path(),
    )
    .unwrap_err();

    let msg = err.to_string();
    assert!(msg.contains("not found"), "got: {msg}");
    assert!(
        msg.contains("better-auth"),
        "should list sibling dirs, got: {msg}"
    );
    assert!(
        msg.contains("security"),
        "should list sibling dirs, got: {msg}"
    );
}

#[test]
fn resolve_dep_skill_cache_miss_errors_with_install_hint() {
    let tmp = TempDir::new().unwrap();
    let lock = ShipLock {
        version: 1,
        packages: vec![],
    };

    let err = resolve_dep_skill("github.com/owner/pkg/skills/name", &lock, tmp.path()).unwrap_err();

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
        stable_id: None,
        description: None,
        license: None,
        compatibility: None,
        allowed_tools: vec![],
        metadata: Default::default(),
        content: "local content".to_string(),
        source: SkillSource::Custom,
        vars: Default::default(),
    };

    let refs = vec!["github.com/owner/pkg/skills/my-skill".to_string()];
    let result = resolve_dep_skills(&refs, &[existing], &lock_path, Some(tmp.path())).unwrap();
    assert!(
        result.is_empty(),
        "already-present skill must not be duplicated"
    );
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

// ── parse_dep_skill spec fields ───────────────────────────────────────

#[test]
fn parse_dep_skill_parses_license_and_compatibility() {
    let raw = "---\nname: Cool\ndescription: x\nlicense: MIT\ncompatibility: claude >= 3\n---\n\nContent.";
    let skill = parse_dep_skill("github.com/owner/pkg/skills/cool", raw);
    assert_eq!(skill.license.as_deref(), Some("MIT"));
    assert_eq!(skill.compatibility.as_deref(), Some("claude >= 3"));
}

#[test]
fn parse_dep_skill_parses_allowed_tools() {
    let raw = "---\nname: Cool\ndescription: x\nallowed-tools: Read Edit\n---\n\nContent.";
    let skill = parse_dep_skill("github.com/owner/pkg/skills/cool", raw);
    assert_eq!(skill.allowed_tools, vec!["Read", "Edit"]);
}

#[test]
fn parse_dep_skill_parses_metadata_block() {
    let raw = "---\nname: Cool\ndescription: x\nmetadata:\n  team: platform\n---\n\nContent.";
    let skill = parse_dep_skill("github.com/owner/pkg/skills/cool", raw);
    assert_eq!(
        skill.metadata.get("team").map(String::as_str),
        Some("platform")
    );
}

#[test]
fn parse_dep_skill_folds_legacy_version_author_into_metadata() {
    let raw = "---\nname: Cool\ndescription: x\nversion: 2.0\nauthor: bob\n---\n\nContent.";
    let skill = parse_dep_skill("github.com/owner/pkg/skills/cool", raw);
    assert_eq!(
        skill.metadata.get("version").map(String::as_str),
        Some("2.0")
    );
    assert_eq!(
        skill.metadata.get("author").map(String::as_str),
        Some("bob")
    );
}
