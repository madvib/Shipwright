use anyhow::Result;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::fs;
use std::path::PathBuf;

// ─── Data types ───────────────────────────────────────────────────────────────

/// A rule file from `agents/rules/*.md`. Rules are always active for the agent.
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct Rule {
    pub file_name: String,
    pub path: String,
    pub content: String,
}

// ─── Paths ────────────────────────────────────────────────────────────────────

fn rules_dir(ship_dir: &std::path::Path) -> PathBuf {
    ship_dir.join("agents").join("rules")
}

// ─── CRUD ─────────────────────────────────────────────────────────────────────

pub fn list_rules(ship_dir: PathBuf) -> Result<Vec<Rule>> {
    let dir = rules_dir(&ship_dir);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut rules = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |e| e == "md") {
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            if file_name == "README.md" {
                continue;
            }
            let content = fs::read_to_string(&path).unwrap_or_default();
            rules.push(Rule {
                file_name,
                path: path.to_string_lossy().to_string(),
                content,
            });
        }
    }
    rules.sort_by(|a, b| a.file_name.cmp(&b.file_name));
    Ok(rules)
}

pub fn get_rule(ship_dir: PathBuf, file_name: &str) -> Result<Rule> {
    let path = rules_dir(&ship_dir).join(file_name);
    let content = fs::read_to_string(&path)?;
    Ok(Rule {
        file_name: file_name.to_string(),
        path: path.to_string_lossy().to_string(),
        content,
    })
}

pub fn create_rule(ship_dir: PathBuf, file_name: &str, content: &str) -> Result<Rule> {
    let dir = rules_dir(&ship_dir);
    fs::create_dir_all(&dir)?;
    let path = dir.join(file_name);
    crate::fs_util::write_atomic(&path, content)?;
    Ok(Rule {
        file_name: file_name.to_string(),
        path: path.to_string_lossy().to_string(),
        content: content.to_string(),
    })
}

pub fn update_rule(ship_dir: PathBuf, file_name: &str, content: &str) -> Result<Rule> {
    let path = rules_dir(&ship_dir).join(file_name);
    crate::fs_util::write_atomic(&path, content)?;
    Ok(Rule {
        file_name: file_name.to_string(),
        path: path.to_string_lossy().to_string(),
        content: content.to_string(),
    })
}

pub fn delete_rule(ship_dir: PathBuf, file_name: &str) -> Result<()> {
    let path = rules_dir(&ship_dir).join(file_name);
    fs::remove_file(path)?;
    Ok(())
}
