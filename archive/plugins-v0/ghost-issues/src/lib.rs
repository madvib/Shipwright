use anyhow::Result;
use chrono::{DateTime, Utc};
use ignore::WalkBuilder;
use regex::Regex;
use runtime::Plugin;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

// ─── Data Structures ──────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum GhostKind {
    Todo,
    Fixme,
    Hack,
    Bug,
}

impl GhostKind {
    pub fn as_str(&self) -> &str {
        match self {
            GhostKind::Todo => "TODO",
            GhostKind::Fixme => "FIXME",
            GhostKind::Hack => "HACK",
            GhostKind::Bug => "BUG",
        }
    }
}

/// A comment found in the codebase that represents a potential issue.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GhostIssue {
    pub kind: GhostKind,
    /// The comment text after the marker
    pub text: String,
    /// Relative path to the file
    pub file: String,
    pub line: usize,
    /// Whether this has already been promoted to a real issue
    pub promoted: bool,
    pub found_at: DateTime<Utc>,
}

impl GhostIssue {
    pub fn suggested_title(&self) -> String {
        let text = self.text.trim();
        let title = if text.len() > 60 {
            format!("{}...", &text[..57])
        } else {
            text.to_string()
        };
        if title.is_empty() {
            format!("{} in {}", self.kind.as_str(), self.file)
        } else {
            title
        }
    }

    pub fn display(&self) -> String {
        format!(
            "[{}] {}:{} — {}",
            self.kind.as_str(),
            self.file,
            self.line,
            self.text.trim()
        )
    }
}

/// Scan results with metadata.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScanResult {
    pub scanned_at: DateTime<Utc>,
    pub root: String,
    pub issues: Vec<GhostIssue>,
}

// ─── Plugin Implementation ────────────────────────────────────────────────────

pub struct GhostIssues;

impl Plugin for GhostIssues {
    fn name(&self) -> &str {
        "ghost-issues"
    }

    fn description(&self) -> &str {
        "Scan codebase for TODO/FIXME/HACK/BUG comments and surface as suggested issues"
    }
}

// ─── Storage ─────────────────────────────────────────────────────────────────

fn plugin_dir(project_dir: &Path) -> PathBuf {
    project_dir.join("ghost-issues")
}

fn scan_result_path(project_dir: &Path) -> PathBuf {
    plugin_dir(project_dir).join("last-scan.json")
}

pub fn load_last_scan(project_dir: &Path) -> Result<Option<ScanResult>> {
    let path = scan_result_path(project_dir);
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path)?;
    Ok(Some(serde_json::from_str(&content)?))
}

fn save_scan(project_dir: &Path, result: &ScanResult) -> Result<()> {
    fs::create_dir_all(plugin_dir(project_dir))?;
    let json = serde_json::to_string_pretty(result)?;
    fs::write(scan_result_path(project_dir), json)?;
    Ok(())
}

// ─── Scanning ────────────────────────────────────────────────────────────────

// Matches: TODO, FIXME, HACK, BUG — with optional colon, author, and text
// Examples:
//   // TODO: fix this
//   // FIXME(alice): broken
//   # TODO something
static PATTERN: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();

fn get_pattern() -> &'static Regex {
    PATTERN.get_or_init(|| {
        Regex::new(r"(?i)\b(TODO|FIXME|HACK|BUG)\b(?:\([^)]*\))?:?\s*(.*)").expect("valid regex")
    })
}

fn classify(marker: &str) -> GhostKind {
    match marker.to_uppercase().as_str() {
        "FIXME" => GhostKind::Fixme,
        "HACK" => GhostKind::Hack,
        "BUG" => GhostKind::Bug,
        _ => GhostKind::Todo,
    }
}

/// Scan `root_dir` for ghost issues. Respects `.gitignore` and skips `.ship/` and `target/`.
pub fn scan(project_dir: &Path, root_dir: &Path) -> Result<ScanResult> {
    let pattern = get_pattern();
    let mut ghost_issues = Vec::new();

    let walker = WalkBuilder::new(root_dir)
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            // Skip build artifacts, plugin data, and binary dirs
            !matches!(
                name.as_ref(),
                "target" | ".ship" | "node_modules" | ".git" | "dist"
            )
        })
        .build();

    for entry in walker {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        // Only scan text-like files
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !is_text_file(ext) {
            continue;
        }

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue, // skip binary files that slipped through
        };

        let rel_path = path
            .strip_prefix(root_dir)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        for (line_num, line) in content.lines().enumerate() {
            if let Some(caps) = pattern.captures(line) {
                let marker = caps.get(1).map_or("", |m| m.as_str());
                let text = caps.get(2).map_or("", |m| m.as_str()).trim().to_string();
                ghost_issues.push(GhostIssue {
                    kind: classify(marker),
                    text,
                    file: rel_path.clone(),
                    line: line_num + 1,
                    promoted: false,
                    found_at: Utc::now(),
                });
            }
        }
    }

    let result = ScanResult {
        scanned_at: Utc::now(),
        root: root_dir.to_string_lossy().to_string(),
        issues: ghost_issues,
    };

    save_scan(project_dir, &result)?;
    Ok(result)
}

/// Mark a ghost issue as promoted (i.e. it's been turned into a real issue).
pub fn mark_promoted(project_dir: &Path, file: &str, line: usize) -> Result<bool> {
    let Some(mut scan_result) = load_last_scan(project_dir)? else {
        return Ok(false);
    };
    let mut found = false;
    for g in &mut scan_result.issues {
        if g.file == file && g.line == line {
            g.promoted = true;
            found = true;
        }
    }
    if found {
        save_scan(project_dir, &scan_result)?;
    }
    Ok(found)
}

