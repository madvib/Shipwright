use crate::fs_util::write_atomic;
use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

/// Origin of a skill document.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Type, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SkillSource {
    #[default]
    Custom,
    Builtin,
    AiGenerated,
    Community,
    Imported,
}

/// A callable slash command / skill (→ `.claude/commands/<id>.md`).
/// Skills are the canonical instruction primitive in Ship.
/// They can be invoked explicitly by the user with `/skill-name [args]`
/// and can use `$ARGUMENTS`.
/// Stored as:
/// - project scope: `~/.ship/projects/<project-slug>/skills/<id>/SKILL.md`
/// - user scope: `~/.ship/skills/<id>/SKILL.md`
/// using Agent Skills spec format.
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct Skill {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    /// The command template. Use `$ARGUMENTS` as a placeholder for user input.
    pub content: String,
    #[serde(default)]
    pub source: SkillSource,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SkillSpecMetadata {
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    source: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SkillSpecFrontmatter {
    name: String,
    description: String,
    #[serde(default)]
    compatibility: Option<String>,
    #[serde(rename = "allowed-tools", default)]
    allowed_tools: Option<Vec<String>>,
    #[serde(default)]
    license: Option<String>,
    #[serde(default)]
    metadata: Option<SkillSpecMetadata>,
}

fn is_valid_skill_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 64 {
        return false;
    }
    if name.starts_with('-') || name.ends_with('-') || name.contains("--") {
        return false;
    }
    name.chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
}

fn validate_skill_spec_frontmatter(fm: &SkillSpecFrontmatter, path: &Path) -> Result<()> {
    if !is_valid_skill_name(&fm.name) {
        return Err(anyhow!(
            "Invalid SKILL.md in {}: name '{}' must match ^[a-z0-9]+(-[a-z0-9]+)*$ and be <= 64 chars",
            path.display(),
            fm.name
        ));
    }

    let desc_len = fm.description.trim().chars().count();
    if desc_len == 0 || desc_len > 1024 {
        return Err(anyhow!(
            "Invalid SKILL.md in {}: description must be 1..=1024 chars",
            path.display()
        ));
    }

    if let Some(compatibility) = &fm.compatibility
        && compatibility.chars().count() > 500
    {
        return Err(anyhow!(
            "Invalid SKILL.md in {}: compatibility must be <= 500 chars",
            path.display()
        ));
    }

    Ok(())
}

fn skills_dir(project_dir: &Path) -> PathBuf {
    crate::project::skills_dir(project_dir)
}

fn user_skills_dir() -> PathBuf {
    crate::project::user_skills_dir()
}

fn legacy_project_skills_dir(project_dir: &Path) -> PathBuf {
    project_dir.join("agents").join("skills")
}

fn legacy_user_skills_dir() -> Result<PathBuf> {
    Ok(crate::project::get_global_dir()?
        .join("agents")
        .join("skills"))
}

fn migrate_legacy_skill_dirs(legacy_dir: &Path, target_dir: &Path) -> Result<()> {
    if !legacy_dir.exists() || !legacy_dir.is_dir() {
        return Ok(());
    }

    fs::create_dir_all(target_dir)?;

    for entry in fs::read_dir(legacy_dir)? {
        let entry = entry?;
        let source = entry.path();
        if !source.is_dir() {
            continue;
        }
        let Some(name) = source.file_name() else {
            continue;
        };
        let target = target_dir.join(name);
        if target.exists() {
            continue;
        }
        if fs::rename(&source, &target).is_err() {
            fs::create_dir_all(&target)?;
            for child in fs::read_dir(&source)? {
                let child = child?;
                let child_source = child.path();
                let child_target = target.join(child.file_name());
                if child_source.is_dir() {
                    fs::create_dir_all(&child_target)?;
                } else {
                    fs::copy(&child_source, &child_target)?;
                }
            }
            let _ = fs::remove_dir_all(&source);
        }
    }

    if fs::read_dir(legacy_dir)?.next().is_none() {
        let _ = fs::remove_dir_all(legacy_dir);
    }

    Ok(())
}

fn ensure_project_skills_storage(project_dir: &Path) -> Result<PathBuf> {
    let target = skills_dir(project_dir);
    migrate_legacy_skill_dirs(&legacy_project_skills_dir(project_dir), &target)?;
    fs::create_dir_all(&target)?;
    Ok(target)
}

fn ensure_user_skills_storage() -> Result<PathBuf> {
    let target = user_skills_dir();
    if let Ok(legacy) = legacy_user_skills_dir() {
        migrate_legacy_skill_dirs(&legacy, &target)?;
    }
    fs::create_dir_all(&target)?;
    Ok(target)
}

fn skill_dir(project_dir: &Path, id: &str) -> PathBuf {
    skills_dir(project_dir).join(id)
}

fn user_skill_dir(id: &str) -> Result<PathBuf> {
    Ok(user_skills_dir().join(id))
}

fn parse_skill_spec(dir: &Path) -> Result<Skill> {
    let path = dir.join("SKILL.md");
    if !path.exists() {
        return Err(anyhow!("Missing SKILL.md in {}", dir.display()));
    }

    let raw = fs::read_to_string(&path)?;
    if !raw.starts_with("---\n") {
        return Err(anyhow!(
            "Invalid SKILL.md frontmatter in {}",
            path.display()
        ));
    }
    let rest = &raw[4..];
    let end = rest.find("\n---").ok_or_else(|| {
        anyhow!(
            "Invalid SKILL.md: missing closing frontmatter in {}",
            path.display()
        )
    })?;
    let yaml = &rest[..end];
    let body = rest[end + 4..].trim_start_matches('\n').to_string();

    let fm: SkillSpecFrontmatter = serde_yaml::from_str(yaml)
        .with_context(|| format!("Failed to parse SKILL.md frontmatter in {}", path.display()))?;
    validate_skill_spec_frontmatter(&fm, &path)?;

    let id = dir
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow!("Invalid skill directory name: {}", dir.display()))?
        .to_string();

    if fm.name != id {
        return Err(anyhow!(
            "Invalid SKILL.md in {}: frontmatter name '{}' must match directory id '{}'",
            path.display(),
            fm.name,
            id
        ));
    }

    let name = fm
        .metadata
        .as_ref()
        .and_then(|meta| meta.display_name.clone())
        .unwrap_or_else(|| id.clone());
    let source = fm
        .metadata
        .as_ref()
        .and_then(|meta| meta.source.as_deref())
        .map(|source| match source {
            "builtin" => SkillSource::Builtin,
            "ai-generated" => SkillSource::AiGenerated,
            "community" => SkillSource::Community,
            "imported" => SkillSource::Imported,
            _ => SkillSource::Custom,
        })
        .unwrap_or(SkillSource::Custom);

    Ok(Skill {
        id,
        name,
        description: Some(fm.description),
        version: None,
        author: None,
        content: body,
        source,
    })
}

