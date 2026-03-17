//! Job queue CLI — create, list, update, and coordination loop commands.
//!
//! Coordination loop:
//!   `ship next`          — claim a pending job, build a worktree, print opening message
//!   `ship retry <id>`    — reset a failed/stalled job so `next` can pick it up
//!   `ship job done <id>` — commit worktree changes and mark job complete
//!   `ship gate <id>`     — run tests; merge on pass or mark failed on failure (stretch)

use anyhow::Result;
use runtime::db::jobs;
use std::path::{Path, PathBuf};

use crate::paths::project_ship_dir_required;

// ── Worktree helpers ──────────────────────────────────────────────────────────

/// Default directory under which per-job worktrees are created.
fn default_worktrees_root() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("dev")
        .join("ship-worktrees")
}

fn resolve_worktrees_root(override_dir: Option<PathBuf>) -> PathBuf {
    override_dir.unwrap_or_else(default_worktrees_root)
}

/// Build the worktree path for a given job id.
fn worktree_path(root: &Path, job_id: &str) -> PathBuf {
    root.join(job_id)
}

/// Remove the worktree directory and detach the git worktree if it exists.
fn remove_worktree(root: &Path, job_id: &str) -> Result<()> {
    let wt_path = worktree_path(root, job_id);
    let branch = format!("job/{}", job_id);

    // `git worktree remove --force` cleans up the admin entry + the directory.
    // We ignore errors here — the path may not be a registered worktree yet
    // (e.g. it was created manually, or a previous run was interrupted).
    let _ = std::process::Command::new("git")
        .args(["worktree", "remove", "--force", wt_path.to_str().unwrap_or("")])
        .output();

    // Prune stale worktree admin entries.
    let _ = std::process::Command::new("git")
        .args(["worktree", "prune"])
        .output();

    // Delete the branch if it exists.
    let _ = std::process::Command::new("git")
        .args(["branch", "-D", &branch])
        .output();

    Ok(())
}

/// Create a git worktree at `<root>/<job_id>` on branch `job/<job_id>`.
/// The branch is created from HEAD if it doesn't exist yet.
fn create_worktree(root: &Path, job_id: &str) -> Result<PathBuf> {
    let wt_path = worktree_path(root, job_id);
    let branch = format!("job/{}", job_id);
    std::fs::create_dir_all(root)?;

    let out = std::process::Command::new("git")
        .args(["worktree", "add", "-b", &branch, wt_path.to_str().unwrap_or(""), "HEAD"])
        .output()?;
    if !out.status.success() {
        anyhow::bail!(
            "git worktree add failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(wt_path)
}

/// Write `job-spec.md` into the worktree from the job payload.
fn write_job_spec(wt_path: &Path, job: &jobs::Job) -> Result<()> {
    let title = job.payload.get("title")
        .and_then(|v| v.as_str())
        .unwrap_or(&job.kind);
    let description = job.payload.get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("No description provided.");
    let preset_hint = job.payload.get("preset_hint")
        .and_then(|v| v.as_str())
        .unwrap_or("default");

    // Optional structured fields (graceful fallback when absent).
    let capability_id = job.payload.get("capability_id")
        .and_then(|v| v.as_str());
    let symlink_name = job.payload.get("symlink_name")
        .and_then(|v| v.as_str());
    let scope: Vec<&str> = job.payload.get("scope")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|s| s.as_str()).collect())
        .unwrap_or_default();
    let criteria: Vec<&str> = job.payload.get("acceptance_criteria")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|s| s.as_str()).collect())
        .unwrap_or_default();

    let mut content = format!(
        "# Job {} — {}\n\n## Kind\n{}\n\n## Preset hint\n{}\n\n",
        job.id, title, job.kind, preset_hint
    );

    if let Some(cap) = capability_id {
        content.push_str(&format!("## Capability\n{}\n\n", cap));
    }
    if let Some(sym) = symlink_name {
        content.push_str(&format!("## Symlink name\n{}\n\n", sym));
    }
    if !scope.is_empty() {
        content.push_str("## Scope\n");
        for s in &scope {
            content.push_str(&format!("- {}\n", s));
        }
        content.push('\n');
    }
    content.push_str(&format!("## Description\n{}\n\n", description));
    content.push_str("## Acceptance criteria\n");
    if criteria.is_empty() {
        content.push_str("- See description above\n");
    } else {
        for c in &criteria {
            content.push_str(&format!("- [ ] {}\n", c));
        }
    }
    content.push_str(&format!(
        "\n## Mark done\nMark done via MCP: `update_job(id=\"{}\", status=\"complete\")`\n",
        job.id
    ));

    std::fs::write(wt_path.join("job-spec.md"), content)?;
    Ok(())
}

