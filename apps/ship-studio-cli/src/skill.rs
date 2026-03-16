//! Skill management: list + create local project skills.

use anyhow::Result;
use std::path::Path;

use crate::paths::{agents_skills_dir, global_skills_dir};

// ── Source parsing ────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
pub enum SkillSource {
    /// GitHub owner + repo
    GitHub { owner: String, repo: String },
    /// skill-id@registry-name
    Registry { id: String, registry: String },
}

/// Parse a source string into a `SkillSource`.
pub fn parse_source(source: &str) -> anyhow::Result<SkillSource> {
    // Full GitHub URL: https://github.com/owner/repo
    if source.starts_with("https://github.com/") {
        let rest = source.trim_start_matches("https://github.com/").trim_end_matches('/');
        let parts: Vec<&str> = rest.splitn(2, '/').collect();
        if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
            return Ok(SkillSource::GitHub {
                owner: parts[0].to_string(),
                repo: parts[1].to_string(),
            });
        }
        anyhow::bail!("Invalid GitHub URL '{}'. Expected https://github.com/owner/repo", source);
    }

    // registry format: skill-id@registry-name
    if let Some(at) = source.find('@') {
        let id = &source[..at];
        let registry = &source[at + 1..];
        if !id.is_empty() && !registry.is_empty() {
            return Ok(SkillSource::Registry {
                id: id.to_string(),
                registry: registry.to_string(),
            });
        }
    }

    // Shorthand: owner/repo (no https://, no @)
    if source.contains('/') {
        let parts: Vec<&str> = source.splitn(2, '/').collect();
        if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
            return Ok(SkillSource::GitHub {
                owner: parts[0].to_string(),
                repo: parts[1].to_string(),
            });
        }
    }

    anyhow::bail!(
        "Cannot parse source '{}'. Use: https://github.com/owner/repo, owner/repo, or skill-id@registry",
        source
    )
}

// ── Public add entry point ────────────────────────────────────────────────────

/// Install a skill by delegating to `npx skills add`.
/// For registry sources (skill-id@registry), delegates to `claude plugin install`.
pub fn add(source: &str, skill_id: Option<&str>, global: bool) -> Result<()> {
    let parsed = parse_source(source)?;

    match parsed {
        SkillSource::Registry { id, registry } => {
            println!("Installing plugin: claude plugin install {}@{}", id, registry);
            let status = std::process::Command::new("claude")
                .args(["plugin", "install", &format!("{}@{}", id, registry)])
                .status()?;
            if !status.success() {
                anyhow::bail!("claude plugin install failed");
            }
        }
        SkillSource::GitHub { .. } => {
            // Delegate to the `skills` package manager (npx skills add).
            // See https://skills.sh for the open Agent Skills standard.
            let mut cmd = std::process::Command::new("npx");
            cmd.args(["skills", "add", source, "--yes"]);
            if let Some(id) = skill_id {
                cmd.args(["--skill", id]);
            }
            if global {
                cmd.arg("--global");
            }
            let status = cmd.status()?;
            if !status.success() {
                anyhow::bail!("npx skills add failed");
            }
        }
    }

    Ok(())
}

pub fn list() -> Result<()> {
    let mut found = false;

    let project_dir = agents_skills_dir();
    if project_dir.exists() {
        let skills = collect_skill_ids(&project_dir);
        if !skills.is_empty() {
            println!("Project skills (.ship/agents/skills/):");
            for id in &skills { println!("  - {}", id); }
            found = true;
        }
    }

    let global_dir = global_skills_dir();
    if global_dir.exists() {
        let skills = collect_skill_ids(&global_dir);
        if !skills.is_empty() {
            println!("Global skills (~/.ship/skills/):");
            for id in &skills { println!("  - {}", id); }
            found = true;
        }
    }

    if !found {
        println!("No skills installed.");
        println!("Create one with: ship skill create <id>");
    }
    Ok(())
}

