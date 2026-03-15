use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Duration, Utc};
use runtime::Plugin;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

// ─── Data Structures ──────────────────────────────────────────────────────────

/// A timer that is currently running.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActiveTimer {
    /// Filename of the issue being timed (e.g. "my-feature.md")
    pub issue_file: String,
    /// Human-readable issue title
    pub issue_title: String,
    pub started_at: DateTime<Utc>,
    pub note: Option<String>,
}

/// A completed time entry.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TimeEntry {
    pub issue_file: String,
    pub issue_title: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    /// Duration in whole minutes
    pub duration_minutes: u64,
    pub note: Option<String>,
}

impl TimeEntry {
    pub fn duration_display(&self) -> String {
        format_duration(self.duration_minutes)
    }
}

pub struct TimeTracker;

impl Plugin for TimeTracker {
    fn name(&self) -> &str {
        "time-tracker"
    }

    fn description(&self) -> &str {
        "Track time spent on issues"
    }
}

// ─── Storage Helpers ──────────────────────────────────────────────────────────

fn plugin_dir(project_dir: &Path) -> PathBuf {
    project_dir.join("time-tracker")
}

fn active_timer_path(project_dir: &Path) -> PathBuf {
    plugin_dir(project_dir).join("active.json")
}

fn entries_path(project_dir: &Path) -> PathBuf {
    plugin_dir(project_dir).join("entries.json")
}

fn ensure_dir(project_dir: &Path) -> Result<()> {
    fs::create_dir_all(plugin_dir(project_dir))
        .context("Failed to create time-tracker plugin directory")
}

fn load_entries(project_dir: &Path) -> Result<Vec<TimeEntry>> {
    let path = entries_path(project_dir);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&content).unwrap_or_default())
}

fn save_entries(project_dir: &Path, entries: &[TimeEntry]) -> Result<()> {
    let json = serde_json::to_string_pretty(entries)?;
    fs::write(entries_path(project_dir), json)?;
    Ok(())
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Returns the currently running timer, if any.
pub fn get_active_timer(project_dir: &Path) -> Result<Option<ActiveTimer>> {
    let path = active_timer_path(project_dir);
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path)?;
    Ok(Some(serde_json::from_str(&content)?))
}

/// Start a timer for the given issue. Errors if a timer is already running.
pub fn start_timer(
    project_dir: &Path,
    issue_file: &str,
    issue_title: &str,
    note: Option<String>,
) -> Result<ActiveTimer> {
    ensure_dir(project_dir)?;

    if let Some(active) = get_active_timer(project_dir)? {
        return Err(anyhow!(
            "Timer already running for '{}'. Stop it first with `ship time stop`.",
            active.issue_title
        ));
    }

    let timer = ActiveTimer {
        issue_file: issue_file.to_string(),
        issue_title: issue_title.to_string(),
        started_at: Utc::now(),
        note,
    };

    let json = serde_json::to_string_pretty(&timer)?;
    fs::write(active_timer_path(project_dir), json)?;

    Ok(timer)
}

/// Stop the running timer and save the entry. Returns the completed entry.
pub fn stop_timer(project_dir: &Path, note: Option<String>) -> Result<TimeEntry> {
    let active =
        get_active_timer(project_dir)?.ok_or_else(|| anyhow!("No timer is currently running."))?;

    let ended_at = Utc::now();
    let duration = ended_at - active.started_at;
    let duration_minutes = duration.num_minutes().max(0) as u64;

    let entry = TimeEntry {
        issue_file: active.issue_file,
        issue_title: active.issue_title,
        started_at: active.started_at,
        ended_at,
        duration_minutes,
        note: note.or(active.note),
    };

    // Append to entries
    let mut entries = load_entries(project_dir)?;
    entries.push(entry.clone());
    save_entries(project_dir, &entries)?;

    // Remove active timer
    fs::remove_file(active_timer_path(project_dir))?;

    Ok(entry)
}

/// Manually log time for an issue without using start/stop.
pub fn log_time(
    project_dir: &Path,
    issue_file: &str,
    issue_title: &str,
    duration_minutes: u64,
    note: Option<String>,
) -> Result<TimeEntry> {
    ensure_dir(project_dir)?;

    let ended_at = Utc::now();
    let started_at = ended_at - Duration::minutes(duration_minutes as i64);

    let entry = TimeEntry {
        issue_file: issue_file.to_string(),
        issue_title: issue_title.to_string(),
        started_at,
        ended_at,
        duration_minutes,
        note,
    };

    let mut entries = load_entries(project_dir)?;
    entries.push(entry.clone());
    save_entries(project_dir, &entries)?;

    Ok(entry)
}

/// List all time entries, optionally filtered to a specific issue file.
pub fn list_entries(project_dir: &Path, issue_file: Option<&str>) -> Result<Vec<TimeEntry>> {
    let entries = load_entries(project_dir)?;
    Ok(match issue_file {
        Some(f) => entries.into_iter().filter(|e| e.issue_file == f).collect(),
        None => entries,
    })
}