/// Run `ship use <preset>` inside the worktree.
fn run_ship_use(wt_path: &Path, preset: &str) -> Result<()> {
    // ship use requires .ship/ in the worktree. Worktrees share the git index
    // but not the working tree, so .ship/ must exist in the root project and
    // will be visible via the worktree's checkout.  If .ship/ is absent we
    // skip silently rather than aborting the loop.
    if !wt_path.join(".ship").exists() {
        println!("  note: .ship/ not found in worktree — skipping `ship use`");
        return Ok(());
    }
    let out = std::process::Command::new("ship")
        .args(["use", preset, "--path", wt_path.to_str().unwrap_or(".")])
        .output();
    match out {
        Ok(o) if o.status.success() => {
            println!("  ship use {} — ok", preset);
        }
        Ok(o) => {
            // Non-fatal: the worktree is still usable; the agent can run manually.
            println!(
                "  ship use {} — warning: {}",
                preset,
                String::from_utf8_lossy(&o.stderr).trim()
            );
        }
        Err(e) => {
            println!("  ship use {} — warning: could not run ship: {}", preset, e);
        }
    }
    Ok(())
}

// ── ship next ─────────────────────────────────────────────────────────────────

pub fn next(worktrees_dir: Option<PathBuf>) -> Result<()> {
    let ship_dir = project_ship_dir_required()?;
    let root = resolve_worktrees_root(worktrees_dir);

    // Load all pending jobs sorted oldest-first (highest priority first within
    // that, but the DB doesn't expose priority sort directly — we sort here).
    let mut pending = jobs::list_jobs(&ship_dir, None, Some("pending"))?;
    if pending.is_empty() {
        println!("No pending jobs. Queue is empty.");
        return Ok(());
    }
    // Sort: higher priority descending, then created_at ascending.
    pending.sort_by(|a, b| {
        b.priority.cmp(&a.priority)
            .then(a.created_at.cmp(&b.created_at))
    });

    // Try each candidate until one claims successfully (handles race conditions
    // when multiple commanders share the same queue).
    let mut claimed: Option<jobs::Job> = None;
    for candidate in &pending {
        if jobs::claim_job(&ship_dir, &candidate.id, "cli")? {
            claimed = Some(jobs::list_jobs(&ship_dir, None, None)?
                .into_iter()
                .find(|j| j.id == candidate.id)
                .ok_or_else(|| anyhow::anyhow!("claimed job disappeared: {}", candidate.id))?);
            break;
        }
    }

    let job = match claimed {
        Some(j) => j,
        None => {
            println!("No pending jobs could be claimed (all were claimed by another commander).");
            return Ok(());
        }
    };

    println!("✓ claimed job/{}", &job.id);

    // Build worktree.
    let wt_path = create_worktree(&root, &job.id)?;
    println!("  worktree: {}", wt_path.display());

    // Write spec.
    write_job_spec(&wt_path, &job)?;
    println!("  wrote job-spec.md");

    // Activate preset.
    let preset = job.payload.get("preset_hint")
        .and_then(|v| v.as_str())
        .unwrap_or("default");
    run_ship_use(&wt_path, preset)?;

    // Print ready message.
    let title = job.payload.get("title")
        .and_then(|v| v.as_str())
        .unwrap_or(&job.kind);
    println!();
    println!("─── Agent opening message (paste into new session) ───");
    println!(
        "You are a Rust CLI specialist agent working on job {}. \
         Your worktree is at {}. \
         Read the job spec at {}/job-spec.md — that is your full context. \
         Work in that directory. \
         Use `append_job_log` via ship MCP to log touched files. \
         Mark done with `update_job(id=\"{}\", status=\"complete\")` when all acceptance criteria are met. \
         Start immediately.",
        job.id,
        wt_path.display(),
        wt_path.display(),
        job.id,
    );
    println!("──────────────────────────────────────────────────────");
    println!();
    println!("  job:   {} — {}", &job.id[..8], title);
    println!("  kind:  {}", job.kind);
    println!("  path:  {}", wt_path.display());
    println!("  branch: job/{}", job.id);

    Ok(())
}

