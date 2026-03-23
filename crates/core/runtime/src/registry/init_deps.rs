//! Default registry dependencies for `ship init`.
//!
//! When `ship init` creates a new project, this module seeds default `@ship/*`
//! dependencies into `ship.jsonc` and optionally resolves them via the registry.
//! If the registry is unreachable, the caller falls back to bare scaffold skills.

use std::path::Path;

use anyhow::{Context, Result};

/// Default dependencies seeded into every new project.
/// Each entry is `(package_path, version_constraint)`.
pub const DEFAULT_INIT_DEPS: &[(&str, &str)] = &[
    ("@ship/task-policy", "latest"),
    ("@ship/ship-help", "latest"),
];

/// Result of seeding default dependencies.
#[derive(Debug)]
pub struct SeedResult {
    /// Number of dependencies added to ship.jsonc.
    pub added: usize,
    /// True if ship.jsonc already had a `dependencies` section (idempotent no-op).
    pub already_present: bool,
}

/// Merge default `@ship/*` dependencies into an existing `ship.jsonc`.
///
/// Idempotent: if the file already contains a `dependencies` key, no changes are
/// made. Returns [`SeedResult`] describing what happened.
///
/// The function reads the raw JSONC, strips comments, round-trips through
/// `serde_json::Value`, injects the `dependencies` object, and writes back
/// with pretty formatting.
pub fn seed_default_dependencies(ship_dir: &Path) -> Result<SeedResult> {
    let jsonc_path = ship_dir.join(crate::config::PRIMARY_CONFIG_FILE);
    if !jsonc_path.exists() {
        anyhow::bail!(
            "ship.jsonc not found at {}; run project init first",
            jsonc_path.display()
        );
    }

    let raw = std::fs::read_to_string(&jsonc_path)
        .with_context(|| format!("reading {}", jsonc_path.display()))?;

    let mut doc: serde_json::Value = compiler::jsonc::from_jsonc_str(&raw)
        .with_context(|| format!("parsing {}", jsonc_path.display()))?;

    // Idempotent: if dependencies already exist, leave them alone.
    if doc.get("dependencies").is_some() {
        return Ok(SeedResult {
            added: 0,
            already_present: true,
        });
    }

    let mut deps = serde_json::Map::new();
    for &(path, version) in DEFAULT_INIT_DEPS {
        deps.insert(
            path.to_string(),
            serde_json::Value::String(version.to_string()),
        );
    }

    doc.as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("ship.jsonc root is not a JSON object"))?
        .insert(
            "dependencies".to_string(),
            serde_json::Value::Object(deps),
        );

    let pretty = serde_json::to_string_pretty(&doc).context("serializing ship.jsonc")?;
    crate::fs_util::write_atomic(&jsonc_path, pretty)?;

    Ok(SeedResult {
        added: DEFAULT_INIT_DEPS.len(),
        already_present: false,
    })
}

