use crate::config::{LEGACY_CONFIG_FILE, PRIMARY_CONFIG_FILE, SECONDARY_CONFIG_FILE};
use crate::project::{
    AppState, ProjectRegistry, adrs_dir, features_dir, generated_ns, issues_dir, notes_dir,
    project_ns, releases_dir, specs_dir, workflow_ns,
};
use crate::state_db::{DatabaseMigrationReport, ensure_global_database, ensure_project_database};
use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectFileMigrationReport {
    pub copied_files: usize,
    pub skipped_identical_files: usize,
    pub conflict_files: usize,
    pub copied_directories: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectStateMigrationReport {
    pub files: ProjectFileMigrationReport,
    pub db: DatabaseMigrationReport,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GlobalStateMigrationReport {
    pub registry_entries_before: usize,
    pub registry_entries_after: usize,
    pub normalized_paths: usize,
    pub app_state_paths_normalized: usize,
    pub db: DatabaseMigrationReport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FileCopyOutcome {
    Copied,
    SkippedIdentical,
    ConflictCopied,
}

pub fn migrate_project_state(ship_dir: &Path) -> Result<ProjectStateMigrationReport> {
    let files = migrate_project_files(ship_dir)?;
    let db = ensure_project_database(ship_dir)?;
    Ok(ProjectStateMigrationReport { files, db })
}

pub fn migrate_global_state(global_dir: &Path) -> Result<GlobalStateMigrationReport> {
    fs::create_dir_all(global_dir)?;

    let registry_path = global_dir.join("projects.json");
    let mut registry: ProjectRegistry = if registry_path.exists() {
        serde_json::from_str(&fs::read_to_string(&registry_path)?)?
    } else {
        ProjectRegistry {
            projects: Vec::new(),
        }
    };

    let before = registry.projects.len();
    let mut normalized_paths = 0usize;
    let mut dedupe = HashSet::new();
    let mut projects = Vec::new();
    for mut project in registry.projects {
        let normalized = normalize_project_path(&project.path);
        if normalized != project.path {
            normalized_paths += 1;
            project.path = normalized;
        }
        let dedupe_key = project.path.to_string_lossy().to_string();
        if dedupe.insert(dedupe_key) {
            projects.push(project);
        }
    }
    registry.projects = projects;
    let after = registry.projects.len();
    if before != after || normalized_paths > 0 {
        fs::write(&registry_path, serde_json::to_string_pretty(&registry)?)?;
    }

    let app_state_path = global_dir.join("app_state.json");
    let mut app_state_paths_normalized = 0usize;
    if app_state_path.exists() {
        let mut app_state: AppState = serde_json::from_str(&fs::read_to_string(&app_state_path)?)?;
        if let Some(path) = app_state.active_project.clone() {
            let normalized = normalize_project_path(&path);
            if normalized != path {
                app_state.active_project = Some(normalized);
                app_state_paths_normalized += 1;
            }
        }
        let mut normalized_recent = Vec::with_capacity(app_state.recent_projects.len());
        for path in app_state.recent_projects {
            let normalized = normalize_project_path(&path);
            if normalized != path {
                app_state_paths_normalized += 1;
            }
            if !normalized_recent.contains(&normalized) {
                normalized_recent.push(normalized);
            }
        }
        app_state.recent_projects = normalized_recent;
        if app_state_paths_normalized > 0 {
            fs::write(&app_state_path, serde_json::to_string_pretty(&app_state)?)?;
        }
    }

    let db = ensure_global_database(global_dir)?;
    Ok(GlobalStateMigrationReport {
        registry_entries_before: before,
        registry_entries_after: after,
        normalized_paths,
        app_state_paths_normalized,
        db,
    })
}

fn migrate_project_files(ship_dir: &Path) -> Result<ProjectFileMigrationReport> {
    let mut report = ProjectFileMigrationReport::default();

    // Ensure namespace roots exist before we begin moving/copying.
    fs::create_dir_all(project_ns(ship_dir))?;
    fs::create_dir_all(workflow_ns(ship_dir))?;
    fs::create_dir_all(generated_ns(ship_dir))?;
    fs::create_dir_all(notes_dir(ship_dir))?;
    migrate_project_config_file(ship_dir)?;
    migrate_workflow_event_stream(ship_dir)?;
    migrate_event_index_location(ship_dir)?;
    migrate_template_layout(ship_dir, &mut report)?;

    let mappings = [
        (ship_dir.join("issues"), issues_dir(ship_dir)),
        (ship_dir.join("specs"), specs_dir(ship_dir)),
        (ship_dir.join("features"), features_dir(ship_dir)),
        (ship_dir.join("adrs"), adrs_dir(ship_dir)),
        (ship_dir.join("notes"), notes_dir(ship_dir)),
        (
            ship_dir.join("workflow").join("releases"),
            releases_dir(ship_dir),
        ),
        (ship_dir.join("releases"), releases_dir(ship_dir)),
    ];
    for (legacy, modern) in mappings {
        migrate_directory_tree(&legacy, &modern, &mut report)?;
    }

    let modern_vision = ship_dir.join("project").join("vision.md");
    if !modern_vision.exists() {
        let legacy_vision_candidates = [
            ship_dir.join("specs").join("vision.md"),
            ship_dir.join("workflow").join("specs").join("vision.md"),
        ];
        if let Some(legacy_vision) = legacy_vision_candidates.iter().find(|path| path.exists()) {
            if let Some(parent) = modern_vision.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(legacy_vision, &modern_vision)?;
            report.copied_files += 1;
        }
    }

    let legacy_log = ship_dir.join("log.md");
    if legacy_log.exists() {
        fs::remove_file(legacy_log)?;
    }

    let legacy_plugins = ship_dir.join("plugins");
    if legacy_plugins.is_dir() && fs::read_dir(&legacy_plugins)?.next().is_none() {
        fs::remove_dir(&legacy_plugins)?;
    }

    Ok(report)
}

fn migrate_project_config_file(ship_dir: &Path) -> Result<()> {
    let primary = ship_dir.join(PRIMARY_CONFIG_FILE);
    if !primary.exists() {
        for legacy_name in [SECONDARY_CONFIG_FILE, LEGACY_CONFIG_FILE] {
            let legacy = ship_dir.join(legacy_name);
            if legacy.exists() {
                move_file(&legacy, &primary)?;
                break;
            }
        }
    }

    if primary.exists() {
        for legacy_name in [SECONDARY_CONFIG_FILE, LEGACY_CONFIG_FILE] {
            let legacy = ship_dir.join(legacy_name);
            if legacy.exists() {
                fs::remove_file(legacy)?;
            }
        }
    }
    Ok(())
}

fn migrate_workflow_event_stream(ship_dir: &Path) -> Result<()> {
    crate::events::ensure_event_log(ship_dir)
}

fn migrate_event_index_location(ship_dir: &Path) -> Result<()> {
    let legacy = ship_dir.join("workflow").join("event_index.json");
    if !legacy.exists() {
        return Ok(());
    }
    let target = generated_ns(ship_dir).join("event_index.json");
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }

    if !target.exists() {
        move_file(&legacy, &target)?;
        return Ok(());
    }

    let legacy_content = fs::read(&legacy)?;
    let target_content = fs::read(&target)?;
    if legacy_content != target_content {
        let conflict = conflict_path_for(&target);
        fs::write(conflict, legacy_content)?;
    }
    fs::remove_file(legacy)?;
    Ok(())
}

fn migrate_template_layout(ship_dir: &Path, report: &mut ProjectFileMigrationReport) -> Result<()> {
    let legacy_templates = ship_dir.join("templates");
    if !legacy_templates.is_dir() {
        return Ok(());
    }

    let mappings = [
        ("ISSUE.md", issues_dir(ship_dir).join("TEMPLATE.md")),
        ("SPEC.md", specs_dir(ship_dir).join("TEMPLATE.md")),
        ("FEATURE.md", features_dir(ship_dir).join("TEMPLATE.md")),
        ("RELEASE.md", releases_dir(ship_dir).join("TEMPLATE.md")),
        ("ADR.md", adrs_dir(ship_dir).join("TEMPLATE.md")),
        ("NOTE.md", notes_dir(ship_dir).join("TEMPLATE.md")),
        ("VISION.md", project_ns(ship_dir).join("TEMPLATE.md")),
    ];

    for (legacy_name, target) in mappings {
        let source = legacy_templates.join(legacy_name);
        if !source.exists() {
            continue;
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        match copy_file_with_conflict_guard(&source, &target)? {
            FileCopyOutcome::Copied => {
                report.copied_files += 1;
                fs::remove_file(&source)?;
            }
            FileCopyOutcome::SkippedIdentical => {
                report.skipped_identical_files += 1;
                fs::remove_file(&source)?;
            }
            FileCopyOutcome::ConflictCopied => {
                report.conflict_files += 1;
            }
        }
    }

    if fs::read_dir(&legacy_templates)?.next().is_none() {
        fs::remove_dir(&legacy_templates)?;
    }
    Ok(())
}

fn migrate_directory_tree(
    source: &Path,
    target: &Path,
    report: &mut ProjectFileMigrationReport,
) -> Result<()> {
    if !source.exists() || !source.is_dir() {
        return Ok(());
    }

    if !target.exists() {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        move_dir(source, target)?;
        report.copied_directories += 1;
        return Ok(());
    }

    let conflicts_before = report.conflict_files;
    report.copied_directories += copy_markdown_tree(source, target, report)?;
    // Remove the legacy source directory if every file transferred cleanly
    // (no new conflicts and no archive subdirs we're intentionally keeping).
    if report.conflict_files == conflicts_before && !has_archive_subdir(source) {
        let _ = fs::remove_dir_all(source);
    }
    Ok(())
}

fn has_archive_subdir(dir: &Path) -> bool {
    fs::read_dir(dir)
        .ok()
        .map(|entries| {
            entries
                .flatten()
                .any(|e| e.file_name() == "archive" && e.path().is_dir())
        })
        .unwrap_or(false)
}

fn move_dir(source: &Path, target: &Path) -> Result<()> {
    if fs::rename(source, target).is_ok() {
        return Ok(());
    }
    fs::create_dir_all(target)?;
    for entry in WalkDir::new(source) {
        let entry = entry?;
        let source_path = entry.path();
        let rel = source_path.strip_prefix(source).with_context(|| {
            format!(
                "Failed to strip source root {} from {}",
                source.display(),
                source_path.display()
            )
        })?;
        let target_path = target.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target_path)?;
            continue;
        }
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(source_path, &target_path)?;
    }
    fs::remove_dir_all(source)?;
    Ok(())
}

fn move_file(source: &Path, target: &Path) -> Result<()> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    if fs::rename(source, target).is_ok() {
        return Ok(());
    }
    fs::copy(source, target)?;
    fs::remove_file(source)?;
    Ok(())
}