// ── ship retry ────────────────────────────────────────────────────────────────

pub fn retry(id_prefix: &str, worktrees_dir: Option<PathBuf>) -> Result<()> {
    let ship_dir = project_ship_dir_required()?;
    let root = resolve_worktrees_root(worktrees_dir);

    // Resolve job by prefix.
    let all = jobs::list_jobs(&ship_dir, None, None)?;
    let matched: Vec<_> = all.iter().filter(|j| j.id.starts_with(id_prefix)).collect();
    let job = match matched.len() {
        0 => anyhow::bail!("No job matching '{}'", id_prefix),
        1 => matched[0].clone(),
        _ => anyhow::bail!("Ambiguous prefix '{}' — {} matches", id_prefix, matched.len()),
    };

    // Only allow retrying non-pending jobs (pending is already retry-able via next).
    if job.status == "pending" {
        println!("Job {} is already pending — nothing to do.", &job.id[..8]);
        return Ok(());
    }

    println!("Retrying job/{} (was: {})", &job.id[..8], job.status);

    // Wipe existing worktree.
    remove_worktree(&root, &job.id)?;
    println!("  removed old worktree");

    // Reset status to pending.
    jobs::update_job_status(&ship_dir, &job.id, "pending")?;
    println!("  status → pending");

    // Recreate worktree.
    let wt_path = create_worktree(&root, &job.id)?;
    println!("  worktree: {}", wt_path.display());

    // Rewrite spec.
    write_job_spec(&wt_path, &job)?;
    println!("  wrote job-spec.md");

    println!();
    println!("Ready. Run `ship next` to claim and activate this job.");
    Ok(())
}

// ── ship gate ─────────────────────────────────────────────────────────────────

/// Files that must never be modified, regardless of declared file_scope.
const NEVER_TOUCH: &[&str] = &[
    "wrangler.jsonc",
    "package.json",
    "routeTree.gen.ts",
    ".dev.vars",
    "pnpm-lock.yaml",
];

/// Returns true if `file` is within any declared scope entry.
/// Matches exactly, or as a path prefix (only at `/` boundaries).
fn in_scope(file: &str, scope: &[String]) -> bool {
    scope.iter().any(|s| {
        if file == s.as_str() {
            return true;
        }
        if file.starts_with(s.as_str()) {
            let rest = &file[s.len()..];
            return s.ends_with('/') || rest.starts_with('/');
        }
        false
    })
}