fn write_skill_spec(dir: &Path, skill: &Skill) -> Result<()> {
    fs::create_dir_all(dir)?;
    let frontmatter = SkillSpecFrontmatter {
        name: skill.id.clone(),
        description: skill
            .description
            .clone()
            .unwrap_or_else(|| format!("Project skill '{}'", skill.id)),
        compatibility: None,
        allowed_tools: None,
        license: None,
        metadata: Some(SkillSpecMetadata {
            display_name: Some(skill.name.clone()),
            source: Some(match skill.source {
                SkillSource::Builtin => "builtin".to_string(),
                SkillSource::AiGenerated => "ai-generated".to_string(),
                SkillSource::Community => "community".to_string(),
                SkillSource::Imported => "imported".to_string(),
                SkillSource::Custom => "custom".to_string(),
            }),
        }),
    };
    let frontmatter_yaml = serde_yaml::to_string(&frontmatter)?;
    let content = format!("---\n{}---\n\n{}", frontmatter_yaml, skill.content.trim());
    let path = dir.join("SKILL.md");
    write_atomic(&path, content)?;
    Ok(())
}

fn parse_skill(dir: &Path) -> Result<Skill> {
    parse_skill_spec(dir)
}

fn write_skill(dir: &Path, skill: &Skill) -> Result<()> {
    write_skill_spec(dir, skill)?;
    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "kebab-case")]
pub enum SkillInstallScope {
    Project,
    User,
}