fn copy_markdown_tree(
    source_root: &Path,
    target_root: &Path,
    report: &mut ProjectFileMigrationReport,
) -> Result<usize> {
    let mut touched_dirs = HashSet::new();
    for entry in WalkDir::new(source_root) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let source_path = entry.path();
        if !source_path.is_file() {
            continue;
        }
        let rel = source_path.strip_prefix(source_root).with_context(|| {
            format!(
                "Failed to strip source root {} from {}",
                source_root.display(),
                source_path.display()
            )
        })?;
        if rel.components().any(|c| c.as_os_str() == "archive") {
            // Keep archives where they are for now; they are documentation history.
            continue;
        }
        if source_path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        let target_path = target_root.join(rel);
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
            touched_dirs.insert(parent.to_path_buf());
        }
        match copy_file_with_conflict_guard(source_path, &target_path)? {
            FileCopyOutcome::Copied => report.copied_files += 1,
            FileCopyOutcome::SkippedIdentical => report.skipped_identical_files += 1,
            FileCopyOutcome::ConflictCopied => report.conflict_files += 1,
        }
    }
    Ok(touched_dirs.len())
}

fn copy_file_with_conflict_guard(source: &Path, target: &Path) -> Result<FileCopyOutcome> {
    if !target.exists() {
        fs::copy(source, target).with_context(|| {
            format!(
                "Failed to copy legacy file {} -> {}",
                source.display(),
                target.display()
            )
        })?;
        return Ok(FileCopyOutcome::Copied);
    }

    let source_content = fs::read(source)?;
    let target_content = fs::read(target)?;
    if source_content == target_content {
        return Ok(FileCopyOutcome::SkippedIdentical);
    }

    let conflict = conflict_path_for(target);
    if !conflict.exists() {
        fs::write(&conflict, source_content).with_context(|| {
            format!(
                "Failed writing conflict copy {} from {}",
                conflict.display(),
                source.display()
            )
        })?;
    } else {
        let existing = fs::read(&conflict)?;
        if existing != source_content {
            fs::write(&conflict, source_content)?;
        }
    }
    Ok(FileCopyOutcome::ConflictCopied)
}

