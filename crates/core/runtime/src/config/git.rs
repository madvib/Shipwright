use super::io::{get_config, save_config};
use super::types::GitConfig;
use crate::{EventAction, EventEntity, append_event};
use anyhow::Result;
use std::path::Path;

pub fn get_git_config(project_dir: &Path) -> Result<GitConfig> {
    let config = get_config(Some(project_dir.to_path_buf()))?;
    Ok(config.git)
}

pub fn set_git_config(project_dir: &Path, git: GitConfig) -> Result<()> {
    let mut config = get_config(Some(project_dir.to_path_buf()))?;
    config.git = git;
    generate_gitignore(project_dir, &config.git)?;
    save_config(&config, Some(project_dir.to_path_buf()))?;
    Ok(())
}

/// Toggle a named category in/out of git commit tracking.
pub fn set_category_committed(project_dir: &Path, category: &str, commit: bool) -> Result<()> {
    let mut git = get_git_config(project_dir)?;
    let categories: Vec<&str> = if category == "agents" {
        vec!["mcp", "permissions", "rules"]
    } else {
        vec![category]
    };
    for category in categories {
        if commit {
            if !git.commit.contains(&category.to_string()) {
                git.commit.push(category.to_string());
            }
            git.ignore.retain(|i| i != category);
        } else {
            git.commit.retain(|c| c != category);
            if !git.ignore.contains(&category.to_string()) {
                git.ignore.push(category.to_string());
            }
        }
    }
    set_git_config(project_dir, git)?;
    append_event(
        project_dir,
        "logic",
        EventEntity::Config,
        if commit {
            EventAction::Set
        } else {
            EventAction::Clear
        },
        "git_category",
        Some(format!("category={}", category)),
    )?;
    Ok(())
}

pub fn is_category_committed(git: &GitConfig, category: &str) -> bool {
    if category == "agents" {
        return ["mcp", "permissions", "rules"]
            .iter()
            .all(|entry| git.commit.contains(&entry.to_string()));
    }
    git.commit.contains(&category.to_string())
}

/// Write `.ship/.gitignore`. Everything not in `git.commit` is ignored by default.
/// Keys use namespace paths (e.g. "project/specs", "project/adrs").
pub fn generate_gitignore(ship_dir: &Path, git: &GitConfig) -> Result<()> {
    // (key, namespace path) — key is what appears in git.commit config
    let known: &[(&str, &str)] = &[
        ("specs", "project/specs"),
        ("features", "project/features"),
        ("releases", "project/releases"),
        ("adrs", "project/adrs"),
        ("notes", "project/notes"),
        ("vision", "vision.md"),
        ("mcp", "mcp.jsonc"),
        ("permissions", "permissions.jsonc"),
        ("rules", "rules"),
        ("skills", "skills"),
        ("ship-readme", "README.md"),
        ("project-readme", "project/README.md"),
        ("ship.jsonc", "ship.jsonc"),
        ("templates", "**/TEMPLATE.md"),
    ];
    let mut lines = vec![
        "# Managed by Ship — edit via `ship git include/exclude`".to_string(),
        String::new(),
    ];
    for (key, path) in known {
        if !git.commit.contains(&key.to_string()) {
            lines.push(path.to_string());
        }
    }
    if !lines.iter().any(|line| line == ".tmp-global/") {
        lines.push(".tmp-global/".to_string());
    }
    let content = lines.join("\n") + "\n";
    crate::fs_util::write_atomic(&ship_dir.join(".gitignore"), content)?;
    Ok(())
}
