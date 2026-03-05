use anyhow::{Result, anyhow};
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

fn normalize_rule_file_name(file_name: &str) -> Result<String> {
    let trimmed = file_name.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("Rule file name cannot be empty"));
    }

    let path = std::path::Path::new(trimmed);
    let is_single_component = path
        .components()
        .all(|component| matches!(component, std::path::Component::Normal(_)));
    if !is_single_component {
        return Err(anyhow!(
            "Rule file name must be a single file name without directories"
        ));
    }

    if trimmed.contains('\\') {
        return Err(anyhow!("Rule file name cannot contain path separators"));
    }

    let normalized = if trimmed.to_ascii_lowercase().ends_with(".md") {
        trimmed.to_string()
    } else if !trimmed.contains('.') {
        format!("{}.md", trimmed)
    } else {
        return Err(anyhow!(
            "Rule file name must end in .md (or omit extension)"
        ));
    };

    if normalized.eq_ignore_ascii_case("README.md") {
        return Err(anyhow!(
            "README.md is reserved and cannot be managed as a rule"
        ));
    }

    Ok(normalized)
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
        if path.is_file() && path.extension().is_some_and(|e| e == "md") {
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
    let file_name = normalize_rule_file_name(file_name)?;
    let path = rules_dir(&ship_dir).join(&file_name);
    let content = fs::read_to_string(&path)?;
    Ok(Rule {
        file_name,
        path: path.to_string_lossy().to_string(),
        content,
    })
}

pub fn create_rule(ship_dir: PathBuf, file_name: &str, content: &str) -> Result<Rule> {
    let file_name = normalize_rule_file_name(file_name)?;
    let dir = rules_dir(&ship_dir);
    fs::create_dir_all(&dir)?;
    let path = dir.join(&file_name);
    crate::fs_util::write_atomic(&path, content)?;
    Ok(Rule {
        file_name,
        path: path.to_string_lossy().to_string(),
        content: content.to_string(),
    })
}

pub fn update_rule(ship_dir: PathBuf, file_name: &str, content: &str) -> Result<Rule> {
    let file_name = normalize_rule_file_name(file_name)?;
    let path = rules_dir(&ship_dir).join(&file_name);
    crate::fs_util::write_atomic(&path, content)?;
    Ok(Rule {
        file_name,
        path: path.to_string_lossy().to_string(),
        content: content.to_string(),
    })
}

pub fn delete_rule(ship_dir: PathBuf, file_name: &str) -> Result<()> {
    let file_name = normalize_rule_file_name(file_name)?;
    let path = rules_dir(&ship_dir).join(file_name);
    fs::remove_file(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::init_project;
    use tempfile::tempdir;

    #[test]
    fn create_get_update_delete_rule_round_trip() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = init_project(tmp.path().to_path_buf())?;

        let created = create_rule(ship_dir.clone(), "runtime-hardening", "# Initial\n")?;
        assert_eq!(created.file_name, "runtime-hardening.md");

        let fetched = get_rule(ship_dir.clone(), "runtime-hardening")?;
        assert_eq!(fetched.file_name, "runtime-hardening.md");
        assert_eq!(fetched.content, "# Initial\n");

        let updated = update_rule(ship_dir.clone(), "runtime-hardening.md", "# Updated\n")?;
        assert_eq!(updated.file_name, "runtime-hardening.md");
        assert_eq!(updated.content, "# Updated\n");

        delete_rule(ship_dir.clone(), "runtime-hardening")?;
        assert!(
            get_rule(ship_dir, "runtime-hardening").is_err(),
            "rule should not exist after delete"
        );
        Ok(())
    }

    #[test]
    fn rule_crud_rejects_invalid_file_names() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = init_project(tmp.path().to_path_buf())?;

        for invalid in ["", "README.md", "../escape.md", "nested/rule.md", "bad.txt"] {
            assert!(
                create_rule(ship_dir.clone(), invalid, "# Rule\n").is_err(),
                "create_rule should reject '{}'",
                invalid
            );
            assert!(
                get_rule(ship_dir.clone(), invalid).is_err(),
                "get_rule should reject '{}'",
                invalid
            );
            assert!(
                update_rule(ship_dir.clone(), invalid, "# Rule\n").is_err(),
                "update_rule should reject '{}'",
                invalid
            );
            assert!(
                delete_rule(ship_dir.clone(), invalid).is_err(),
                "delete_rule should reject '{}'",
                invalid
            );
        }

        Ok(())
    }
}
