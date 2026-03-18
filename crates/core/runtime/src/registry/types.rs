/// Stub types for ShipManifest and ShipLock.
///
/// TODO: Replace with types from the compiler crate once job khqBn4u2 merges.
/// The compiler crate will own `[module]`, `[dependencies]`, and `[exports]`
/// sections of ship.toml, and the ship.lock file format.
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// A single dependency entry in ship.toml `[dependencies]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// Version constraint: semver range, branch, or 40-char commit SHA.
    pub version: String,
    /// Optional tool-permission grants for this dep's skills.
    #[serde(default)]
    pub grant: Vec<String>,
}

/// Minimal ship.toml manifest (stub until compiler crate exposes its types).
#[derive(Debug, Clone, Default)]
pub struct ShipManifest {
    /// `[dependencies]` section: dep path → Dependency.
    pub dependencies: HashMap<String, Dependency>,
}

/// A single locked package entry in ship.lock.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LockedPackage {
    pub path: String,
    pub version: String,
    pub commit: String,
    pub hash: String,
}

/// ship.lock file contents (stub until compiler crate exposes its types).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShipLock {
    /// Lock format version — always `1` for v0.1.
    pub version: u32,
    #[serde(default)]
    pub package: Vec<LockedPackage>,
}

impl ShipLock {
    /// Detect which dependency paths are present in `manifest` but absent from
    /// this lockfile, and vice versa.
    pub fn check_sync(&self, manifest: &ShipManifest) -> SyncStatus {
        let lock_paths: std::collections::HashSet<&str> =
            self.package.iter().map(|p| p.path.as_str()).collect();
        let manifest_paths: std::collections::HashSet<&str> =
            manifest.dependencies.keys().map(|s| s.as_str()).collect();

        let added: Vec<String> = manifest_paths
            .difference(&lock_paths)
            .map(|s| s.to_string())
            .collect();
        let removed: Vec<String> = lock_paths
            .difference(&manifest_paths)
            .map(|s| s.to_string())
            .collect();

        SyncStatus { added, removed }
    }
}

/// Result of comparing ship.toml and ship.lock.
#[derive(Debug, Default)]
pub struct SyncStatus {
    /// Deps in manifest but not in lock (newly added).
    pub added: Vec<String>,
    /// Deps in lock but not in manifest (removed).
    pub removed: Vec<String>,
}

impl SyncStatus {
    pub fn is_in_sync(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty()
    }
}

/// Parse a ship.lock TOML file.
pub fn parse_ship_lock(content: &str) -> anyhow::Result<ShipLock> {
    let raw: toml::Value = toml::from_str(content)
        .map_err(|e| anyhow::anyhow!("invalid ship.lock: {}", e))?;

    let version = raw
        .get("version")
        .and_then(|v| v.as_integer())
        .unwrap_or(1) as u32;

    let packages = match raw.get("package") {
        Some(toml::Value::Array(arr)) => arr
            .iter()
            .map(|entry| {
                let path = entry
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("[[package]] missing 'path'"))?
                    .to_string();
                let ver = entry
                    .get("version")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("[[package]] missing 'version'"))?
                    .to_string();
                let commit = entry
                    .get("commit")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("[[package]] missing 'commit'"))?
                    .to_string();
                let hash = entry
                    .get("hash")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("[[package]] missing 'hash'"))?
                    .to_string();
                Ok(LockedPackage { path, version: ver, commit, hash })
            })
            .collect::<anyhow::Result<Vec<_>>>()?,
        _ => Vec::new(),
    };

    Ok(ShipLock { version, package: packages })
}

/// Serialize a ShipLock to TOML.
///
/// Output is deterministic: packages sorted by path, fields in fixed order.
pub fn serialize_ship_lock(lock: &ShipLock) -> anyhow::Result<String> {
    let mut packages = lock.package.clone();
    packages.sort_by(|a, b| a.path.cmp(&b.path));

    let mut out = format!("version = {}\n", lock.version);
    for pkg in &packages {
        out.push('\n');
        out.push_str("[[package]]\n");
        out.push_str(&format!("path = {:?}\n", pkg.path));
        out.push_str(&format!("version = {:?}\n", pkg.version));
        out.push_str(&format!("commit = {:?}\n", pkg.commit));
        out.push_str(&format!("hash = {:?}\n", pkg.hash));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ship_lock_round_trip() -> anyhow::Result<()> {
        let raw = r#"version = 1

[[package]]
path = "github.com/owner/pkg"
version = "v1.0.0"
commit = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
hash = "sha256:abc123"
"#;
        let lock = parse_ship_lock(raw)?;
        assert_eq!(lock.version, 1);
        assert_eq!(lock.package.len(), 1);
        assert_eq!(lock.package[0].path, "github.com/owner/pkg");
        assert_eq!(lock.package[0].commit, "a".repeat(40));
        Ok(())
    }

    #[test]
    fn test_serialize_ship_lock_deterministic() -> anyhow::Result<()> {
        let lock = ShipLock {
            version: 1,
            package: vec![
                LockedPackage {
                    path: "github.com/z/z".into(),
                    version: "v2.0.0".into(),
                    commit: "b".repeat(40),
                    hash: "sha256:def".into(),
                },
                LockedPackage {
                    path: "github.com/a/a".into(),
                    version: "v1.0.0".into(),
                    commit: "a".repeat(40),
                    hash: "sha256:abc".into(),
                },
            ],
        };
        let s1 = serialize_ship_lock(&lock)?;
        let s2 = serialize_ship_lock(&lock)?;
        assert_eq!(s1, s2);
        // Sorted by path: a/a before z/z.
        let pos_a = s1.find("github.com/a/a").unwrap();
        let pos_z = s1.find("github.com/z/z").unwrap();
        assert!(pos_a < pos_z);
        Ok(())
    }

    #[test]
    fn test_check_sync_in_sync() {
        let mut manifest = ShipManifest::default();
        manifest.dependencies.insert(
            "github.com/owner/pkg".into(),
            Dependency { version: "^1.0".into(), grant: vec![] },
        );
        let lock = ShipLock {
            version: 1,
            package: vec![LockedPackage {
                path: "github.com/owner/pkg".into(),
                version: "v1.0.0".into(),
                commit: "a".repeat(40),
                hash: "sha256:abc".into(),
            }],
        };
        assert!(lock.check_sync(&manifest).is_in_sync());
    }

    #[test]
    fn test_check_sync_added() {
        let mut manifest = ShipManifest::default();
        manifest.dependencies.insert(
            "github.com/owner/new".into(),
            Dependency { version: "^1.0".into(), grant: vec![] },
        );
        let lock = ShipLock { version: 1, package: vec![] };
        let status = lock.check_sync(&manifest);
        assert!(!status.is_in_sync());
        assert!(status.added.contains(&"github.com/owner/new".to_string()));
        assert!(status.removed.is_empty());
    }

    #[test]
    fn test_check_sync_removed() {
        let manifest = ShipManifest::default();
        let lock = ShipLock {
            version: 1,
            package: vec![LockedPackage {
                path: "github.com/owner/old".into(),
                version: "v1.0.0".into(),
                commit: "a".repeat(40),
                hash: "sha256:abc".into(),
            }],
        };
        let status = lock.check_sync(&manifest);
        assert!(!status.is_in_sync());
        assert!(status.removed.contains(&"github.com/owner/old".to_string()));
    }
}