fn conflict_path_for(path: &Path) -> PathBuf {
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = if ext.is_empty() {
        format!("{}-legacy-conflict", stem)
    } else {
        format!("{}-legacy-conflict.{}", stem, ext)
    };
    parent.join(file_name)
}

fn normalize_project_path(input: &Path) -> PathBuf {
    let path = std::fs::canonicalize(input).unwrap_or_else(|_| input.to_path_buf());
    if path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == ".ship")
        .unwrap_or(false)
    {
        return path;
    }
    let ship_candidate = path.join(".ship");
    if ship_candidate.exists() && ship_candidate.is_dir() {
        std::fs::canonicalize(&ship_candidate).unwrap_or(ship_candidate)
    } else {
        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn migrate_project_files_copies_legacy_documents() -> Result<()> {
        let tmp = tempdir()?;
        let ship = tmp.path().join(".ship");
        fs::create_dir_all(ship.join("issues/backlog"))?;
        fs::create_dir_all(ship.join("specs"))?;
        fs::create_dir_all(ship.join("features"))?;
        fs::create_dir_all(ship.join("releases"))?;
        fs::create_dir_all(ship.join("workflow/releases"))?;
        fs::create_dir_all(ship.join("adrs"))?;
        fs::create_dir_all(ship.join("project/releases"))?;

        fs::write(
            ship.join("issues/backlog/legacy.md"),
            "+++\ntitle = \"Legacy\"\n+++\n\nbody",
        )?;
        fs::write(ship.join("specs/vision.md"), "legacy vision")?;
        fs::write(
            ship.join("features/auth.md"),
            "+++\ntitle=\"Auth\"\n+++\n\nbody",
        )?;
        fs::write(
            ship.join("workflow/releases/v1.md"),
            "+++\nversion=\"v1\"\n+++\n\nbody",
        )?;

        let report = migrate_project_files(&ship)?;
        assert!(
            report.copied_files + report.copied_directories >= 4,
            "expected documents/directories to be migrated"
        );
        assert!(ship.join("workflow/issues/backlog/legacy.md").exists());
        assert!(ship.join("project/features/auth.md").exists());
        assert!(ship.join("project/releases/v1.md").exists());
        assert!(ship.join("project/vision.md").exists());
        Ok(())
    }

    #[test]
    fn migrate_global_state_normalizes_registry_paths() -> Result<()> {
        let tmp = tempdir()?;
        let global = tmp.path().join(".ship");
        fs::create_dir_all(&global)?;
        let project_root = tmp.path().join("workspace");
        fs::create_dir_all(project_root.join(".ship"))?;
        let registry = serde_json::json!({
            "projects": [
                { "name": "a", "path": project_root.to_string_lossy() },
                { "name": "a", "path": project_root.join(".ship").to_string_lossy() }
            ]
        });
        fs::write(
            global.join("projects.json"),
            serde_json::to_string_pretty(&registry)?,
        )?;

        let report = migrate_global_state(&global)?;
        assert_eq!(report.registry_entries_before, 2);
        assert_eq!(report.registry_entries_after, 1);
        Ok(())
    }
}
