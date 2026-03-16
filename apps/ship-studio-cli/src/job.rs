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
    let job = jobs::create_job(&ship_dir, kind, branch, Some(payload), Some("human"))?;
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

    let mut current_milestone = String::new();
    // Print oldest-first for queue ordering
    let mut sorted = filtered.clone();
    sorted.sort_by(|a, b| a.created_at.cmp(&b.created_at));

    for job in sorted {
        let title = job.payload.get("title").and_then(|v| v.as_str()).unwrap_or(&job.kind);
        let ms = job.payload.get("milestone").and_then(|v| v.as_str()).unwrap_or("");
        if ms != current_milestone {
            if !current_milestone.is_empty() { println!(); }
            if !ms.is_empty() { println!("  {}", ms); }
            current_milestone = ms.to_string();
        }
        let status_icon = match job.status.as_str() {
            "pending"  => "○",
            "running"  => "●",
            "done"     => "✓",
            "blocked"  => "✗",
            _          => "?",
        };
        println!("  {} {}  {}", status_icon, &job.id[..8], title);
    }
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
