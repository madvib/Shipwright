//! Resolve skill discovery directories from the project manifest.
//!
//! By default, skills live in `.ship/skills/`. The `project.skill_paths` field
//! in `ship.jsonc` allows additional directories (relative to the project root).

use std::path::{Path, PathBuf};

/// Default skill path — always anchored to `.ship/`.
const DEFAULT_SKILL_DIR: &str = "skills/";

/// Read `project.skill_paths` from the manifest and return resolved absolute
/// directory paths.
///
/// - `.ship/skills/` is always included as the first entry.
/// - Custom paths from `project.skill_paths` resolve relative to `project_root`.
/// - Absolute paths are rejected and skipped.
pub fn read_skill_paths(ship_dir: &Path, project_root: &Path) -> Vec<PathBuf> {
    let mut result = vec![ship_dir.join(DEFAULT_SKILL_DIR)];

    let raw_paths = read_raw_skill_paths(ship_dir);
    for p in &raw_paths {
        if Path::new(p).is_absolute() {
            continue;
        }
        let resolved = project_root.join(p);
        if resolved != result[0] {
            result.push(resolved);
        }
    }

    result
}

/// Read the raw string values from the manifest without resolving them.
/// Returns an empty vec when the field is absent or unreadable.
fn read_raw_skill_paths(ship_dir: &Path) -> Vec<String> {
    let primary = ship_dir.join(crate::config::PRIMARY_CONFIG_FILE);
    let content = match std::fs::read_to_string(&primary) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let parsed: serde_json::Value = match compiler::jsonc::from_jsonc_str(&content) {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    let paths = parsed
        .get("project")
        .and_then(|p| p.get("skill_paths"))
        .and_then(|v| v.as_array());

    match paths {
        Some(arr) => arr
            .iter()
            .filter_map(|v| v.as_str().map(ToOwned::to_owned))
            .collect(),
        None => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write_manifest(ship_dir: &Path, content: &str) {
        fs::create_dir_all(ship_dir).unwrap();
        fs::write(ship_dir.join(crate::config::PRIMARY_CONFIG_FILE), content).unwrap();
    }

    #[test]
    fn defaults_to_skills_when_absent() {
        let tmp = tempdir().unwrap();
        let project_root = tmp.path();
        let ship_dir = project_root.join(".ship");
        write_manifest(&ship_dir, r#"{"id": "test123"}"#);

        let paths = read_skill_paths(&ship_dir, project_root);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], ship_dir.join("skills/"));
    }

    #[test]
    fn defaults_when_no_manifest() {
        let tmp = tempdir().unwrap();
        let project_root = tmp.path();
        let ship_dir = project_root.join(".ship");
        fs::create_dir_all(&ship_dir).unwrap();

        let paths = read_skill_paths(&ship_dir, project_root);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], ship_dir.join("skills/"));
    }

    #[test]
    fn reads_custom_skill_paths() {
        let tmp = tempdir().unwrap();
        let project_root = tmp.path();
        let ship_dir = project_root.join(".ship");
        write_manifest(
            &ship_dir,
            r#"{"id": "test123", "project": {"skill_paths": ["docs/", "internal/skills/"]}}"#,
        );

        let paths = read_skill_paths(&ship_dir, project_root);
        assert_eq!(paths.len(), 3);
        assert_eq!(paths[0], ship_dir.join("skills/"));
        assert_eq!(paths[1], project_root.join("docs/"));
        assert_eq!(paths[2], project_root.join("internal/skills/"));
    }

    #[test]
    fn rejects_absolute_paths() {
        let tmp = tempdir().unwrap();
        let project_root = tmp.path();
        let ship_dir = project_root.join(".ship");
        write_manifest(
            &ship_dir,
            r#"{"id": "test123", "project": {"skill_paths": ["/etc/evil/"]}}"#,
        );

        let paths = read_skill_paths(&ship_dir, project_root);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], ship_dir.join("skills/"));
    }

    #[test]
    fn empty_array_falls_back_to_default() {
        let tmp = tempdir().unwrap();
        let project_root = tmp.path();
        let ship_dir = project_root.join(".ship");
        write_manifest(
            &ship_dir,
            r#"{"id": "test123", "project": {"skill_paths": []}}"#,
        );

        let paths = read_skill_paths(&ship_dir, project_root);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], ship_dir.join("skills/"));
    }

    #[test]
    fn project_root_relative_custom_path() {
        let tmp = tempdir().unwrap();
        let project_root = tmp.path();
        let ship_dir = project_root.join(".ship");
        write_manifest(
            &ship_dir,
            r#"{"id": "test123", "project": {"skill_paths": ["custom-skills/"]}}"#,
        );

        let paths = read_skill_paths(&ship_dir, project_root);
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0], ship_dir.join("skills/"));
        assert_eq!(paths[1], project_root.join("custom-skills/"));
    }
}