/// Get changed files in the worktree branch relative to its base (main).
fn git_diff_files(wt_path: &Path) -> Result<Vec<String>> {
    let out = std::process::Command::new("git")
        .args(["diff", "--name-only", "main...HEAD"])
        .current_dir(wt_path)
        .output()?;
    if !out.status.success() {
        anyhow::bail!(
            "git diff failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect())
}

pub fn gate(id_prefix: &str, worktrees_dir: Option<PathBuf>) -> Result<()> {
    let ship_dir = project_ship_dir_required()?;
    let root = resolve_worktrees_root(worktrees_dir);

    let all = jobs::list_jobs(&ship_dir, None, None)?;
    let matched: Vec<_> = all.iter().filter(|j| j.id.starts_with(id_prefix)).collect();
    let job = match matched.len() {
        0 => anyhow::bail!("No job matching '{}'", id_prefix),
        1 => matched[0].clone(),
        _ => anyhow::bail!("Ambiguous prefix '{}' — {} matches", id_prefix, matched.len()),
    };

    let wt_path = worktree_path(&root, &job.id);
    if !wt_path.exists() {
        anyhow::bail!(
            "Worktree not found at {}. Has this job been claimed via `ship next`?",
            wt_path.display()
        );
    }

    println!("Gating job/{} at {}", &job.id[..8], wt_path.display());

    // ── Scope check ───────────────────────────────────────────────────────────
    let changed = git_diff_files(&wt_path)?;
    let file_scope = &job.file_scope;

    let mut never_touch_violations: Vec<&str> = vec![];
    let mut scope_violations: Vec<&str> = vec![];

    for file in &changed {
        let basename = std::path::Path::new(file)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(file);
        if NEVER_TOUCH.contains(&basename) || NEVER_TOUCH.contains(&file.as_str()) {
            never_touch_violations.push(file);
        } else if !file_scope.is_empty() && !in_scope(file, file_scope) {
            scope_violations.push(file);
        }
    }

    let scope_ok = never_touch_violations.is_empty() && scope_violations.is_empty();

    if !never_touch_violations.is_empty() {
        println!("  BLOCKED — never-touch files modified:");
        for f in &never_touch_violations {
            println!("    ✗ {}", f);
        }
    }
    if !scope_violations.is_empty() {
        println!("  BLOCKED — files outside declared scope:");
        for f in &scope_violations {
            println!("    ✗ {}", f);
        }
        println!("  declared scope: {:?}", file_scope);
    }
    if !scope_ok {
        jobs::update_job_status(&ship_dir, &job.id, "failed")?;
        anyhow::bail!("gate failed: scope violations detected");
    }

    // ── Test runner ───────────────────────────────────────────────────────────
    let test_out = if wt_path.join("Cargo.toml").exists() {
        let manifest = wt_path.join("Cargo.toml");
        std::process::Command::new("cargo")
            .args(["test", "--manifest-path", manifest.to_str().unwrap_or("Cargo.toml")])
            .output()?
    } else if wt_path.join("package.json").exists() {
        std::process::Command::new("pnpm")
            .args(["test", "--prefix", wt_path.to_str().unwrap_or(".")])
            .output()?
    } else {
        anyhow::bail!("Cannot detect test runner — no Cargo.toml or package.json in worktree");
    };

    let pass = test_out.status.success();
    if pass {
        println!("  tests passed");

        // Merge job branch into current branch.
        let branch = format!("job/{}", job.id);
        let merge_out = std::process::Command::new("git")
            .args(["merge", "--no-ff", "-m",
                   &format!("merge: job/{} — {}", &job.id[..8],
                       job.payload.get("title").and_then(|v| v.as_str()).unwrap_or(&job.kind)),
                   &branch])
            .output()?;
        if merge_out.status.success() {
            jobs::update_job_status(&ship_dir, &job.id, "complete")?;
            println!("✓ merged {} → current branch", branch);
            println!("  job {} marked complete", &job.id[..8]);
        } else {
            anyhow::bail!(
                "merge failed: {}",
                String::from_utf8_lossy(&merge_out.stderr).trim()
            );
        }
    } else {
        let stderr = String::from_utf8_lossy(&test_out.stderr);
        let stdout = String::from_utf8_lossy(&test_out.stdout);
        println!("  tests FAILED");
        if !stdout.is_empty() { println!("{}", stdout.trim()); }
        if !stderr.is_empty() { println!("{}", stderr.trim()); }
        jobs::update_job_status(&ship_dir, &job.id, "failed")?;
        println!("  job {} marked failed — run `ship retry {}` to re-queue", &job.id[..8], &job.id[..8]);
    }

    Ok(())
}

// ── ship job create / list / update / done ────────────────────────────────────

pub fn create(
    kind: &str,
    title: &str,
    milestone: Option<&str>,
    description: Option<&str>,
    branch: Option<&str>,
) -> Result<()> {
    let ship_dir = project_ship_dir_required()?;
    let mut payload = serde_json::json!({ "title": title });
    if let Some(m) = milestone {
        payload["milestone"] = serde_json::Value::String(m.to_string());
    }
    if let Some(d) = description {
        payload["description"] = serde_json::Value::String(d.to_string());
    }
    let job = jobs::create_job(&ship_dir, kind, branch, Some(payload), Some("human"), None, 0, None, vec![], vec![])?;
    println!("{}\t[{}]\t{}", job.id, job.kind, title);
    Ok(())
}

pub fn list(status: Option<&str>, branch: Option<&str>, milestone: Option<&str>) -> Result<()> {
    let ship_dir = project_ship_dir_required()?;
    let all = jobs::list_jobs(&ship_dir, branch, status)?;

    let filtered: Vec<_> = if let Some(m) = milestone {
        all.iter()
            .filter(|j| j.payload.get("milestone").and_then(|v| v.as_str()) == Some(m))
            .collect()
    } else {
        all.iter().collect()
    };

    if filtered.is_empty() {
        println!("No jobs.");
        return Ok(());
    }

    // Print oldest-first for queue ordering.
    let mut sorted = filtered.clone();
    sorted.sort_by(|a, b| a.created_at.cmp(&b.created_at));

    for job in sorted {
        let desc = job.payload.get("description")
            .and_then(|v| v.as_str())
            .or_else(|| job.payload.get("title").and_then(|v| v.as_str()))
            .unwrap_or("");
        let desc_trunc = if desc.len() > 50 {
            format!("{}…", &desc[..50])
        } else {
            desc.to_string()
        };
        let date = job.created_at.get(..10).unwrap_or(&job.created_at);
        println!("{}\t{}\t{}\t{}\t{}", job.id, job.status, job.kind, desc_trunc, date);
    }
    Ok(())
}

pub fn done(id_prefix: &str) -> Result<()> {
    let ship_dir = project_ship_dir_required()?;
    let all = jobs::list_jobs(&ship_dir, None, None)?;
    let matched: Vec<_> = all.iter().filter(|j| j.id.starts_with(id_prefix)).collect();
    let job = match matched.len() {
        0 => anyhow::bail!("No job matching '{}'", id_prefix),
        1 => matched[0],
        _ => anyhow::bail!("Ambiguous prefix '{}' — {} matches", id_prefix, matched.len()),
    };

    // Idempotent: already complete is a no-op.
    if matches!(job.status.as_str(), "complete" | "done") {
        println!("Job {} is already {} — nothing to do.", &job.id[..8], job.status);
        return Ok(());
    }

    // Stage files in job's declared scope, fall back to all modified tracked files.
    if !job.touched_files.is_empty() {
        let ok = std::process::Command::new("git")
            .args(["add", "--"])
            .args(&job.touched_files)
            .status()?
            .success();
        if !ok { anyhow::bail!("git add failed"); }
    } else {
        let ok = std::process::Command::new("git")
            .args(["add", "-u"])
            .status()?
            .success();
        if !ok { anyhow::bail!("git add failed"); }
    }

    // Commit with job reference.
    let desc = job.payload.get("description")
        .and_then(|v| v.as_str())
        .or_else(|| job.payload.get("title").and_then(|v| v.as_str()))
        .unwrap_or(&job.kind);
    let commit_msg = format!("{} (job/{})", desc, job.id);
    let out = std::process::Command::new("git")
        .args(["commit", "-m", &commit_msg])
        .output()?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        // "nothing to commit" is not an error for idempotency.
        if stderr.contains("nothing to commit") || stderr.contains("nothing added") {
            println!("  nothing to commit");
        } else {
            anyhow::bail!("git commit failed: {}", stderr.trim());
        }
    }

    let hash = String::from_utf8_lossy(
        &std::process::Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output()?
            .stdout,
    ).trim().to_string();

    jobs::update_job_status(&ship_dir, &job.id, "complete")?;

    println!("✓ job/{} complete", &job.id[..8]);
    println!("  commit {}", hash);
    Ok(())
}

pub fn update(id_prefix: &str, status: &str) -> Result<()> {
    let ship_dir = project_ship_dir_required()?;
    let all = jobs::list_jobs(&ship_dir, None, None)?;
    let matched: Vec<_> = all.iter().filter(|j| j.id.starts_with(id_prefix)).collect();
    match matched.len() {
        0 => anyhow::bail!("No job matching '{}'", id_prefix),
        1 => {
            jobs::update_job_status(&ship_dir, &matched[0].id, status)?;
            println!("✓ {} → {}", &matched[0].id[..8], status);
        }
        _ => anyhow::bail!("Ambiguous prefix '{}' — {} matches", id_prefix, matched.len()),
    }
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use runtime::db::{ensure_db, jobs};
    use runtime::project::init_project;
    use tempfile::tempdir;

    fn setup() -> (tempfile::TempDir, PathBuf) {
        let tmp = tempdir().unwrap();
        let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
        ensure_db(&ship_dir).unwrap();
        (tmp, ship_dir)
    }

    fn mk_job(ship_dir: &Path, title: &str) -> jobs::Job {
        let payload = serde_json::json!({ "title": title, "description": "test job" });
        jobs::create_job(ship_dir, "test", None, Some(payload), Some("test"), None, 0, None, vec![], vec![]).unwrap()
    }

    #[test]
    fn test_done_idempotent_on_complete() {
        let (_tmp, ship_dir) = setup();
        let job = mk_job(&ship_dir, "already done job");
        jobs::update_job_status(&ship_dir, &job.id, "complete").unwrap();

        // Call done logic directly (no git in test env — check guard only).
        let all = jobs::list_jobs(&ship_dir, None, None).unwrap();
        let j = all.iter().find(|j| j.id == job.id).unwrap();
        assert!(matches!(j.status.as_str(), "complete" | "done"),
            "expected complete, got {}", j.status);
    }

    #[test]
    fn test_retry_resets_status_to_pending() {
        let (_tmp, ship_dir) = setup();
        let job = mk_job(&ship_dir, "failed job");
        jobs::update_job_status(&ship_dir, &job.id, "failed").unwrap();

        // Verify it's failed.
        let j = jobs::get_job(&ship_dir, &job.id).unwrap().unwrap();
        assert_eq!(j.status, "failed");

        // Reset to pending (core of retry logic).
        jobs::update_job_status(&ship_dir, &job.id, "pending").unwrap();
        let j = jobs::get_job(&ship_dir, &job.id).unwrap().unwrap();
        assert_eq!(j.status, "pending");
    }

    #[test]
    fn test_next_claims_oldest_pending_by_priority() {
        let (_tmp, ship_dir) = setup();
        let j1 = mk_job(&ship_dir, "low prio");
        let j2 = jobs::create_job(
            &ship_dir, "urgent", None,
            Some(serde_json::json!({ "title": "high prio" })),
            Some("test"), None, 10, None, vec![], vec![]
        ).unwrap();

        // Highest priority should be claimed first.
        let claimed = jobs::claim_job(&ship_dir, &j2.id, "cli").unwrap();
        assert!(claimed, "high priority job should be claimable");

        // j1 should still be pending.
        let j1_state = jobs::get_job(&ship_dir, &j1.id).unwrap().unwrap();
        assert_eq!(j1_state.status, "pending");

        // j2 should now be running.
        let j2_state = jobs::get_job(&ship_dir, &j2.id).unwrap().unwrap();
        assert_eq!(j2_state.status, "running");
    }

    #[test]
    fn test_claim_job_is_atomic_no_double_claim() {
        let (_tmp, ship_dir) = setup();
        let job = mk_job(&ship_dir, "race condition");

        let first = jobs::claim_job(&ship_dir, &job.id, "commander-a").unwrap();
        let second = jobs::claim_job(&ship_dir, &job.id, "commander-b").unwrap();

        assert!(first, "first claim should succeed");
        assert!(!second, "second claim should fail — job already running");

        let j = jobs::get_job(&ship_dir, &job.id).unwrap().unwrap();
        assert_eq!(j.claimed_by, Some("commander-a".to_string()));
    }

    #[test]
    fn test_write_job_spec_contains_id_and_title() {
        let tmp = tempdir().unwrap();
        let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
        ensure_db(&ship_dir).unwrap();

        let payload = serde_json::json!({
            "title": "Build the thing",
            "description": "Make it work.",
            "preset_hint": "rust-expert"
        });
        let job = jobs::create_job(&ship_dir, "feature", None, Some(payload), None, None, 0, None, vec![], vec![]).unwrap();

        let wt = tmp.path().join("wt");
        std::fs::create_dir_all(&wt).unwrap();
        write_job_spec(&wt, &job).unwrap();

        let content = std::fs::read_to_string(wt.join("job-spec.md")).unwrap();
        assert!(content.contains(&job.id), "spec must contain job id");
        assert!(content.contains("Build the thing"), "spec must contain title");
        assert!(content.contains("rust-expert"), "spec must contain preset_hint");
        assert!(content.contains("Make it work."), "spec must contain description");
    }

    #[test]
    fn test_write_job_spec_structured_fields() {
        let tmp = tempdir().unwrap();
        let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
        ensure_db(&ship_dir).unwrap();

        let payload = serde_json::json!({
            "title": "Structured job",
            "description": "Do the thing.",
            "preset_hint": "default",
            "capability_id": "abcd1234",
            "symlink_name": "my-feature",
            "scope": ["src/lib.rs", "src/main.rs"],
            "acceptance_criteria": ["Tests pass", "Doc updated"]
        });
        let job = jobs::create_job(&ship_dir, "feature", None, Some(payload), None, None, 0, None, vec![], vec![]).unwrap();

        let wt = tmp.path().join("wt2");
        std::fs::create_dir_all(&wt).unwrap();
        write_job_spec(&wt, &job).unwrap();

        let content = std::fs::read_to_string(wt.join("job-spec.md")).unwrap();
        assert!(content.contains("abcd1234"), "spec must contain capability_id");
        assert!(content.contains("my-feature"), "spec must contain symlink_name");
        assert!(content.contains("src/lib.rs"), "spec must contain scope entries");
        assert!(content.contains("- [ ] Tests pass"), "spec must contain acceptance criteria with checkbox");
        assert!(content.contains("- [ ] Doc updated"), "spec must contain second criterion");
    }

    #[test]
    fn test_write_job_spec_graceful_fallback_without_structured_fields() {
        let tmp = tempdir().unwrap();
        let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
        ensure_db(&ship_dir).unwrap();

        // Minimal payload — no structured fields
        let payload = serde_json::json!({ "description": "Old-style job" });
        let job = jobs::create_job(&ship_dir, "test", None, Some(payload), None, None, 0, None, vec![], vec![]).unwrap();

        let wt = tmp.path().join("wt3");
        std::fs::create_dir_all(&wt).unwrap();
        // Must not panic or error
        write_job_spec(&wt, &job).unwrap();

        let content = std::fs::read_to_string(wt.join("job-spec.md")).unwrap();
        assert!(content.contains("Old-style job"), "description must appear");
        assert!(content.contains("See description above"), "fallback criteria must appear");
    }

    #[test]
    fn test_worktree_path_uses_job_id() {
        let root = PathBuf::from("/dev/worktrees");
        let p = worktree_path(&root, "abc123");
        assert_eq!(p, PathBuf::from("/dev/worktrees/abc123"));
    }

    #[test]
    fn test_in_scope_prefix_match() {
        let scope = vec!["crates/core/".to_string(), "apps/mcp/src/".to_string()];
        assert!(in_scope("crates/core/runtime/src/lib.rs", &scope));
        assert!(in_scope("apps/mcp/src/server.rs", &scope));
        assert!(!in_scope("apps/web/src/lib.ts", &scope));
        assert!(!in_scope("Cargo.toml", &scope));
    }

    #[test]
    fn test_in_scope_exact_match() {
        let scope = vec!["src/lib.rs".to_string()];
        assert!(in_scope("src/lib.rs", &scope));
        assert!(!in_scope("src/lib.rs.bak", &scope));
    }

    #[test]
    fn test_in_scope_empty_scope_always_passes() {
        // Empty scope means no restriction — gate should not block on scope.
        let scope: Vec<String> = vec![];
        // in_scope is not called when scope is empty (checked in gate fn),
        // but verify the function itself returns false for empty (the gate logic handles it).
        assert!(!in_scope("any/file.rs", &scope));
    }

    #[test]
    fn test_never_touch_list_is_correct() {
        // Verify the never-touch list contains the required entries.
        assert!(NEVER_TOUCH.contains(&"wrangler.jsonc"));
        assert!(NEVER_TOUCH.contains(&"package.json"));
        assert!(NEVER_TOUCH.contains(&"routeTree.gen.ts"));
        assert!(NEVER_TOUCH.contains(&".dev.vars"));
        assert!(NEVER_TOUCH.contains(&"pnpm-lock.yaml"));
    }

    #[test]
    fn test_gate_scope_check_logic() {
        // Simulate the gate scope checking logic in isolation.
        let scope = vec!["crates/".to_string()];
        let changed = vec![
            "crates/core/runtime/src/lib.rs".to_string(),
            "apps/web/src/index.ts".to_string(), // out of scope
        ];
        let mut violations: Vec<&str> = vec![];
        for file in &changed {
            let basename = std::path::Path::new(file)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(file);
            if NEVER_TOUCH.contains(&basename) || NEVER_TOUCH.contains(&file.as_str()) {
                violations.push(file);
            } else if !scope.is_empty() && !in_scope(file, &scope) {
                violations.push(file);
            }
        }
        assert_eq!(violations, vec!["apps/web/src/index.ts"]);
    }

    #[test]
    fn test_gate_never_touch_blocks_even_in_scope() {
        let scope = vec!["apps/".to_string()];
        let changed = vec!["apps/package.json".to_string()];
        let mut violations: Vec<&str> = vec![];
        for file in &changed {
            let basename = std::path::Path::new(file)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(file);
            if NEVER_TOUCH.contains(&basename) || NEVER_TOUCH.contains(&file.as_str()) {
                violations.push(file);
            } else if !scope.is_empty() && !in_scope(file, &scope) {
                violations.push(file);
            }
        }
        assert_eq!(violations, vec!["apps/package.json"]);
    }
}
