//! Job queue CLI — create, list, and update jobs for agent coordination.

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
    let ship_dir = project_ship_dir_required()?;
    let mut payload = serde_json::json!({ "title": title });
    if let Some(m) = milestone {
        payload["milestone"] = serde_json::Value::String(m.to_string());
    }
    if let Some(d) = description {
        payload["description"] = serde_json::Value::String(d.to_string());
    }
    let job = jobs::create_job(&ship_dir, kind, branch, Some(payload), Some("human"), None, 0, None, vec![])?;
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

    // Print oldest-first for queue ordering
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
    if matches!(job.status.as_str(), "complete" | "failed" | "done") {
        anyhow::bail!("Job {} is already {}", &job.id[..8], job.status);
    }

    // Stage files in job's declared scope
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

    // Commit with job reference
    let desc = job.payload.get("description")
        .and_then(|v| v.as_str())
        .unwrap_or(&job.kind);
    let commit_msg = format!("{} (job/{})", desc, job.id);
    let out = std::process::Command::new("git")
        .args(["commit", "-m", &commit_msg])
        .output()?;
    if !out.status.success() {
        anyhow::bail!("git commit failed: {}", String::from_utf8_lossy(&out.stderr).trim());
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
    // Allow prefix match
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
