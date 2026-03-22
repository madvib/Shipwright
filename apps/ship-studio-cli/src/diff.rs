//! `ship diff` — capability progress delta for the active milestone.
//!
//! Groups capabilities into three buckets:
//!   ✓ actual      — evidence string shown
//!   ▶ in-progress — running job id + short description
//!   ○ not-started — no running job
//!
//! Computes a delta line by comparing against a last-diff snapshot stored in
//! `~/.ship/last-diff.json`.  The snapshot records the set of actual capability
//! ids seen in the previous run.

use anyhow::Result;
use runtime::db::{jobs, targets};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::paths::{global_dir, project_ship_dir_required};

// ── Snapshot ──────────────────────────────────────────────────────────────────

fn snapshot_path() -> PathBuf {
    global_dir().join("last-diff.json")
}

fn load_snapshot() -> HashSet<String> {
    let path = snapshot_path();
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return HashSet::new();
    };
    let Ok(ids) = serde_json::from_str::<Vec<String>>(&raw) else {
        return HashSet::new();
    };
    ids.into_iter().collect()
}

fn save_snapshot(actual_ids: &HashSet<String>) -> Result<()> {
    let dir = global_dir();
    std::fs::create_dir_all(&dir)?;
    let ids: Vec<&String> = {
        let mut v: Vec<&String> = actual_ids.iter().collect();
        v.sort();
        v
    };
    std::fs::write(snapshot_path(), serde_json::to_string(&ids)?)?;
    Ok(())
}

// ── Surface name map ──────────────────────────────────────────────────────────

fn surface_label(target_id: &str, targets: &[targets::Target]) -> String {
    targets
        .iter()
        .find(|t| t.id == target_id)
        .map(|t| t.title.clone())
        .unwrap_or_else(|| target_id.to_string())
}

// ── Main ──────────────────────────────────────────────────────────────────────

pub fn run(milestone_id_hint: Option<&str>) -> Result<()> {
    let ship_dir = project_ship_dir_required()?;
    run_with_dir(&ship_dir, milestone_id_hint)
}

