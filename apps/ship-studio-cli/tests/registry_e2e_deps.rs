//! Registry e2e tests: dep skill resolution and compile pipeline (Tests 1-4).
//!
//! Exercises:
//! - Full happy path: dep skill from cache appears in compiled output.
//! - Cache miss: missing lock or missing package entry → actionable error.
//! - Missing skill path: error names the dep, bad path, and available skills.
//! - Local + dep skills: both present in compiled output.
//! - Git fetch path: fetch_package_content via file:// URL.

mod fixtures;

use std::fs;
use std::path::Path;
use tempfile::TempDir;

use compiler::{
    AgentProfile, McpRefs, ProfileMeta, ProfilePermissions, ProfileRules, PluginRefs,
    ProjectLibrary, Rule, Skill, SkillRefs, SkillSource,
    compile, resolve_library, CompileOutput,
};
use compiler::lockfile::{LockPackage, ShipLock};
use runtime::registry::cache::PackageCache;

use fixtures::{extract_repo_content, git_head_commit, make_local_dep_repo, write, write_lock};

// ── Dep-skill resolution helpers ──────────────────────────────────────────────
//
// Inline versions of dep_skills.rs logic that accept an explicit cache_root,
// keeping tests isolated from ~/.ship/cache.

fn parse_dep_ref(s: &str) -> Option<(&str, &str)> {
    if !s.starts_with("github.com/") { return None; }
    let after = &s["github.com/".len()..];
    let s1 = after.find('/')?;
    let after2 = &after[s1 + 1..];
    let s2 = after2.find('/')?;
    let pkg_end = "github.com/".len() + s1 + 1 + s2;
    let within = &s[pkg_end + 1..];
    if within.is_empty() { return None; }
    Some((&s[..pkg_end], within))
}

fn resolve_single(ref_str: &str, lock: &ShipLock, cache: &Path) -> anyhow::Result<Skill> {
    use anyhow::Context;
    let (pkg_path, within) = parse_dep_ref(ref_str)
        .ok_or_else(|| anyhow::anyhow!("invalid dep ref: '{}'", ref_str))?;

    let pkg = lock.packages.iter().find(|p| p.path == pkg_path)
        .ok_or_else(|| anyhow::anyhow!("dependency {} not in cache — run ship install", pkg_path))?;

    let hex = pkg.hash.strip_prefix("sha256:")
        .ok_or_else(|| anyhow::anyhow!("malformed hash '{}' for {}", pkg.hash, pkg_path))?;

    let skill_md = cache.join("objects").join(hex).join(within).join("SKILL.md");
    if !skill_md.exists() {
        let skills_dir = cache.join("objects").join(hex).join("skills");
        let available: Vec<String> = fs::read_dir(&skills_dir)
            .into_iter().flatten().flatten()
            .filter(|e| e.path().is_dir())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        let avail_str = if available.is_empty() { "(none found)".into() } else { available.join(", ") };
        anyhow::bail!(
            "dep skill '{}': path '{}' not found in cached package '{}'; available skills: {}",
            ref_str, within, pkg_path, avail_str
        );
    }

    let raw = fs::read_to_string(&skill_md)
        .with_context(|| format!("reading {}", skill_md.display()))?;

    Ok(Skill {
        id: ref_str.to_string(), name: ref_str.to_string(),
        description: None, license: None, compatibility: None,
        allowed_tools: vec![], metadata: Default::default(),
        content: raw.trim().to_string(), source: SkillSource::Community,
    })
}

fn resolve_deps(refs: &[String], local: &[Skill], lock: &ShipLock, cache: &Path) -> anyhow::Result<Vec<Skill>> {
    use anyhow::Context;
    let ids: std::collections::HashSet<&str> = local.iter().map(|s| s.id.as_str()).collect();
    let mut out = Vec::new();
    for r in refs {
        if !r.starts_with("github.com/") || ids.contains(r.as_str()) { continue; }
        out.push(resolve_single(r, lock, cache).with_context(|| format!("resolving '{}'", r))?);
    }
    Ok(out)
}

// ── Compile helper ────────────────────────────────────────────────────────────