pub fn create(id: &str, name: Option<&str>, description: Option<&str>) -> Result<()> {
    if !is_valid_id(id) {
        anyhow::bail!("Invalid skill ID '{}'. Use lowercase letters, digits, and hyphens.", id);
    }
    let skill_dir = agents_skills_dir().join(id);
    if skill_dir.exists() {
        anyhow::bail!("Skill '{}' already exists at {}", id, skill_dir.display());
    }
    std::fs::create_dir_all(&skill_dir)?;

    let name = name.unwrap_or(id);
    let description = description.unwrap_or("Describe when this skill should activate.");
    let content = format!(
"---
name: {name}
description: {description}
---

## Instructions

<!-- Add instructions for the agent here -->
");
    std::fs::write(skill_dir.join("SKILL.md"), content)?;
    println!("✓ created skill '{}' at {}", id, skill_dir.display());
    println!("  Edit: {}/SKILL.md", skill_dir.display());
    Ok(())
}

pub fn remove(id: &str, global: bool) -> Result<()> {
    let dir = if global { global_skills_dir().join(id) } else { agents_skills_dir().join(id) };
    if !dir.exists() {
        anyhow::bail!("Skill '{}' not found at {}", id, dir.display());
    }
    std::fs::remove_dir_all(&dir)?;
    println!("✓ removed skill '{}'", id);
    Ok(())
}

fn collect_skill_ids(dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(dir) else { return vec![]; };
    let mut ids: Vec<String> = entries
        .flatten()
        .filter(|e| e.path().is_dir() && e.path().join("SKILL.md").exists())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    ids.sort();
    ids
}

fn is_valid_id(id: &str) -> bool {
    !id.is_empty() && id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_skill_ids() {
        assert!(is_valid_id("rust-expert"));
        assert!(is_valid_id("test-driven"));
        assert!(is_valid_id("skill123"));
    }

    #[test]
    fn invalid_skill_ids() {
        assert!(!is_valid_id(""));
        assert!(!is_valid_id("Rust Expert"));
        assert!(!is_valid_id("rust_expert"));
        assert!(!is_valid_id("rust/expert"));
    }

    #[test]
    fn collect_skill_ids_from_dir() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        for id in &["beta-skill", "alpha-skill"] {
            let skill_dir = dir.join(id);
            std::fs::create_dir_all(&skill_dir).unwrap();
            std::fs::write(skill_dir.join("SKILL.md"), "content").unwrap();
        }
        // Dir without SKILL.md should be ignored
        std::fs::create_dir_all(dir.join("no-skill-md")).unwrap();

        let ids = collect_skill_ids(dir);
        assert_eq!(ids, vec!["alpha-skill", "beta-skill"]);
    }

    // ── parse_source tests ────────────────────────────────────────────────────

    #[test]
    fn parse_full_github_url() {
        let s = parse_source("https://github.com/rivet-dev/skills").unwrap();
        assert_eq!(
            s,
            SkillSource::GitHub {
                owner: "rivet-dev".into(),
                repo: "skills".into()
            }
        );
    }

    #[test]
    fn parse_github_url_with_trailing_slash() {
        let s = parse_source("https://github.com/cloudflare/skills/").unwrap();
        assert_eq!(
            s,
            SkillSource::GitHub {
                owner: "cloudflare".into(),
                repo: "skills".into()
            }
        );
    }

    #[test]
    fn parse_github_shorthand() {
        let s = parse_source("org/repo").unwrap();
        assert_eq!(
            s,
            SkillSource::GitHub {
                owner: "org".into(),
                repo: "repo".into()
            }
        );
    }

    #[test]
    fn parse_registry_format() {
        let s = parse_source("my-skill@ship").unwrap();
        assert_eq!(
            s,
            SkillSource::Registry {
                id: "my-skill".into(),
                registry: "ship".into()
            }
        );
    }

    #[test]
    fn parse_invalid_source_returns_error() {
        assert!(parse_source("notvalid").is_err());
        assert!(parse_source("https://github.com/only-owner/").is_err());
        assert!(parse_source("").is_err());
    }

}