fn resolve_repo_source(source: &str) -> String {
    let trimmed = source.trim();
    if let Some(suffix) = trimmed.strip_prefix("~/")
        && let Some(home) = std::env::var_os("HOME")
    {
        return PathBuf::from(home)
            .join(suffix)
            .to_string_lossy()
            .to_string();
    }

    let source_path = Path::new(trimmed);
    if source_path.is_absolute()
        || source_path.exists()
        || trimmed.starts_with("./")
        || trimmed.starts_with("../")
    {
        return trimmed.to_string();
    }

    if trimmed.contains("://") || trimmed.starts_with("git@") || trimmed.ends_with(".git") {
        return trimmed.to_string();
    }

    let parts: Vec<&str> = trimmed.split('/').collect();
    if parts.len() == 2 && parts.iter().all(|part| !part.trim().is_empty()) {
        return format!("https://github.com/{}.git", trimmed);
    }

    trimmed.to_string()
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = dst.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir_recursive(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path)?;
        }
    }
    Ok(())
}

fn find_skill_source_dir(search_root: &Path, skill_id: &str) -> Result<PathBuf> {
    let mut candidates = Vec::new();
    let mut available = Vec::new();

    for entry in walkdir::WalkDir::new(search_root)
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if !entry.file_type().is_file() || entry.file_name() != "SKILL.md" {
            continue;
        }
        let Some(parent) = entry.path().parent() else {
            continue;
        };
        let Some(dir_name) = parent.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        available.push(dir_name.to_string());
        if dir_name == skill_id {
            candidates.push(parent.to_path_buf());
        }
    }

    if candidates.is_empty() {
        available.sort();
        available.dedup();
        let list = if available.is_empty() {
            "(no skills found)".to_string()
        } else {
            available.join(", ")
        };
        return Err(anyhow!(
            "Skill '{}' not found under {}. Available IDs: {}",
            skill_id,
            search_root.display(),
            list
        ));
    }

    candidates.sort();
    Ok(candidates.remove(0))
}

fn install_skill_from_source_into_dir(
    dest_root: &Path,
    source: &str,
    skill_id: &str,
    git_ref: &str,
    repo_path: &str,
    force: bool,
) -> Result<Skill> {
    let tmp_root = std::env::temp_dir().join(format!("ship-skill-install-{}", crate::gen_nanoid()));
    fs::create_dir_all(&tmp_root)?;
    struct CleanupGuard(PathBuf);
    impl Drop for CleanupGuard {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = CleanupGuard(tmp_root.clone());

    let cloned_repo = tmp_root.join("repo");
    let resolved_source = resolve_repo_source(source);
    let mut clone_cmd = ProcessCommand::new("git");
    clone_cmd
        .arg("clone")
        .arg("--depth")
        .arg("1")
        .arg("--branch")
        .arg(git_ref)
        .arg(&resolved_source)
        .arg(&cloned_repo);
    let clone_output = clone_cmd
        .output()
        .with_context(|| "Failed to run `git clone` while installing skill")?;

    if !clone_output.status.success() {
        return Err(anyhow!(
            "Failed to clone '{}' (ref '{}'): {}",
            resolved_source,
            git_ref,
            String::from_utf8_lossy(&clone_output.stderr).trim()
        ));
    }

    let search_root = cloned_repo.join(repo_path);
    if !search_root.exists() || !search_root.is_dir() {
        return Err(anyhow!(
            "repo-path '{}' does not exist in cloned repository",
            repo_path
        ));
    }

    let source_dir = find_skill_source_dir(&search_root, skill_id)?;
    let source_skill = parse_skill(&source_dir)?;

    fs::create_dir_all(dest_root)?;
    let destination_dir = dest_root.join(&source_skill.id);
    if destination_dir.exists() {
        if !force {
            return Err(anyhow!(
                "Skill '{}' already exists at {} (use --force to overwrite)",
                source_skill.id,
                destination_dir.display()
            ));
        }
        fs::remove_dir_all(&destination_dir)?;
    }

    copy_dir_recursive(&source_dir, &destination_dir)?;
    parse_skill(&destination_dir)
}

pub fn install_skill_from_source(
    project_dir: Option<&Path>,
    source: &str,
    skill_id: &str,
    git_ref: Option<&str>,
    repo_path: Option<&str>,
    scope: SkillInstallScope,
    force: bool,
) -> Result<Skill> {
    let git_ref = git_ref.unwrap_or("main");
    let repo_path = repo_path.unwrap_or(".");

    match scope {
        SkillInstallScope::User => {
            let dest_root = ensure_user_skills_storage()?;
            install_skill_from_source_into_dir(
                &dest_root, source, skill_id, git_ref, repo_path, force,
            )
        }
        SkillInstallScope::Project => {
            let project_dir =
                project_dir.ok_or_else(|| anyhow!("Project scope requires project_dir"))?;
            let dest_root = ensure_project_skills_storage(project_dir)?;
            let installed = install_skill_from_source_into_dir(
                &dest_root, source, skill_id, git_ref, repo_path, force,
            )?;

            let mut config = crate::config::get_config(Some(project_dir.to_path_buf()))?;
            if !config.agent.skills.contains(&installed.id) {
                config.agent.skills.push(installed.id.clone());
                crate::config::save_config(&config, Some(project_dir.to_path_buf()))?;
            }
            Ok(installed)
        }
    }
}

// ─── CRUD ─────────────────────────────────────────────────────────────────────

fn list_skills_from_dir(dir: &Path) -> Result<Vec<Skill>> {
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut skills = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            match parse_skill(&path) {
                Ok(s) => skills.push(s),
                Err(e) => eprintln!("warn: skipping {}: {}", path.display(), e),
            }
        }
    }
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}

