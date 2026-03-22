//! ship.lock parser and writer.
//!
//! `ship.lock` is the registry dependency lockfile (`.ship/ship.lock`).
//! It records direct resolved dependencies — path, version, commit, and content hash.
//!
//! NOTE: `.ship/ship.state` (formerly `.ship/ship.lock`) stores workspace state
//! (active_profile, compiled_at). These are completely separate files.

use std::path::Path;

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::manifest::ShipManifest;

// ── Types ──────────────────────────────────────────────────────────────────────

/// A single locked package entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LockPackage {
    /// Package path, e.g. `github.com/owner/repo`.
    pub path: String,
    /// Resolved version string (tag or branch@commit).
    pub version: String,
    /// Exact 40-char hex commit SHA.
    pub commit: String,
    /// Content hash in `sha256:<hex>` format.
    pub hash: String,
}

/// The parsed `.ship/ship.lock` file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShipLock {
    /// Lockfile format version. Must be `1` for v0.1.
    pub version: u32,
    /// Locked packages, one entry per direct dependency.
    #[serde(rename = "package", default)]
    pub packages: Vec<LockPackage>,
}

impl Default for ShipLock {
    fn default() -> Self {
        ShipLock {
            version: 1,
            packages: vec![],
        }
    }
}

impl ShipLock {
    /// Parse a `ship.lock` file.
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Cannot read {}", path.display()))?;
        let lock: ShipLock = toml::from_str(&content)
            .with_context(|| format!("Failed to parse {}", path.display()))?;
        Ok(lock)
    }

    /// Serialize to a deterministic TOML string.
    ///
    /// Packages are sorted by path. Fields are emitted in a fixed order
    /// (path, version, commit, hash) so the output is byte-identical for
    /// the same logical content.
    pub fn to_toml_string(&self) -> String {
        let mut pkgs = self.packages.clone();
        pkgs.sort_by(|a, b| a.path.cmp(&b.path));

        let mut out = format!("version = {}\n", self.version);
        for pkg in &pkgs {
            out.push('\n');
            out.push_str("[[package]]\n");
            out.push_str(&format!("path = {:?}\n", pkg.path));
            out.push_str(&format!("version = {:?}\n", pkg.version));
            out.push_str(&format!("commit = {:?}\n", pkg.commit));
            out.push_str(&format!("hash = {:?}\n", pkg.hash));
        }
        out
    }

    /// Atomically write the lockfile: write to `<path>.tmp` then rename.
    pub fn write_atomic(&self, path: &Path) -> anyhow::Result<()> {
        let tmp = path.with_extension("lock.tmp");
        let content = self.to_toml_string();
        std::fs::write(&tmp, &content)
            .with_context(|| format!("Cannot write temp lockfile {}", tmp.display()))?;
        std::fs::rename(&tmp, path)
            .with_context(|| format!("Cannot rename lockfile into place at {}", path.display()))?;
        Ok(())
    }

    /// Compare manifest dependencies against locked packages.
    ///
    /// Returns `(added, removed)` where:
    /// - `added`   = dep paths in manifest but not in lockfile
    /// - `removed` = dep paths in lockfile but not in manifest
    pub fn check_sync(&self, manifest: &ShipManifest) -> (Vec<String>, Vec<String>) {
        let manifest_keys: std::collections::HashSet<&str> =
            manifest.dependencies.keys().map(|k| k.as_str()).collect();
        let lock_paths: std::collections::HashSet<&str> =
            self.packages.iter().map(|p| p.path.as_str()).collect();

        let added: Vec<String> = manifest_keys
            .difference(&lock_paths)
            .map(|s| s.to_string())
            .collect();
        let removed: Vec<String> = lock_paths
            .difference(&manifest_keys)
            .map(|s| s.to_string())
            .collect();
        (added, removed)
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::ShipManifest;

    fn sample_lock() -> ShipLock {
        ShipLock {
            version: 1,
            packages: vec![
                LockPackage {
                    path: "github.com/b/second".into(),
                    version: "1.0.0".into(),
                    commit: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into(),
                    hash: "sha256:bbbb".into(),
                },
                LockPackage {
                    path: "github.com/a/first".into(),
                    version: "2.0.0".into(),
                    commit: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
                    hash: "sha256:aaaa".into(),
                },
            ],
        }
    }

    #[test]
    fn to_toml_string_is_sorted() {
        let lock = sample_lock();
        let s = lock.to_toml_string();
        let a_pos = s.find("github.com/a/first").unwrap();
        let b_pos = s.find("github.com/b/second").unwrap();
        assert!(a_pos < b_pos, "packages should be sorted by path");
    }

    #[test]
    fn to_toml_string_fixed_field_order() {
        let lock = ShipLock {
            version: 1,
            packages: vec![LockPackage {
                path: "github.com/x/y".into(),
                version: "1.0.0".into(),
                commit: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
                hash: "sha256:cafe".into(),
            }],
        };
        let s = lock.to_toml_string();
        // Find the [[package]] block start, then check field order within it
        let pkg_block_start = s.find("[[package]]").unwrap();
        let block = &s[pkg_block_start..];
        let path_pos = block.find("path =").unwrap();
        // Find version = after path = (skip the top-level "version = 1")
        let ver_pos = block.find("version =").unwrap();
        let commit_pos = block.find("commit =").unwrap();
        let hash_pos = block.find("hash =").unwrap();
        assert!(path_pos < ver_pos, "path before version in package block");
        assert!(
            ver_pos < commit_pos,
            "version before commit in package block"
        );
        assert!(commit_pos < hash_pos, "commit before hash in package block");
    }

    #[test]
    fn to_toml_string_deterministic() {
        let lock = sample_lock();
        assert_eq!(lock.to_toml_string(), lock.to_toml_string());
    }

    #[test]
    fn round_trip_via_file() {
        let lock = sample_lock();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ship.lock");
        lock.write_atomic(&path).unwrap();
        let loaded = ShipLock::from_file(&path).unwrap();
        // Both must contain same packages (order may differ after toml re-parse)
        assert_eq!(loaded.version, 1);
        let mut names: Vec<_> = loaded.packages.iter().map(|p| p.path.as_str()).collect();
        names.sort();
        assert_eq!(names, vec!["github.com/a/first", "github.com/b/second"]);
    }

    #[test]
    fn from_file_missing_returns_error() {
        let r = ShipLock::from_file(std::path::Path::new("/nonexistent/ship.lock"));
        assert!(r.is_err());
    }

    #[test]
    fn check_sync_both_empty() {
        let lock = ShipLock::default();
        let manifest = ShipManifest::from_toml_str(
            r#"[module]
name = "github.com/x/y"
version = "1.0.0"
"#,
        )
        .unwrap();
        let (added, removed) = lock.check_sync(&manifest);
        assert!(added.is_empty());
        assert!(removed.is_empty());
    }

    #[test]
    fn check_sync_added_dep() {
        let lock = ShipLock::default();
        let manifest = ShipManifest::from_toml_str(
            r#"
[module]
name = "github.com/x/y"
version = "1.0.0"

[dependencies]
"github.com/a/b" = "^1.0.0"
"#,
        )
        .unwrap();
        let (added, removed) = lock.check_sync(&manifest);
        assert_eq!(added, vec!["github.com/a/b"]);
        assert!(removed.is_empty());
    }

    #[test]
    fn check_sync_removed_dep() {
        let lock = ShipLock {
            version: 1,
            packages: vec![LockPackage {
                path: "github.com/a/b".into(),
                version: "1.0.0".into(),
                commit: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
                hash: "sha256:aaaa".into(),
            }],
        };
        let manifest = ShipManifest::from_toml_str(
            r#"[module]
name = "github.com/x/y"
version = "1.0.0"
"#,
        )
        .unwrap();
        let (added, removed) = lock.check_sync(&manifest);
        assert!(added.is_empty());
        assert_eq!(removed, vec!["github.com/a/b"]);
    }

    #[test]
    fn write_atomic_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ship.lock");
        assert!(!path.exists());
        ShipLock::default().write_atomic(&path).unwrap();
        assert!(path.exists());
        // Ensure tmp file is cleaned up
        assert!(!dir.path().join("ship.lock.tmp").exists());
    }

    #[test]
    fn version_field_is_written() {
        let s = ShipLock::default().to_toml_string();
        assert!(s.starts_with("version = 1"), "got: {s}");
    }
}
