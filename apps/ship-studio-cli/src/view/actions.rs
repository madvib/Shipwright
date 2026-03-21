//! TUI mutation helpers — thin wrappers around runtime DB calls.
//!
//! All functions are infallible: they return a status message string
//! suitable for display in the TUI footer.

use runtime::db::{
    jobs::update_job_status,
    targets::{update_capability, CapabilityPatch},
};
use std::path::Path;

/// Cycle a job through: pending → running → complete → failed → pending.
pub fn cycle_job_status(ship_dir: &Path, job_id: &str, current: &str) -> String {
    let next = match current {
        "pending" => "running",
        "running" => "complete",
        "complete" => "failed",
        "failed" => "pending",
        _ => "pending",
    };
    match update_job_status(ship_dir, job_id, next) {
        Ok(()) => format!("job → {next}"),
        Err(e) => format!("error: {e}"),
    }
}

/// Cycle a capability through: aspirational → in_progress → actual → aspirational.
pub fn cycle_cap_status(ship_dir: &Path, cap_id: &str, current: &str) -> String {
    let next = match current {
        "aspirational" => "in_progress",
        "in_progress" => "actual",
        "actual" => "aspirational",
        _ => "aspirational",
    };
    let patch = CapabilityPatch {
        status: Some(next.to_string()),
        title: None,
        phase: None,
        acceptance_criteria: None,
        preset_hint: None,
        file_scope: None,
        assigned_to: None,
        priority: None,
    };
    match update_capability(ship_dir, cap_id, patch) {
        Ok(()) => format!("cap → {next}"),
        Err(e) => format!("error: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use runtime::db::{ensure_db, jobs::{create_job, get_job}};
    use runtime::db::targets::{create_target, create_capability, get_capability};
    use runtime::project::init_project;
    use tempfile::TempDir;

    fn setup() -> (TempDir, std::path::PathBuf) {
        let tmp = TempDir::new().unwrap();
        let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
        ensure_db(&ship_dir).unwrap();
        (tmp, ship_dir)
    }

    #[test]
    fn status_cycling_job_and_capability() {
        let (_tmp, ship_dir) = setup();

        // ── Job: full cycle pending → running → complete → failed → pending ──
        let job = create_job(
            &ship_dir, "test", None, None, None, None, 0, None,
            vec![], vec![],
        ).unwrap();
        assert_eq!(job.status, "pending");

        let transitions = [
            ("pending", "running"),
            ("running", "complete"),
            ("complete", "failed"),
            ("failed", "pending"),
        ];
        let mut current = job.status.clone();
        for (from, to) in &transitions {
            assert_eq!(&current, from);
            let msg = cycle_job_status(&ship_dir, &job.id, &current);
            assert_eq!(msg, format!("job → {to}"));
            current = get_job(&ship_dir, &job.id).unwrap().unwrap().status;
            assert_eq!(&current, to);
        }

        // ── Capability: aspirational → in_progress → actual → aspirational ──
        let target = create_target(&ship_dir, "milestone", "Test Target", None, None, None)
            .unwrap();
        let cap = create_capability(&ship_dir, &target.id, "Test Cap", None)
            .unwrap();
        assert_eq!(cap.status, "aspirational");

        let cap_transitions = [
            ("aspirational", "in_progress"),
            ("in_progress", "actual"),
            ("actual", "aspirational"),
        ];
        let mut cap_status = cap.status.clone();
        for (from, to) in &cap_transitions {
            assert_eq!(&cap_status, from);
            let msg = cycle_cap_status(&ship_dir, &cap.id, &cap_status);
            assert_eq!(msg, format!("cap → {to}"));
            cap_status = get_capability(&ship_dir, &cap.id).unwrap().unwrap().status;
            assert_eq!(&cap_status, to);
        }
    }
}