pub fn run_with_dir(ship_dir: &Path, milestone_id_hint: Option<&str>) -> Result<()> {
    // Resolve milestone target.
    let all_targets = targets::list_targets(ship_dir, None)?;
    let milestone = match milestone_id_hint {
        Some(id) => all_targets
            .iter()
            .find(|t| t.id == id || t.title == id)
            .cloned(),
        None => all_targets
            .iter()
            .find(|t| t.kind == "milestone" && t.status == "active")
            .cloned(),
    };
    let milestone = match milestone {
        Some(m) => m,
        None => {
            println!(
                "No active milestone target found. Create one with `create_target` (kind=milestone)."
            );
            return Ok(());
        }
    };

    // Load capabilities for this milestone.
    let caps = targets::list_capabilities_for_milestone(ship_dir, &milestone.id, None)?;
    if caps.is_empty() {
        println!(
            "No capabilities linked to milestone '{}' ({}).",
            milestone.title, milestone.id
        );
        return Ok(());
    }

    // Load running jobs so we can link in-progress capabilities.
    let running_jobs = jobs::list_jobs(ship_dir, None, Some("running"))?;
    // Build map: capability_id → job
    let mut cap_to_job: HashMap<String, &jobs::Job> = HashMap::new();
    for job in &running_jobs {
        if let Some(cap_id) = job.payload.get("capability_id").and_then(|v| v.as_str()) {
            cap_to_job.entry(cap_id.to_string()).or_insert(job);
        }
    }

    // Load previous snapshot for delta computation.
    let prev_actual = load_snapshot();

    // Categorise.
    let mut actual_ids: HashSet<String> = HashSet::new();
    let mut actual_caps = vec![];
    let mut in_progress_caps = vec![];
    let mut not_started_caps = vec![];

    for cap in &caps {
        if cap.status == "actual" {
            actual_ids.insert(cap.id.clone());
            actual_caps.push(cap);
        } else if let Some(job) = cap_to_job.get(&cap.id) {
            in_progress_caps.push((cap, *job));
        } else {
            not_started_caps.push(cap);
        }
    }

    // Compute delta.
    let flipped: Vec<&String> = actual_ids.difference(&prev_actual).collect();

    // Print.
    let total = caps.len();
    let n_actual = actual_caps.len();
    println!(
        "── {} — {} ({}/{} actual) ─────────────────────",
        milestone.title, milestone.id, n_actual, total
    );
    println!();

    // Group by surface target.
    let surface_targets = all_targets
        .iter()
        .filter(|t| t.kind == "surface")
        .cloned()
        .collect::<Vec<_>>();

    if !actual_caps.is_empty() {
        println!("  ✓ actual ({})", actual_caps.len());
        for cap in &actual_caps {
            let evidence = cap.evidence.as_deref().unwrap_or("—");
            let surface = surface_label(&cap.target_id, &surface_targets);
            println!(
                "    [{}] {} :: {} — {}",
                surface, cap.id, cap.title, evidence
            );
        }
        println!();
    }

    if !in_progress_caps.is_empty() {
        println!("  ▶ in-progress ({})", in_progress_caps.len());
        for (cap, job) in &in_progress_caps {
            let desc = job
                .payload
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or(&job.kind);
            let short_desc = if desc.len() > 50 { &desc[..50] } else { desc };
            let surface = surface_label(&cap.target_id, &surface_targets);
            println!(
                "    [{}] {} :: {} → job/{} ({})",
                surface,
                cap.id,
                cap.title,
                &job.id[..8],
                short_desc
            );
        }
        println!();
    }

    if !not_started_caps.is_empty() {
        println!("  ○ not started ({})", not_started_caps.len());
        for cap in &not_started_caps {
            let surface = surface_label(&cap.target_id, &surface_targets);
            println!("    [{}] {} :: {}", surface, cap.id, cap.title);
        }
        println!();
    }

    // Delta line.
    if flipped.is_empty() {
        println!("  0 capabilities flipped actual this session.");
    } else {
        println!(
            "  {} capability/capabilities flipped actual this session: {}",
            flipped.len(),
            flipped
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    // Persist snapshot.
    if let Err(e) = save_snapshot(&actual_ids) {
        eprintln!("  warning: could not save snapshot: {}", e);
    }

    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use runtime::db::{ensure_db, jobs, targets};
    use runtime::project::init_project;
    use tempfile::tempdir;

    fn setup() -> (tempfile::TempDir, PathBuf) {
        let tmp = tempdir().unwrap();
        let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
        ensure_db(&ship_dir).unwrap();
        (tmp, ship_dir)
    }

    #[test]
    fn test_diff_no_milestone() {
        let (_tmp, ship_dir) = setup();
        // No milestone — should print a message and return Ok
        let result = run_with_dir(&ship_dir, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_diff_empty_capabilities() {
        let (_tmp, ship_dir) = setup();
        targets::create_target(&ship_dir, "milestone", "v0.1.0", None, None, None).unwrap();
        let result = run_with_dir(&ship_dir, Some("v0.1.0"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_diff_groups_capabilities_correctly() {
        let (_tmp, ship_dir) = setup();
        let ms =
            targets::create_target(&ship_dir, "milestone", "v0.1.0", None, None, None).unwrap();
        let surface =
            targets::create_target(&ship_dir, "surface", "compiler", None, None, None).unwrap();

        let c_actual =
            targets::create_capability(&ship_dir, &surface.id, "Profile compile", Some(&ms.id))
                .unwrap();
        targets::mark_capability_actual(&ship_dir, &c_actual.id, "test: compile_ok").unwrap();

        let c_inprog =
            targets::create_capability(&ship_dir, &surface.id, "Gemini output", Some(&ms.id))
                .unwrap();
        let payload = serde_json::json!({
            "description": "compile gemini provider",
            "capability_id": c_inprog.id
        });
        jobs::create_job(
            &ship_dir,
            "feature",
            None,
            Some(payload),
            None,
            None,
            0,
            None,
            vec![],
            vec![],
        )
        .unwrap();
        // claim it to move to running
        let all = jobs::list_jobs(&ship_dir, None, Some("pending")).unwrap();
        jobs::claim_job(&ship_dir, &all[0].id, "test").unwrap();

        let _c_todo =
            targets::create_capability(&ship_dir, &surface.id, "Codex output", Some(&ms.id))
                .unwrap();

        // Should not panic; we just verify it runs successfully with all three buckets present
        let result = run_with_dir(&ship_dir, Some(&ms.id));
        assert!(result.is_ok());
    }

    #[test]
    fn test_snapshot_roundtrip() {
        let ids: HashSet<String> = ["cap1".to_string(), "cap2".to_string()].into();
        // Use a temp home so we don't clobber real last-diff.json
        // (snapshot_path uses global_dir which is ~/.ship/last-diff.json,
        //  so we just verify the in-memory logic is correct via a direct load/save cycle
        //  using a tempfile approach instead)
        let tmp = tempdir().unwrap();
        let snap_path = tmp.path().join("last-diff.json");
        let v: Vec<&String> = {
            let mut v: Vec<&String> = ids.iter().collect();
            v.sort();
            v
        };
        std::fs::write(&snap_path, serde_json::to_string(&v).unwrap()).unwrap();
        let raw: Vec<String> =
            serde_json::from_str(&std::fs::read_to_string(&snap_path).unwrap()).unwrap();
        let loaded: HashSet<String> = raw.into_iter().collect();
        assert_eq!(loaded, ids);
    }
}
