//! Resolve skill discovery directories from the project manifest.
//!
//! By default, skills live in `.ship/skills/`. The `project.skill_paths` field
//! in `ship.jsonc` allows additional directories (relative to `.ship/`).

use std::path::{Path, PathBuf};

/// Default skill path when `project.skill_paths` is absent from the manifest.
const DEFAULT_SKILL_PATH: &str = "skills/";

/// Read `project.skill_paths` from the manifest and return resolved absolute
/// directory paths. Falls back to `["skills/"]` when the field is absent.
///
/// Paths are relative to `ship_dir` (the `.ship/` directory). Absolute paths
/// are rejected and skipped.
pub fn read_skill_paths(ship_dir: &Path) -> Vec<PathBuf> {
    let raw_paths = read_raw_skill_paths(ship_dir);
    let mut result = Vec::new();
    for p in &raw_paths {
        if Path::new(p).is_absolute() {
            continue;
        }
        result.push(ship_dir.join(p));
    }
    result
}

/// Read the raw string values from the manifest without resolving them.
/// Returns `["skills/"]` when the field is absent or unreadable.
fn read_raw_skill_paths(ship_dir: &Path) -> Vec<String> {
    let primary = ship_dir.join(crate::config::PRIMARY_CONFIG_FILE);
    let content = match std::fs::read_to_string(&primary) {
        Ok(c) => c,
        Err(_) => return vec![DEFAULT_SKILL_PATH.to_string()],
    };
    let parsed: serde_json::Value = match compiler::jsonc::from_jsonc_str(&content) {
        Ok(v) => v,
        Err(_) => return vec![DEFAULT_SKILL_PATH.to_string()],
    };

    let paths = parsed
        .get("project")
        .and_then(|p| p.get("skill_paths"))
        .and_then(|v| v.as_array());

    match paths {
        Some(arr) => {
            let strings: Vec<String> = arr
                .iter()
                .filter_map(|v| v.as_str().map(ToOwned::to_owned))
                .collect();
            if strings.is_empty() {
                vec![DEFAULT_SKILL_PATH.to_string()]
            } else {
                strings
            }
        }
        None => vec![DEFAULT_SKILL_PATH.to_string()],
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
        let ship_dir = tmp.path().join(".ship");
        write_manifest(&ship_dir, r#"{"id": "test123"}"#);

        let paths = read_skill_paths(&ship_dir);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], ship_dir.join("skills/"));
    }

    #[test]
    fn defaults_when_no_manifest() {
        let tmp = tempdir().unwrap();
        let ship_dir = tmp.path().join(".ship");
        fs::create_dir_all(&ship_dir).unwrap();
        // No ship.jsonc written

        let paths = read_skill_paths(&ship_dir);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], ship_dir.join("skills/"));
    }

    #[test]
    fn reads_custom_skill_paths() {
        let tmp = tempdir().unwrap();
        let ship_dir = tmp.path().join(".ship");
        write_manifest(
            &ship_dir,
            r#"{"id": "test123", "project": {"skill_paths": ["skills/", "docs/", "internal/skills/"]}}"#,
        );

        let paths = read_skill_paths(&ship_dir);
        assert_eq!(paths.len(), 3);
        assert_eq!(paths[0], ship_dir.join("skills/"));
        assert_eq!(paths[1], ship_dir.join("docs/"));
        assert_eq!(paths[2], ship_dir.join("internal/skills/"));
    }

    #[test]
    fn rejects_absolute_paths() {
        let tmp = tempdir().unwrap();
        let ship_dir = tmp.path().join(".ship");
        write_manifest(
            &ship_dir,
            r#"{"id": "test123", "project": {"skill_paths": ["skills/", "/etc/evil/"]}}"#,
        );

        let paths = read_skill_paths(&ship_dir);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], ship_dir.join("skills/"));
    }

    #[test]
    fn empty_array_falls_back_to_default() {
        let tmp = tempdir().unwrap();
        let ship_dir = tmp.path().join(".ship");
        write_manifest(
            &ship_dir,
            r#"{"id": "test123", "project": {"skill_paths": []}}"#,
        );

        let paths = read_skill_paths(&ship_dir);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], ship_dir.join("skills/"));
    }

    #[test]
    fn raw_paths_returns_strings() {
        let tmp = tempdir().unwrap();
        let ship_dir = tmp.path().join(".ship");
        write_manifest(
            &ship_dir,
            r#"{"id": "test123", "project": {"skill_paths": ["skills/", "custom/"]}}"#,
        );

        let raw = read_raw_skill_paths(&ship_dir);
        assert_eq!(raw, vec!["skills/", "custom/"]);
    }
}
