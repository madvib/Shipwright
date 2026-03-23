//! Job queue CLI — create, list, update.

use anyhow::Result;
use runtime::db::jobs;

use crate::paths::project_ship_dir_required;

pub fn create(
    kind: &str,
    title: &str,
    milestone: Option<&str>,
    description: Option<&str>,
    branch: Option<&str>,
) -> Result<()> {
    let _ship_dir = project_ship_dir_required()?;
    let mut payload = serde_json::json!({ "title": title });
    if let Some(m) = milestone {
        payload["milestone"] = serde_json::Value::String(m.to_string());
    }
    if let Some(d) = description {
        payload["description"] = serde_json::Value::String(d.to_string());
    }
    let job = jobs::create_job(
        kind,
        branch,
        Some(payload),
        Some("human"),
        None,
        0,
        None,
        vec![],
        vec![],
    )?;
    println!("{}\t[{}]\t{}", job.id, job.kind, title);
    Ok(())
}

pub fn list(status: Option<&str>, branch: Option<&str>, milestone: Option<&str>) -> Result<()> {
    let _ship_dir = project_ship_dir_required()?;
    let all = jobs::list_jobs(branch, status)?;

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

    let mut sorted = filtered.clone();
    sorted.sort_by(|a, b| a.created_at.cmp(&b.created_at));

    for job in sorted {
        let desc = job
            .payload
            .get("description")
            .and_then(|v| v.as_str())
            .or_else(|| job.payload.get("title").and_then(|v| v.as_str()))
            .unwrap_or("");
        let desc_trunc = if desc.len() > 50 {
            format!("{}…", &desc[..50])
        } else {
            desc.to_string()
        };
        let date = job.created_at.get(..10).unwrap_or(&job.created_at);
        println!(
            "{}\t{}\t{}\t{}\t{}",
            job.id, job.status, job.kind, desc_trunc, date
        );
    }
    Ok(())
}

pub fn update(id_prefix: &str, status: &str) -> Result<()> {
    let _ship_dir = project_ship_dir_required()?;
    let all = jobs::list_jobs(None, None)?;
    let matched: Vec<_> = all.iter().filter(|j| j.id.starts_with(id_prefix)).collect();
    match matched.len() {
        0 => anyhow::bail!("No job matching '{}'", id_prefix),
        1 => {
            jobs::update_job_status(&matched[0].id, status)?;
            println!("✓ {} → {}", &matched[0].id[..8], status);
        }
        _ => anyhow::bail!(
            "Ambiguous prefix '{}' — {} matches",
            id_prefix,
            matched.len()
        ),
    }
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use runtime::db::{ensure_db, jobs};
    use runtime::project::init_project;
    use std::path::Path;
    use tempfile::tempdir;

    fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
        let tmp = tempdir().unwrap();
        let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
        ensure_db().unwrap();
        (tmp, ship_dir)
    }

    fn mk_job(_ship_dir: &Path, title: &str) -> jobs::Job {
        let payload = serde_json::json!({ "title": title, "description": "test job" });
        jobs::create_job(
            "test",
            None,
            Some(payload),
            Some("test"),
            None,
            0,
            None,
            vec![],
            vec![],
        )
        .unwrap()
    }

    #[test]
    fn update_job_status_complete() {
        let (_tmp, ship_dir) = setup();
        let job = mk_job(&ship_dir, "a job");
        jobs::update_job_status(&job.id, "complete").unwrap();
        let j = jobs::get_job(&job.id).unwrap().unwrap();
        assert_eq!(j.status, "complete");
    }

    #[test]
    fn update_job_status_pending_reset() {
        let (_tmp, ship_dir) = setup();
        let job = mk_job(&ship_dir, "failed job");
        jobs::update_job_status(&job.id, "failed").unwrap();
        jobs::update_job_status(&job.id, "pending").unwrap();
        let j = jobs::get_job(&job.id).unwrap().unwrap();
        assert_eq!(j.status, "pending");
    }

    #[test]
    fn claim_job_is_atomic() {
        let (_tmp, ship_dir) = setup();
        let job = mk_job(&ship_dir, "race");
        let first = jobs::claim_job(&job.id, "a").unwrap();
        let second = jobs::claim_job(&job.id, "b").unwrap();
        assert!(first);
        assert!(!second);
        let j = jobs::get_job(&job.id).unwrap().unwrap();
        assert_eq!(j.claimed_by, Some("a".to_string()));
    }
}