/// Attempt to install the dependencies declared in `ship.jsonc`.
///
/// This is a best-effort operation for use after `ship init`. It reads the
/// `dependencies` section from ship.jsonc (without requiring a `module` section),
/// builds a lightweight registry manifest, and runs `resolve_and_fetch`.
///
/// Returns `Ok(true)` if install succeeded, `Ok(false)` if it was skipped
/// (no deps), or an error. Callers should catch errors and fall back gracefully.
pub fn try_install_init_deps(ship_dir: &Path) -> Result<bool> {
    let jsonc_path = ship_dir.join(crate::config::PRIMARY_CONFIG_FILE);
    if !jsonc_path.exists() {
        return Ok(false);
    }

    let raw = std::fs::read_to_string(&jsonc_path)
        .with_context(|| format!("reading {}", jsonc_path.display()))?;
    let doc: serde_json::Value = compiler::jsonc::from_jsonc_str(&raw)
        .with_context(|| format!("parsing {}", jsonc_path.display()))?;

    let deps_val = match doc.get("dependencies") {
        Some(v) if v.is_object() => v,
        _ => return Ok(false),
    };

    // Build a lightweight RegistryManifest from the dependencies section.
    let deps_map: std::collections::HashMap<String, super::types::Dependency> = deps_val
        .as_object()
        .unwrap()
        .iter()
        .filter_map(|(k, v)| {
            let version = match v {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Object(obj) => {
                    obj.get("version")?.as_str()?.to_string()
                }
                _ => return None,
            };
            Some((
                k.clone(),
                super::types::Dependency {
                    version,
                    grant: vec![],
                },
            ))
        })
        .collect();

    if deps_map.is_empty() {
        return Ok(false);
    }

    let manifest = super::types::ShipManifest {
        dependencies: deps_map,
    };

    let lock_path = ship_dir.join("ship.lock");
    let cache = super::PackageCache::new().context("initializing package cache")?;
    let opts = super::install::InstallOptions {
        frozen: false,
        offline: false,
    };

    super::install::resolve_and_fetch(&manifest, &lock_path, &cache, &opts)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn init_ship_dir(tmp: &std::path::Path) -> std::path::PathBuf {
        let ship_dir = tmp.join(".ship");
        fs::create_dir_all(&ship_dir).unwrap();
        ship_dir
    }

    fn write_ship_jsonc(ship_dir: &std::path::Path, content: &str) {
        fs::write(
            ship_dir.join(crate::config::PRIMARY_CONFIG_FILE),
            content,
        )
        .unwrap();
    }

    #[test]
    fn seed_adds_dependencies_to_empty_manifest() {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = init_ship_dir(tmp.path());
        write_ship_jsonc(
            &ship_dir,
            r#"{ "project": { "providers": ["claude"] } }"#,
        );

        let result = seed_default_dependencies(&ship_dir).unwrap();
        assert!(!result.already_present);
        assert_eq!(result.added, DEFAULT_INIT_DEPS.len());

        // Verify the file now has dependencies.
        let content = fs::read_to_string(
            ship_dir.join(crate::config::PRIMARY_CONFIG_FILE),
        )
        .unwrap();
        let doc: serde_json::Value = serde_json::from_str(&content).unwrap();
        let deps = doc.get("dependencies").expect("dependencies key missing");
        assert!(deps.is_object());
        for &(path, version) in DEFAULT_INIT_DEPS {
            assert_eq!(
                deps.get(path).and_then(|v| v.as_str()),
                Some(version),
                "missing dep {path}"
            );
        }
    }

    #[test]
    fn seed_is_idempotent_when_deps_exist() {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = init_ship_dir(tmp.path());
        write_ship_jsonc(
            &ship_dir,
            r#"{ "dependencies": { "github.com/a/b": "main" } }"#,
        );

        let result = seed_default_dependencies(&ship_dir).unwrap();
        assert!(result.already_present);
        assert_eq!(result.added, 0);

        // Original deps must be unchanged.
        let content = fs::read_to_string(
            ship_dir.join(crate::config::PRIMARY_CONFIG_FILE),
        )
        .unwrap();
        let doc: serde_json::Value = serde_json::from_str(&content).unwrap();
        let deps = doc.get("dependencies").unwrap();
        assert_eq!(
            deps.get("github.com/a/b").and_then(|v| v.as_str()),
            Some("main")
        );
        // Default deps must NOT have been added.
        assert!(deps.get("@ship/task-policy").is_none());
    }

    #[test]
    fn seed_errors_when_no_ship_jsonc() {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = init_ship_dir(tmp.path());
        // No ship.jsonc written.
        let err = seed_default_dependencies(&ship_dir).unwrap_err();
        assert!(
            err.to_string().contains("ship.jsonc not found"),
            "got: {err}"
        );
    }

    #[test]
    fn seed_preserves_existing_fields() {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = init_ship_dir(tmp.path());
        write_ship_jsonc(
            &ship_dir,
            r#"{ "id": "abc123", "project": { "providers": ["claude"] } }"#,
        );

        seed_default_dependencies(&ship_dir).unwrap();

        let content = fs::read_to_string(
            ship_dir.join(crate::config::PRIMARY_CONFIG_FILE),
        )
        .unwrap();
        let doc: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(doc.get("id").and_then(|v| v.as_str()), Some("abc123"));
        assert!(doc.get("project").is_some());
        assert!(doc.get("dependencies").is_some());
    }

    #[test]
    fn seed_handles_jsonc_with_comments() {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = init_ship_dir(tmp.path());
        write_ship_jsonc(
            &ship_dir,
            r#"{
  // Ship project manifest
  "project": {
    "providers": ["claude"], // default provider
  },
}"#,
        );

        let result = seed_default_dependencies(&ship_dir).unwrap();
        assert!(!result.already_present);
        assert_eq!(result.added, DEFAULT_INIT_DEPS.len());
    }

    #[test]
    fn try_install_returns_false_when_no_deps() {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = init_ship_dir(tmp.path());
        write_ship_jsonc(
            &ship_dir,
            r#"{ "project": { "providers": ["claude"] } }"#,
        );

        let result = try_install_init_deps(&ship_dir).unwrap();
        assert!(!result, "should return false when no dependencies");
    }

    #[test]
    fn try_install_returns_false_when_no_file() {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = init_ship_dir(tmp.path());

        let result = try_install_init_deps(&ship_dir).unwrap();
        assert!(!result);
    }

    #[test]
    fn try_install_fails_gracefully_for_unreachable_registry() {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = init_ship_dir(tmp.path());
        write_ship_jsonc(
            &ship_dir,
            r#"{ "dependencies": { "@ship/task-policy": "latest" } }"#,
        );

        // @ship/* packages can't resolve via git — this will error.
        let result = try_install_init_deps(&ship_dir);
        // The function returns an error (not a panic) which the caller catches.
        assert!(result.is_err(), "expected error for unresolvable @ship/* dep");
    }
}