pub fn list_skills(project_dir: &Path) -> Result<Vec<Skill>> {
    let dir = ensure_project_skills_storage(project_dir)?;
    list_skills_from_dir(&dir)
}

pub fn list_user_skills() -> Result<Vec<Skill>> {
    let dir = ensure_user_skills_storage()?;
    list_skills_from_dir(&dir)
}

/// Returns merged user + project skills keyed by id.
/// Project-scoped skills override user-scoped skills with the same id.
pub fn list_effective_skills(project_dir: &Path) -> Result<Vec<Skill>> {
    let mut by_id: HashMap<String, Skill> = HashMap::new();
    for skill in list_user_skills()? {
        by_id.insert(skill.id.clone(), skill);
    }
    for skill in list_skills(project_dir)? {
        by_id.insert(skill.id.clone(), skill);
    }
    let mut merged = by_id.into_values().collect::<Vec<_>>();
    merged.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(merged)
}

pub fn get_skill(project_dir: &Path, id: &str) -> Result<Skill> {
    let _ = ensure_project_skills_storage(project_dir)?;
    let dir = skill_dir(project_dir, id);
    if !dir.exists() {
        return Err(anyhow!("Skill '{}' not found", id));
    }
    parse_skill(&dir)
}

pub fn get_user_skill(id: &str) -> Result<Skill> {
    let _ = ensure_user_skills_storage()?;
    let dir = user_skill_dir(id)?;
    if !dir.exists() {
        return Err(anyhow!("Skill '{}' not found", id));
    }
    parse_skill(&dir)
}

/// Resolve a skill by checking project scope first, then user scope.
pub fn get_effective_skill(project_dir: &Path, id: &str) -> Result<Skill> {
    let _ = ensure_project_skills_storage(project_dir)?;
    let _ = ensure_user_skills_storage()?;
    let local_dir = skill_dir(project_dir, id);
    if local_dir.exists() {
        return parse_skill(&local_dir);
    }

    let global_dir = user_skill_dir(id)?;
    if global_dir.exists() {
        return parse_skill(&global_dir);
    }

    Err(anyhow!("Skill '{}' not found in project or user scope", id))
}

pub fn create_skill(project_dir: &Path, id: &str, name: &str, content: &str) -> Result<Skill> {
    let _ = ensure_project_skills_storage(project_dir)?;
    let dir = skill_dir(project_dir, id);
    if dir.exists() {
        return Err(anyhow!("Skill '{}' already exists", id));
    }
    let skill = Skill {
        id: id.to_string(),
        name: name.to_string(),
        description: None,
        version: None,
        author: None,
        content: content.to_string(),
        source: SkillSource::Custom,
    };
    write_skill(&dir, &skill)?;
    // Register in project config so checkout hook includes this skill automatically.
    let mut config = crate::config::get_config(Some(project_dir.to_path_buf()))?;
    if !config.agent.skills.contains(&id.to_string()) {
        config.agent.skills.push(id.to_string());
        crate::config::save_config(&config, Some(project_dir.to_path_buf()))?;
    }
    Ok(skill)
}

pub fn create_user_skill(id: &str, name: &str, content: &str) -> Result<Skill> {
    let _ = ensure_user_skills_storage()?;
    let dir = user_skill_dir(id)?;
    if dir.exists() {
        return Err(anyhow!("Skill '{}' already exists", id));
    }
    let skill = Skill {
        id: id.to_string(),
        name: name.to_string(),
        description: None,
        version: None,
        author: None,
        content: content.to_string(),
        source: SkillSource::Custom,
    };
    write_skill(&dir, &skill)?;
    Ok(skill)
}

