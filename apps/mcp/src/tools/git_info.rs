use std::path::Path;
use std::process::Command;

use serde::Serialize;

use crate::requests::{GetGitDiffRequest, GetGitLogRequest};

#[derive(Serialize)]
struct GitStatus {
    branch: String,
    clean: bool,
    staged: Vec<String>,
    modified: Vec<String>,
    untracked: Vec<String>,
}

#[derive(Serialize)]
struct GitLogEntry {
    hash: String,
    short_hash: String,
    message: String,
    author: String,
    date: String,
    files_changed: u32,
}

#[derive(Serialize)]
struct Worktree {
    path: String,
    branch: String,
    head: String,
}

fn run_git(project_dir: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(project_dir)
        .output()
        .map_err(|e| format!("Failed to run git: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git error: {}", stderr.trim()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub fn get_git_status(project_dir: &Path) -> String {
    let raw = match run_git(project_dir, &["status", "--porcelain", "-b"]) {
        Ok(s) => s,
        Err(e) => return format!("Error: {e}"),
    };
    let mut branch = String::new();
    let mut staged = Vec::new();
    let mut modified = Vec::new();
    let mut untracked = Vec::new();

    for line in raw.lines() {
        if line.starts_with("## ") {
            let rest = &line[3..];
            branch = rest
                .split("...")
                .next()
                .unwrap_or(rest)
                .split(' ')
                .next()
                .unwrap_or(rest)
                .to_string();
            continue;
        }
        if line.len() < 4 {
            continue;
        }
        let (x, y) = (line.as_bytes()[0], line.as_bytes()[1]);
        let file = line[3..].to_string();
        if x == b'?' && y == b'?' {
            untracked.push(file);
        } else {
            if x != b' ' && x != b'?' {
                staged.push(file.clone());
            }
            if y != b' ' && y != b'?' {
                modified.push(file);
            }
        }
    }

    let clean = staged.is_empty() && modified.is_empty() && untracked.is_empty();
    serde_json::to_string(&GitStatus {
        branch,
        clean,
        staged,
        modified,
        untracked,
    })
    .unwrap_or_else(|e| format!("Error: {e}"))
}

pub fn get_git_diff(project_dir: &Path, req: GetGitDiffRequest) -> String {
    let mut args = vec!["diff"];
    let base_owned: String;
    if let Some(ref base) = req.base {
        base_owned = base.clone();
        args.push(&base_owned);
    }
    let path_owned: String;
    if let Some(ref path) = req.path {
        path_owned = path.clone();
        args.push("--");
        args.push(&path_owned);
    }
    match run_git(project_dir, &args) {
        Ok(diff) if diff.is_empty() => "No differences found.".into(),
        Ok(diff) => diff,
        Err(e) => format!("Error: {e}"),
    }
}

pub fn get_git_log(project_dir: &Path, req: GetGitLogRequest) -> String {
    let limit = req.limit.unwrap_or(20).min(200);
    let limit_str = format!("-{limit}");
    let fmt = "--format=%H%n%h%n%s%n%an%n%aI";
    let mut args = vec!["log", &limit_str, fmt, "--no-merges"];
    let path_owned: String;
    if let Some(ref path) = req.path {
        path_owned = path.clone();
        args.push("--");
        args.push(&path_owned);
    }
    let raw = match run_git(project_dir, &args) {
        Ok(s) => s,
        Err(e) => return format!("Error: {e}"),
    };
    let lines: Vec<&str> = raw.lines().collect();
    let mut entries = Vec::new();
    for chunk in lines.chunks(5) {
        if chunk.len() < 5 {
            break;
        }
        let files_changed = run_git(
            project_dir,
            &["diff-tree", "--no-commit-id", "--name-only", "-r", chunk[0]],
        )
        .map(|s| s.lines().filter(|l| !l.is_empty()).count() as u32)
        .unwrap_or(0);
        let date = chunk[4].split('T').next().unwrap_or(chunk[4]).to_string();
        entries.push(GitLogEntry {
            hash: chunk[0].to_string(),
            short_hash: chunk[1].to_string(),
            message: chunk[2].to_string(),
            author: chunk[3].to_string(),
            date,
            files_changed,
        });
    }
    serde_json::to_string(&entries).unwrap_or_else(|e| format!("Error: {e}"))
}

pub fn list_worktrees(project_dir: &Path) -> String {
    let raw = match run_git(project_dir, &["worktree", "list", "--porcelain"]) {
        Ok(s) => s,
        Err(e) => return format!("Error: {e}"),
    };
    let mut worktrees = Vec::new();
    let mut path = String::new();
    let mut head = String::new();
    let mut branch = String::new();

    for line in raw.lines() {
        if let Some(p) = line.strip_prefix("worktree ") {
            path = p.to_string();
        } else if let Some(h) = line.strip_prefix("HEAD ") {
            head = h.to_string();
        } else if let Some(b) = line.strip_prefix("branch ") {
            branch = b.strip_prefix("refs/heads/").unwrap_or(b).to_string();
        } else if line.is_empty() && !path.is_empty() {
            worktrees.push(Worktree {
                path: std::mem::take(&mut path),
                branch: std::mem::take(&mut branch),
                head: std::mem::take(&mut head).chars().take(7).collect(),
            });
        }
    }
    if !path.is_empty() {
        worktrees.push(Worktree {
            path,
            branch,
            head: head.chars().take(7).collect(),
        });
    }
    serde_json::to_string(&worktrees).unwrap_or_else(|e| format!("Error: {e}"))
}

#[cfg(test)]
#[path = "git_info_tests.rs"]
mod tests;