fn to_claude(lib: &ProjectLibrary, mode: Option<&str>) -> Option<CompileOutput> {
    let resolved = resolve_library(lib, None, mode);
    compile(&resolved, "claude")
}

/// Flatten all skill_files content into one string for easy assertion.
fn skill_files_content(out: &CompileOutput) -> String {
    let mut parts: Vec<&str> = out.skill_files.values().map(|s| s.as_str()).collect();
    parts.sort(); // deterministic
    parts.join("\n\n")
}

fn profile(id: &str, providers: &[&str], refs: &[&str]) -> AgentProfile {
    AgentProfile {
        profile: ProfileMeta {
            id: id.into(), name: id.into(),
            version: None, description: None,
            providers: providers.iter().map(|s| s.to_string()).collect(),
        },
        skills: SkillRefs { refs: refs.iter().map(|s| s.to_string()).collect() },
        mcp: McpRefs { servers: vec![] },
        plugins: PluginRefs { install: vec![], scope: None },
        permissions: ProfilePermissions::default(),
        rules: ProfileRules::default(),
        provider_settings: std::collections::HashMap::new(),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test 1: Full happy path
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn full_happy_path_dep_skill_appears_in_compiled_output() {
    let tmp = TempDir::new().unwrap();
    let cache_tmp = TempDir::new().unwrap();

    let bare = make_local_dep_repo(tmp.path(), "my-skill", "# Test Skill\nBe excellent.");
    let commit = git_head_commit(&bare);
    let content_dir = extract_repo_content(&bare, tmp.path(), "main");

    let cache = PackageCache::with_root(cache_tmp.path().to_path_buf());
    let pkg = cache.store("github.com/owner/mypkg", &commit, &commit, &content_dir)
        .expect("cache store must succeed");

    let lock_path = tmp.path().join("ship.lock");
    write_lock(&lock_path, "github.com/owner/mypkg", &commit, &pkg.hash);
    let lock = ShipLock::from_file(&lock_path).unwrap();

    let dep_ref = "github.com/owner/mypkg/skills/my-skill".to_string();
    let mut lib = ProjectLibrary::default();
    lib.rules.push(Rule {
        file_name: "base.md".into(), content: "Follow the rules.".into(),
        always_apply: true, globs: vec![], description: None,
    });
    lib.agent_profiles.push(profile("main", &["claude"], &[&dep_ref]));

    let dep_skills = resolve_deps(&[dep_ref], &[], &lock, cache_tmp.path())
        .expect("dep skill resolution must succeed");
    lib.skills.extend(dep_skills);

    let out = to_claude(&lib, Some("main")).expect("compile must produce output");
    let skills = skill_files_content(&out);
    assert!(skills.contains("Be excellent."), "skill_files must contain dep skill; got:\n{skills}");
    let ctx = out.context_content.as_deref().unwrap_or("");
    assert!(ctx.contains("Follow the rules."), "context_content must contain local rules; got:\n{ctx}");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test 2: Cache miss — lock absent
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn cache_miss_when_lock_absent_produces_actionable_error() {
    let tmp = TempDir::new().unwrap();
    let lock_path = tmp.path().join("ship.lock");

    let result = ShipLock::from_file(&lock_path);
    assert!(result.is_err(), "reading a non-existent lock must fail");
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("ship.lock") || msg.contains("Cannot read"),
        "error must reference the missing lock file; got:\n{msg}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test 2b: Cache miss — package not in lock
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn cache_miss_when_package_absent_from_lock_has_actionable_error() {
    let cache_tmp = TempDir::new().unwrap();
    let lock = ShipLock::default();

    let dep_ref = "github.com/owner/missingpkg/skills/cool-skill".to_string();
    let err = resolve_deps(&[dep_ref], &[], &lock, cache_tmp.path()).unwrap_err();
    let chain = format!("{:#}", err);
    assert!(chain.contains("not in cache"), "error must say 'not in cache'; got:\n{chain}");
    assert!(chain.contains("ship install"), "error must say 'ship install'; got:\n{chain}");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test 3: Missing skill path within cached package
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn missing_skill_path_in_cached_package_lists_available() {
    let tmp = TempDir::new().unwrap();
    let cache_tmp = TempDir::new().unwrap();

    let content_dir = tmp.path().join("pkg");
    write(&content_dir, "skills/real-skill/SKILL.md", "# Real\nDoes real things.");

    let cache = PackageCache::with_root(cache_tmp.path().to_path_buf());
    let commit = "a".repeat(40);
    let pkg = cache.store("github.com/owner/pkg", &commit, &commit, &content_dir)
        .expect("cache store must succeed");

    let lock = ShipLock {
        version: 1,
        packages: vec![LockPackage {
            path: "github.com/owner/pkg".into(),
            version: commit.clone(), commit: commit.clone(), hash: pkg.hash,
        }],
    };

    let dep_ref = "github.com/owner/pkg/skills/nonexistent".to_string();
    let err = resolve_deps(&[dep_ref], &[], &lock, cache_tmp.path()).unwrap_err();
    let chain = format!("{:#}", err);

    assert!(chain.contains("nonexistent"), "error must name bad path; got:\n{chain}");
    assert!(chain.contains("real-skill"), "error must list available skills; got:\n{chain}");
    assert!(chain.contains("github.com/owner/pkg"), "error must name the package; got:\n{chain}");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test 4: Local + dep skills both appear in output
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn local_and_dep_skills_both_appear_in_compiled_output() {
    let tmp = TempDir::new().unwrap();
    let cache_tmp = TempDir::new().unwrap();

    let content_dir = tmp.path().join("pkg");
    write(&content_dir, "skills/remote-skill/SKILL.md", "# Remote\nDo remote things.");

    let cache = PackageCache::with_root(cache_tmp.path().to_path_buf());
    let commit = "b".repeat(40);
    let pkg = cache.store("github.com/owner/remotepkg", &commit, &commit, &content_dir)
        .expect("cache store must succeed");

    let lock = ShipLock {
        version: 1,
        packages: vec![LockPackage {
            path: "github.com/owner/remotepkg".into(),
            version: commit.clone(), commit: commit.clone(), hash: pkg.hash,
        }],
    };

    let local = Skill {
        id: "local-skill".into(), name: "Local".into(),
        description: None, license: None, compatibility: None,
        allowed_tools: vec![], metadata: Default::default(),
        content: "Do local things.".into(), source: SkillSource::Custom,
    };
    let dep_ref = "github.com/owner/remotepkg/skills/remote-skill".to_string();
    let dep_skills = resolve_deps(&[dep_ref.clone()], &[local.clone()], &lock, cache_tmp.path())
        .expect("dep skill resolution must succeed");

    let mut lib = ProjectLibrary::default();
    lib.skills.push(local);
    lib.skills.extend(dep_skills);
    lib.agent_profiles.push(profile("combined", &["claude"], &["local-skill", &dep_ref]));

    let out = to_claude(&lib, Some("combined")).expect("compile must produce output");
    let skills = skill_files_content(&out);
    assert!(skills.contains("Do local things."), "skill_files must contain local skill; got:\n{skills}");
    assert!(skills.contains("Do remote things."), "skill_files must contain dep skill; got:\n{skills}");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Bonus: git fetch from local bare repo
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn fetch_package_content_works_with_local_bare_repo() {
    use runtime::registry::fetch::fetch_package_content;

    let tmp = TempDir::new().unwrap();
    let bare = make_local_dep_repo(
        tmp.path(), "fetch-skill", "# Fetched\nFetched from local bare repo.",
    );

    let commit = git_head_commit(&bare);
    let dest = tmp.path().join("dest");
    fs::create_dir_all(&dest).unwrap();

    let url = format!("file://{}", bare.display());
    fetch_package_content(&url, &commit, &dest)
        .expect("fetch from local bare repo must succeed");

    let skill_md = dest.join("skills/fetch-skill/SKILL.md");
    assert!(skill_md.exists(), "SKILL.md must be present after fetch");
    let content = fs::read_to_string(&skill_md).unwrap();
    assert!(
        content.contains("Fetched from local bare repo."),
        "SKILL.md content must match; got:\n{content}"
    );
}