pub fn update_skill(
    project_dir: &Path,
    id: &str,
    name: Option<&str>,
    content: Option<&str>,
) -> Result<Skill> {
    let _ = ensure_project_skills_storage(project_dir)?;
    let dir = skill_dir(project_dir, id);
    let mut skill = parse_skill(&dir)?;
    if let Some(n) = name {
        skill.name = n.to_string();
    }
    if let Some(c) = content {
        skill.content = c.to_string();
    }
    write_skill(&dir, &skill)?;
    Ok(skill)
}

pub fn update_user_skill(id: &str, name: Option<&str>, content: Option<&str>) -> Result<Skill> {
    let _ = ensure_user_skills_storage()?;
    let dir = user_skill_dir(id)?;
    if !dir.exists() {
        return Err(anyhow!("Skill '{}' not found", id));
    }
    let mut skill = parse_skill(&dir)?;
    if let Some(n) = name {
        skill.name = n.to_string();
    }
    if let Some(c) = content {
        skill.content = c.to_string();
    }
    write_skill(&dir, &skill)?;
    Ok(skill)
}

pub fn delete_skill(project_dir: &Path, id: &str) -> Result<()> {
    let _ = ensure_project_skills_storage(project_dir)?;
    let dir = skill_dir(project_dir, id);
    if !dir.exists() {
        return Err(anyhow!("Skill '{}' not found", id));
    }
    fs::remove_dir_all(dir)?;
    Ok(())
}