/// Generate a Markdown summary of the last scan.
pub fn generate_report(project_dir: &Path) -> Result<String> {
    let Some(result) = load_last_scan(project_dir)? else {
        return Ok("No scan results found. Run `ship ghost scan` first.".to_string());
    };

    let unpromoted: Vec<&GhostIssue> = result.issues.iter().filter(|g| !g.promoted).collect();

    let mut report = String::new();
    report.push_str(&format!(
        "# Ghost Issues — {}\n\n",
        result.scanned_at.format("%Y-%m-%d %H:%M UTC")
    ));
    report.push_str(&format!(
        "Found **{}** items ({} already promoted)\n\n",
        result.issues.len(),
        result.issues.iter().filter(|g| g.promoted).count()
    ));

    if unpromoted.is_empty() {
        report.push_str("No unpromoted ghost issues.\n");
        return Ok(report);
    }

    // Group by kind
    for kind in [
        GhostKind::Fixme,
        GhostKind::Bug,
        GhostKind::Todo,
        GhostKind::Hack,
    ] {
        let group: Vec<&&GhostIssue> = unpromoted.iter().filter(|g| g.kind == kind).collect();
        if group.is_empty() {
            continue;
        }
        report.push_str(&format!("## {} ({})\n\n", kind.as_str(), group.len()));
        for g in group {
            report.push_str(&format!("- `{}:{}` {}\n", g.file, g.line, g.text.trim()));
        }
        report.push('\n');
    }

    Ok(report)
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ship_module_project::init_project;
    use std::fs;
    use tempfile::tempdir;

    fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
        let tmp = tempdir().unwrap();
        let project_dir = init_project(tmp.path().to_path_buf()).unwrap();
        (tmp, project_dir)
    }

    fn write_file(root: &std::path::Path, name: &str, content: &str) {
        fs::write(root.join(name), content).unwrap();
    }

    #[test]
    fn test_scan_finds_todos() {
        let (tmp, project_dir) = setup();
        write_file(
            tmp.path(),
            "main.rs",
            "fn main() {\n    // TODO: implement this\n    // FIXME: broken\n}\n",
        );
        let result = scan(&project_dir, tmp.path()).unwrap();
        let kinds: Vec<&str> = result.issues.iter().map(|g| g.kind.as_str()).collect();
        assert!(kinds.contains(&"TODO"));
        assert!(kinds.contains(&"FIXME"));
    }

    #[test]
    fn test_scan_captures_text() {
        let (tmp, project_dir) = setup();
        write_file(tmp.path(), "lib.rs", "// TODO: add error handling\n");
        let result = scan(&project_dir, tmp.path()).unwrap();
        let todo = result
            .issues
            .iter()
            .find(|g| g.kind == GhostKind::Todo)
            .unwrap();
        assert_eq!(todo.text, "add error handling");
        assert_eq!(todo.line, 1);
    }

    #[test]
    fn test_scan_persists_results() {
        let (tmp, project_dir) = setup();
        write_file(tmp.path(), "a.rs", "// HACK: workaround\n");
        scan(&project_dir, tmp.path()).unwrap();
        let loaded = load_last_scan(&project_dir).unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().issues.len(), 1);
    }

    #[test]
    fn test_promote_marks_issue() {
        let (tmp, project_dir) = setup();
        write_file(tmp.path(), "main.rs", "// TODO: fix me\n");
        scan(&project_dir, tmp.path()).unwrap();

        let found = mark_promoted(&project_dir, "main.rs", 1).unwrap();
        assert!(found);

        let scan_result = load_last_scan(&project_dir).unwrap().unwrap();
        assert!(scan_result.issues[0].promoted);
    }

    #[test]
    fn test_suggested_title_short() {
        let g = GhostIssue {
            kind: GhostKind::Todo,
            text: "short text".to_string(),
            file: "a.rs".to_string(),
            line: 1,
            promoted: false,
            found_at: Utc::now(),
        };
        assert_eq!(g.suggested_title(), "short text");
    }

    #[test]
    fn test_suggested_title_truncated() {
        let long = "a".repeat(70);
        let g = GhostIssue {
            kind: GhostKind::Todo,
            text: long,
            file: "a.rs".to_string(),
            line: 1,
            promoted: false,
            found_at: Utc::now(),
        };
        assert!(g.suggested_title().len() <= 60);
        assert!(g.suggested_title().ends_with("..."));
    }

    #[test]
    fn test_report_empty() {
        let (_tmp, project_dir) = setup();
        let report = generate_report(&project_dir).unwrap();
        assert!(report.contains("No scan results"));
    }

    #[test]
    fn test_report_with_results() {
        let (tmp, project_dir) = setup();
        write_file(tmp.path(), "x.rs", "// TODO: one\n// FIXME: two\n");
        scan(&project_dir, tmp.path()).unwrap();
        let report = generate_report(&project_dir).unwrap();
        assert!(report.contains("TODO"));
        assert!(report.contains("FIXME"));
    }
}

fn is_text_file(ext: &str) -> bool {
    matches!(
        ext,
        "rs" | "ts"
            | "tsx"
            | "js"
            | "jsx"
            | "py"
            | "go"
            | "java"
            | "kt"
            | "swift"
            | "c"
            | "cpp"
            | "h"
            | "hpp"
            | "cs"
            | "rb"
            | "php"
            | "sh"
            | "bash"
            | "zsh"
            | "fish"
            | "toml"
            | "yaml"
            | "yml"
            | "json"
            | "md"
            | "txt"
            | "lua"
            | "vim"
            | "el"
            | "ex"
            | "exs"
            | "hs"
            | "ml"
            | "clj"
            | "scala"
            | "dart"
            | "r"
            | "jl"
            | "tf"
    )
}