/// Generate a Markdown time report for the project.
pub fn generate_report(project_dir: &Path) -> Result<String> {
    let entries = load_entries(project_dir)?;
    let active = get_active_timer(project_dir)?;

    let mut report = String::new();

    let project_name = project_dir
        .parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Project".to_string());

    report.push_str(&format!("# Time Report — {}\n\n", project_name));
    report.push_str(&format!(
        "Generated: {}\n\n",
        Utc::now().format("%Y-%m-%d %H:%M UTC")
    ));

    // Active timer
    if let Some(ref t) = active {
        let elapsed = (Utc::now() - t.started_at).num_minutes().max(0) as u64;
        report.push_str(&format!(
            "**Active Timer:** {} — running for {}\n\n",
            t.issue_title,
            format_duration(elapsed)
        ));
    }

    if entries.is_empty() {
        report.push_str("No time entries recorded yet.\n");
        return Ok(report);
    }

    // Totals
    let total_minutes: u64 = entries.iter().map(|e| e.duration_minutes).sum();
    report.push_str(&format!(
        "**Total tracked:** {}\n\n",
        format_duration(total_minutes)
    ));

    // By issue
    report.push_str("## By Issue\n\n");
    let mut by_issue: Vec<(String, u64, usize)> = {
        let mut map: std::collections::HashMap<String, (u64, usize)> =
            std::collections::HashMap::new();
        for e in &entries {
            let entry = map.entry(e.issue_title.clone()).or_insert((0, 0));
            entry.0 += e.duration_minutes;
            entry.1 += 1;
        }
        map.into_iter()
            .map(|(title, (mins, count))| (title, mins, count))
            .collect()
    };
    by_issue.sort_by(|a, b| b.1.cmp(&a.1));

    for (title, mins, sessions) in &by_issue {
        report.push_str(&format!(
            "- **{}** — {} ({} session{})\n",
            title,
            format_duration(*mins),
            sessions,
            if *sessions == 1 { "" } else { "s" }
        ));
    }

    // Recent sessions
    report.push_str("\n## Recent Sessions\n\n");
    let recent: Vec<&TimeEntry> = entries.iter().rev().take(10).collect();
    for e in recent {
        report.push_str(&format!(
            "- `{}` → `{}`  **{}**  {}  {}\n",
            e.started_at.format("%Y-%m-%d %H:%M"),
            e.ended_at.format("%H:%M"),
            e.issue_title,
            format_duration(e.duration_minutes),
            e.note.as_deref().unwrap_or(""),
        ));
    }

    Ok(report)
}

// ─── Formatting ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ship_module_project::init_project;
    use tempfile::tempdir;

    fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
        let tmp = tempdir().unwrap();
        let project_dir = init_project(tmp.path().to_path_buf()).unwrap();
        (tmp, project_dir)
    }

    #[test]
    fn test_start_and_stop_timer() {
        let (_tmp, project_dir) = setup();
        assert!(get_active_timer(&project_dir).unwrap().is_none());

        start_timer(&project_dir, "my-issue.md", "My Issue", None).unwrap();
        let active = get_active_timer(&project_dir).unwrap();
        assert!(active.is_some());
        assert_eq!(active.unwrap().issue_title, "My Issue");

        let entry = stop_timer(&project_dir, Some("done!".to_string())).unwrap();
        assert_eq!(entry.issue_title, "My Issue");
        assert_eq!(entry.note.as_deref(), Some("done!"));
        assert!(get_active_timer(&project_dir).unwrap().is_none());
    }

    #[test]
    fn test_cannot_start_timer_while_running() {
        let (_tmp, project_dir) = setup();
        start_timer(&project_dir, "a.md", "A", None).unwrap();
        assert!(start_timer(&project_dir, "b.md", "B", None).is_err());
    }

    #[test]
    fn test_stop_without_active_timer_errors() {
        let (_tmp, project_dir) = setup();
        assert!(stop_timer(&project_dir, None).is_err());
    }

    #[test]
    fn test_log_time_manual() {
        let (_tmp, project_dir) = setup();
        let entry = log_time(
            &project_dir,
            "issue.md",
            "Issue",
            90,
            Some("manual".to_string()),
        )
        .unwrap();
        assert_eq!(entry.duration_minutes, 90);
        let entries = list_entries(&project_dir, None).unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_list_entries_filtered() {
        let (_tmp, project_dir) = setup();
        log_time(&project_dir, "a.md", "A", 30, None).unwrap();
        log_time(&project_dir, "b.md", "B", 45, None).unwrap();

        let all = list_entries(&project_dir, None).unwrap();
        assert_eq!(all.len(), 2);

        let filtered = list_entries(&project_dir, Some("a.md")).unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].issue_file, "a.md");
    }

    #[test]
    fn test_generate_report_empty() {
        let (_tmp, project_dir) = setup();
        let report = generate_report(&project_dir).unwrap();
        assert!(report.contains("No time entries"));
    }

    #[test]
    fn test_generate_report_with_entries() {
        let (_tmp, project_dir) = setup();
        log_time(&project_dir, "a.md", "Feature A", 120, None).unwrap();
        log_time(&project_dir, "a.md", "Feature A", 30, None).unwrap();
        log_time(&project_dir, "b.md", "Bug Fix", 45, None).unwrap();

        let report = generate_report(&project_dir).unwrap();
        assert!(report.contains("Feature A"));
        assert!(report.contains("Bug Fix"));
        assert!(report.contains("3h 15m")); // total
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0), "< 1m");
        assert_eq!(format_duration(45), "45m");
        assert_eq!(format_duration(60), "1h");
        assert_eq!(format_duration(90), "1h 30m");
        assert_eq!(format_duration(120), "2h");
    }
}

pub fn format_duration(minutes: u64) -> String {
    if minutes == 0 {
        return "< 1m".to_string();
    }
    let h = minutes / 60;
    let m = minutes % 60;
    match (h, m) {
        (0, m) => format!("{}m", m),
        (h, 0) => format!("{}h", h),
        (h, m) => format!("{}h {}m", h, m),
    }
}