pub fn delete_user_skill(id: &str) -> Result<()> {
    let _ = ensure_user_skills_storage()?;
    let dir = user_skill_dir(id)?;
    if !dir.exists() {
        return Err(anyhow!("Skill '{}' not found", id));
    }
    fs::remove_dir_all(dir)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::tempdir;

    fn init_git_repo(path: &Path) -> Result<()> {
        let init = Command::new("git")
            .args(["init", "-b", "main"])
            .current_dir(path)
            .output()?;
        if !init.status.success() {
            return Err(anyhow!(
                "git init failed: {}",
                String::from_utf8_lossy(&init.stderr).trim()
            ));
        }

        let email = Command::new("git")
            .args(["config", "user.email", "test@ship.dev"])
            .current_dir(path)
            .output()?;
        if !email.status.success() {
            return Err(anyhow!(
                "git config user.email failed: {}",
                String::from_utf8_lossy(&email.stderr).trim()
            ));
        }
        let name = Command::new("git")
            .args(["config", "user.name", "Ship Test"])
            .current_dir(path)
            .output()?;
        if !name.status.success() {
            return Err(anyhow!(
                "git config user.name failed: {}",
                String::from_utf8_lossy(&name.stderr).trim()
            ));
        }
        Ok(())
    }

    fn commit_all(path: &Path, message: &str) -> Result<()> {
        let add = Command::new("git")
            .args(["add", "."])
            .current_dir(path)
            .output()?;
        if !add.status.success() {
            return Err(anyhow!(
                "git add failed: {}",
                String::from_utf8_lossy(&add.stderr).trim()
            ));
        }

        let commit = Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(path)
            .output()?;
        if !commit.status.success() {
            return Err(anyhow!(
                "git commit failed: {}",
                String::from_utf8_lossy(&commit.stderr).trim()
            ));
        }
        Ok(())
    }

    #[test]
    fn create_and_get_round_trip() -> Result<()> {
        let tmp = tempdir()?;
        let project_dir = tmp.path().join(".ship");
        fs::create_dir_all(&project_dir)?;
        let s = create_skill(
            &project_dir,
            "review",
            "Code Review",
            "Review this: $ARGUMENTS",
        )?;
        assert_eq!(s.id, "review");
        assert_eq!(s.source, SkillSource::Custom);
        let got = get_skill(&project_dir, "review")?;
        assert_eq!(got.content, "Review this: $ARGUMENTS");
        assert!(skill_dir(&project_dir, "review").is_dir());
        assert!(
            !skill_dir(&project_dir, "review")
                .join("skill.toml")
                .exists()
        );
        assert!(skill_dir(&project_dir, "review").join("SKILL.md").is_file());
        Ok(())
    }

    #[test]
    fn rejects_legacy_skill_format_without_skill_md() -> Result<()> {
        let tmp = tempdir()?;
        let legacy_dir = skill_dir(tmp.path(), "legacy-skill");
        fs::create_dir_all(&legacy_dir)?;
        write_atomic(
            &legacy_dir.join("skill.toml"),
            "id = \"legacy-skill\"\nname = \"Legacy Skill\"\n".to_string(),
        )?;
        write_atomic(&legacy_dir.join("index.md"), "legacy body".to_string())?;
        let err = get_skill(tmp.path(), "legacy-skill").expect_err("expected parse failure");
        assert!(err.to_string().contains("Missing SKILL.md"));
        Ok(())
    }

    #[test]
    fn migrates_legacy_repo_local_skill_storage_to_global_project_storage() -> Result<()> {
        let tmp = tempdir()?;
        let legacy_skill_dir = tmp
            .path()
            .join("agents")
            .join("skills")
            .join("legacy-migrated");
        fs::create_dir_all(&legacy_skill_dir)?;
        write_atomic(
            &legacy_skill_dir.join("SKILL.md"),
            r#"---
name: legacy-migrated
description: Legacy repo-local skill that should be migrated.
metadata:
  display_name: Legacy Migrated
---

Legacy body.
"#,
        )?;

        let skills = list_skills(tmp.path())?;
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].id, "legacy-migrated");
        assert_eq!(skills[0].name, "Legacy Migrated");
        assert!(
            !legacy_skill_dir.exists(),
            "legacy skill dir should be moved"
        );
        assert!(skill_dir(tmp.path(), "legacy-migrated").exists());
        Ok(())
    }

    #[test]
    fn parses_agentskills_spec_format() -> Result<()> {
        let tmp = tempdir()?;
        let dir = skill_dir(tmp.path(), "rust-runtime");
        fs::create_dir_all(&dir)?;
        write_atomic(
            &dir.join("SKILL.md"),
            r#"---
name: rust-runtime
description: Rust runtime and service layer implementation guidance.
metadata:
  display_name: Rust Runtime
  source: custom
---

Use this skill when changing runtime internals.
"#,
        )?;

        let skill = get_skill(tmp.path(), "rust-runtime")?;
        assert_eq!(skill.id, "rust-runtime");
        assert_eq!(skill.name, "Rust Runtime");
        assert_eq!(
            skill.description.as_deref(),
            Some("Rust runtime and service layer implementation guidance.")
        );
        assert!(skill.content.contains("runtime internals"));
        Ok(())
    }

    #[test]
    fn rejects_agentskills_name_directory_mismatch() -> Result<()> {
        let tmp = tempdir()?;
        let dir = skill_dir(tmp.path(), "expected-id");
        fs::create_dir_all(&dir)?;
        write_atomic(
            &dir.join("SKILL.md"),
            r#"---
name: wrong-id
description: Should fail because skill id does not match folder.
---

Bad skill.
"#,
        )?;

        let err = get_skill(tmp.path(), "expected-id").expect_err("expected parse failure");
        assert!(err.to_string().contains("must match directory id"));
        Ok(())
    }

    #[test]
    fn rejects_agentskills_invalid_name_format() -> Result<()> {
        let tmp = tempdir()?;
        let dir = skill_dir(tmp.path(), "Bad_Name");
        fs::create_dir_all(&dir)?;
        write_atomic(
            &dir.join("SKILL.md"),
            r#"---
name: Bad_Name
description: Invalid name format.
---

Bad.
"#,
        )?;

        let err = get_skill(tmp.path(), "Bad_Name").expect_err("expected parse failure");
        assert!(
            err.to_string()
                .contains("must match ^[a-z0-9]+(-[a-z0-9]+)*$")
        );
        Ok(())
    }

    #[test]
    fn rejects_agentskills_empty_description() -> Result<()> {
        let tmp = tempdir()?;
        let dir = skill_dir(tmp.path(), "empty-description");
        fs::create_dir_all(&dir)?;
        write_atomic(
            &dir.join("SKILL.md"),
            r#"---
name: empty-description
description: "   "
---

Body.
"#,
        )?;

        let err = get_skill(tmp.path(), "empty-description").expect_err("expected parse failure");
        assert!(
            err.to_string()
                .contains("description must be 1..=1024 chars")
        );
        Ok(())
    }

    #[test]
    fn list_returns_all_skills() -> Result<()> {
        let tmp = tempdir()?;
        let project_dir = tmp.path().join(".ship");
        fs::create_dir_all(&project_dir)?;
        create_skill(&project_dir, "a", "A", "content a")?;
        create_skill(&project_dir, "b", "B", "content b")?;
        assert_eq!(list_skills(&project_dir)?.len(), 2);
        Ok(())
    }

    #[test]
    fn list_empty_dir_returns_empty() -> Result<()> {
        let tmp = tempdir()?;
        assert!(list_skills(tmp.path())?.is_empty());
        Ok(())
    }

    #[test]
    fn update_skill_persists() -> Result<()> {
        let tmp = tempdir()?;
        let project_dir = tmp.path().join(".ship");
        fs::create_dir_all(&project_dir)?;
        create_skill(&project_dir, "s", "Old", "old")?;
        update_skill(&project_dir, "s", Some("New"), Some("new $ARGUMENTS"))?;
        let reloaded = get_skill(&project_dir, "s")?;
        assert_eq!(reloaded.name, "New");
        assert_eq!(reloaded.content, "new $ARGUMENTS");
        Ok(())
    }

    #[test]
    fn delete_removes_skill() -> Result<()> {
        let tmp = tempdir()?;
        let project_dir = tmp.path().join(".ship");
        fs::create_dir_all(&project_dir)?;
        create_skill(&project_dir, "gone", "Gone", "x")?;
        delete_skill(&project_dir, "gone")?;
        assert!(get_skill(&project_dir, "gone").is_err());
        Ok(())
    }

    #[test]
    fn duplicate_rejected() -> Result<()> {
        let tmp = tempdir()?;
        let project_dir = tmp.path().join(".ship");
        fs::create_dir_all(&project_dir)?;
        create_skill(&project_dir, "dup", "Dup", "x")?;
        assert!(create_skill(&project_dir, "dup", "Dup2", "y").is_err());
        Ok(())
    }

    #[test]
    fn install_skill_from_local_repo_into_user_scope() -> Result<()> {
        let repo = tempdir()?;
        init_git_repo(repo.path())?;

        let source_skill = repo
            .path()
            .join("skills")
            .join("vercel-react-best-practices");
        fs::create_dir_all(&source_skill)?;
        write_atomic(
            &source_skill.join("SKILL.md"),
            r#"---
name: vercel-react-best-practices
description: React guidance from Vercel best practices.
metadata:
  display_name: Vercel React Best Practices
  source: imported
---

Prefer composition and server-first rendering defaults.
"#,
        )?;
        commit_all(repo.path(), "seed skill")?;

        let dest = tempdir()?;
        let installed = install_skill_from_source_into_dir(
            dest.path(),
            repo.path()
                .to_str()
                .ok_or_else(|| anyhow!("invalid repo path"))?,
            "vercel-react-best-practices",
            "main",
            "skills",
            false,
        )?;
        assert_eq!(installed.id, "vercel-react-best-practices");
        assert_eq!(installed.name, "Vercel React Best Practices");
        assert!(
            dest.path()
                .join("vercel-react-best-practices")
                .join("SKILL.md")
                .is_file()
        );
        Ok(())
    }

    #[test]
    fn install_skill_requires_force_when_destination_exists() -> Result<()> {
        let repo = tempdir()?;
        init_git_repo(repo.path())?;

        let source_skill = repo.path().join("skills").join("skill-creator");
        fs::create_dir_all(&source_skill)?;
        write_atomic(
            &source_skill.join("SKILL.md"),
            r#"---
name: skill-creator
description: Create robust Agent Skills.
metadata:
  display_name: Skill Creator
  source: imported
---

Create and validate skills.
"#,
        )?;
        commit_all(repo.path(), "seed skill")?;

        let dest = tempdir()?;
        install_skill_from_source_into_dir(
            dest.path(),
            repo.path()
                .to_str()
                .ok_or_else(|| anyhow!("invalid repo path"))?,
            "skill-creator",
            "main",
            "skills",
            false,
        )?;

        let err = install_skill_from_source_into_dir(
            dest.path(),
            repo.path()
                .to_str()
                .ok_or_else(|| anyhow!("invalid repo path"))?,
            "skill-creator",
            "main",
            "skills",
            false,
        )
        .expect_err("install should fail without force");
        assert!(err.to_string().contains("already exists"));
        Ok(())
    }
}
