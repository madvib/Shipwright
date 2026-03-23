//! Test fixture helpers for registry e2e tests.
//!
//! Provides utilities for creating local bare git repos and managing test
//! project structures, caches, and lock files.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use compiler::lockfile::{LockPackage, ShipLock};

// ── Git repo helpers ──────────────────────────────────────────────────────────

/// Create a local bare git repo at `<base>/repo-<skill_name>.git` with
/// `skills/<skill_name>/SKILL.md` containing `skill_content`.
///
/// Returns the path to the bare repo. Use as `file://<path>` for git ops.
#[allow(dead_code)]
pub fn make_local_dep_repo(base: &Path, skill_name: &str, skill_content: &str) -> PathBuf {
    let work_name = format!("_work-{}", skill_name);
    let work_dir = base.join(&work_name);
    fs::create_dir_all(&work_dir).unwrap();

    git(&work_dir, &["init", "-b", "main"]).expect("git init");
    git(&work_dir, &["config", "user.email", "test@test.local"]).unwrap();
    git(&work_dir, &["config", "user.name", "Test"]).unwrap();

    let skill_dir = work_dir.join("skills").join(skill_name);
    fs::create_dir_all(&skill_dir).unwrap();
    fs::write(skill_dir.join("SKILL.md"), skill_content).unwrap();

    git(&work_dir, &["add", "-A"]).unwrap();
    git(&work_dir, &["commit", "-m", "add skill"]).unwrap();

    let bare_name = format!("repo-{}.git", skill_name);
    let bare = base.join(&bare_name);
    git_cmd(
        base,
        &["clone", "--bare", work_dir.to_str().unwrap(), &bare_name],
    )
    .expect("git clone --bare");

    bare
}

/// Get the HEAD commit SHA from a git repo (bare or regular).
#[allow(dead_code)]
pub fn git_head_commit(repo: &Path) -> String {
    let out = Command::new("git")
        .args(["-C", repo.to_str().unwrap(), "rev-parse", "HEAD"])
        .output()
        .expect("git rev-parse HEAD");
    assert!(out.status.success(), "git rev-parse HEAD failed");
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

/// Clone a bare repo and return the path to the checked-out working tree
/// with the `.git/` directory removed. Suitable for passing to
/// `PackageCache::store()`.
#[allow(dead_code)]
pub fn extract_repo_content(bare_repo: &Path, base: &Path, tag: &str) -> PathBuf {
    let dest_name = format!("_extracted-{}", tag);
    let dest = base.join(&dest_name);
    if dest.exists() {
        fs::remove_dir_all(&dest).unwrap();
    }
    let url = format!("file://{}", bare_repo.display());
    git_cmd(base, &["clone", &url, &dest_name]).expect("git clone for extraction");
    let git_dir = dest.join(".git");
    if git_dir.exists() {
        fs::remove_dir_all(&git_dir).unwrap();
    }
    dest
}

// ── Lock file helpers ─────────────────────────────────────────────────────────

/// Write a minimal ship.lock with a single package entry.
pub fn write_lock(lock_path: &Path, pkg_path: &str, commit: &str, hash: &str) {
    if let Some(p) = lock_path.parent() {
        fs::create_dir_all(p).unwrap();
    }
    let lock = ShipLock {
        version: 1,
        packages: vec![LockPackage {
            path: pkg_path.to_string(),
            version: commit.to_string(),
            commit: commit.to_string(),
            hash: hash.to_string(),
        }],
    };
    lock.write_atomic(lock_path).expect("write_lock");
}

// ── File helpers ──────────────────────────────────────────────────────────────

/// Write `content` to `<root>/<rel>`, creating parent dirs as needed.
pub fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, content).unwrap();
}

// ── Internal git helpers ──────────────────────────────────────────────────────

fn git(work_dir: &Path, args: &[&str]) -> Result<(), String> {
    let status = Command::new("git")
        .args(args)
        .current_dir(work_dir)
        .status()
        .map_err(|e| format!("failed to run git {}: {e}", args[0]))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("git {} exited with {}", args[0], status))
    }
}

fn git_cmd(cwd: &Path, args: &[&str]) -> Result<(), String> {
    git(cwd, args)
}
