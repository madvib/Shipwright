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

// ── HTTP + base64 (no runtime deps) ──────────────────────────────────────────

fn fetch_url(url: &str) -> anyhow::Result<String> {
    let output = std::process::Command::new("curl")
        .args(["-fsSL", "--user-agent", "ship-cli/0.1", url])
        .output()?;
    if !output.status.success() {
        anyhow::bail!("fetch failed for {}: {}", url, String::from_utf8_lossy(&output.stderr));
    }
    Ok(String::from_utf8(output.stdout)?)
}

fn decode_base64(encoded: &str) -> anyhow::Result<String> {
    let clean: String = encoded.chars().filter(|c| !c.is_whitespace()).collect();
    let output = std::process::Command::new("base64")
        .arg("--decode")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            child.stdin.take().unwrap().write_all(clean.as_bytes())?;
            child.wait_with_output()
        })?;
    Ok(String::from_utf8(output.stdout)?)
}

// ── GitHub API ────────────────────────────────────────────────────────────────

fn fetch_github_file(owner: &str, repo: &str, path: &str) -> anyhow::Result<Option<String>> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/contents/{path}");
    let raw = match fetch_url(&url) {
        Ok(v) => v,
        Err(_) => return Ok(None), // 404 or network error — treat as not found
    };
    let v: serde_json::Value = serde_json::from_str(&raw)?;
    if v["type"].as_str() == Some("file") {
        let encoded = v["content"].as_str()
            .ok_or_else(|| anyhow::anyhow!("GitHub API: missing content field"))?;
        Ok(Some(decode_base64(encoded)?))
    } else {
        Ok(None)
    }
}

fn list_github_dir(owner: &str, repo: &str, path: &str) -> anyhow::Result<Option<Vec<(String, String)>>> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/contents/{path}");
    let raw = match fetch_url(&url) {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };
    let arr: serde_json::Value = serde_json::from_str(&raw)?;
    Ok(arr.as_array().map(|entries| {
        entries.iter().filter_map(|e| {
            Some((e["name"].as_str()?.to_string(), e["type"].as_str()?.to_string()))
        }).collect()
    }))
}

fn resolve_github_skills(owner: &str, repo: &str, skill_hint: Option<&str>) -> anyhow::Result<Vec<(String, String)>> {
    if let Some(id) = skill_hint {
        for path in &[
            format!("skills/{id}/SKILL.md"),
            format!("skills/{id}.md"),
            format!("{id}/SKILL.md"),
            "SKILL.md".to_string(),
        ] {
            if let Some(content) = fetch_github_file(owner, repo, path)? {
                return Ok(vec![(id.to_string(), content)]);
            }
        }
        anyhow::bail!("skill '{id}' not found in {owner}/{repo}");
    }

    if let Some(entries) = list_github_dir(owner, repo, "skills")? {
        let dirs: Vec<_> = entries.iter().filter(|(_, t)| t == "dir").map(|(n, _)| n.clone()).collect();
        if !dirs.is_empty() {
            let mut result = Vec::new();
            for name in dirs {
                if let Some(content) = fetch_github_file(owner, repo, &format!("skills/{name}/SKILL.md"))? {
                    result.push((name, content));
                }
            }
            if !result.is_empty() { return Ok(result); }
        }
    }

    if let Some(content) = fetch_github_file(owner, repo, "SKILL.md")? {
        return Ok(vec![(repo.to_string(), content)]);
    }

    // Fallback: root-level dirs each containing SKILL.md (e.g. better-auth/skills layout)
    if let Some(entries) = list_github_dir(owner, repo, "")? {
        let dirs: Vec<_> = entries.iter().filter(|(_, t)| t == "dir")
            .map(|(n, _)| n.clone())
            .filter(|n| !n.starts_with('.'))
            .collect();
        if !dirs.is_empty() {
            let mut result = Vec::new();
            for dir in &dirs {
                // Check direct SKILL.md and nested subdirs one level deep
                if let Some(content) = fetch_github_file(owner, repo, &format!("{dir}/SKILL.md"))? {
                    result.push((dir.clone(), content));
                } else if let Some(sub_entries) = list_github_dir(owner, repo, dir)? {
                    for (name, kind) in sub_entries {
                        if kind == "dir"
                            && let Some(content) = fetch_github_file(owner, repo, &format!("{dir}/{name}/SKILL.md"))?
                        {
                            result.push((format!("{dir}-{name}"), content));
                        }
                    }
                }
            }
            if !result.is_empty() { return Ok(result); }
        }
    }

    anyhow::bail!("no skill content found in {owner}/{repo}")
}

// ── Public add entry point ────────────────────────────────────────────────────

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
        SkillSource::GitHub { owner, repo } => {
            eprintln!("warning: installing skills from external sources — review SKILL.md before use");
            let skills = resolve_github_skills(&owner, &repo, skill_id)?;
            let base = if global { global_skills_dir() } else { agents_skills_dir() };
            for (id, content) in skills {
                let dir = base.join(&id);
                std::fs::create_dir_all(&dir)?;
                std::fs::write(dir.join("SKILL.md"), content)?;
                println!("✓ installed skill '{}' to {}", id, dir.display());
                println!("  review before use: {}/SKILL.md", dir.display());
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
