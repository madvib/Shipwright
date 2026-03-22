use std::path::{Path, PathBuf};

use anyhow::Context;

use super::hash::compute_tree_hash;

/// Percent-encode a dep path for use as a filesystem index key.
/// Only alphanumerics, `-`, `_`, `.` are kept; `/` is encoded as `%2F`, etc.
fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => {
                out.push(b as char);
            }
            other => {
                out.push_str(&format!("%{other:02X}"));
            }
        }
    }
    out
}

/// Metadata for a cached package entry.
#[derive(Debug, Clone)]
pub struct CachedPackage {
    /// dep path e.g. `"github.com/owner/repo"`
    pub path: String,
    /// version string as stored (tag, branch name, or commit SHA)
    pub version: String,
    /// 40-char commit SHA
    pub commit: String,
    /// `"sha256:<hex>"`
    pub hash: String,
    /// Path to the cached content directory under `objects/`.
    pub dir: PathBuf,
}

/// Content-addressed package cache stored at `~/.ship/cache/`.
///
/// Layout:
/// - `<root>/objects/<sha256-hex>/` — package files stored here.
/// - `<root>/index/<url-encoded-dep-path>@<version>` — contains `<sha256-hex>\n<commit>\n`.
pub struct PackageCache {
    root: PathBuf,
}

impl PackageCache {
    /// Create a cache using the default location `~/.ship/cache/`.
    pub fn new() -> anyhow::Result<Self> {
        let home = home::home_dir().context("cannot determine home directory")?;
        Ok(Self::with_root(home.join(".ship").join("cache")))
    }

    /// Create a cache at an explicit root (useful for testing).
    pub fn with_root(root: PathBuf) -> Self {
        Self { root }
    }

    fn objects_dir(&self) -> PathBuf {
        self.root.join("objects")
    }

    fn index_dir(&self) -> PathBuf {
        self.root.join("index")
    }

    fn index_key(&self, dep_path: &str, version: &str) -> String {
        format!("{}@{}", url_encode(dep_path), url_encode(version))
    }

    fn index_path(&self, dep_path: &str, version: &str) -> PathBuf {
        self.index_dir().join(self.index_key(dep_path, version))
    }

    /// Look up a cached package by dep path + version.
    /// Returns `None` on miss or if the index entry is corrupted/missing.
    pub fn get(&self, dep_path: &str, version: &str) -> Option<CachedPackage> {
        let idx_path = self.index_path(dep_path, version);
        let content = std::fs::read_to_string(&idx_path).ok()?;
        let mut lines = content.lines();
        let hex = lines.next()?.trim();
        let commit = lines.next()?.trim();

        if hex.is_empty() || commit.is_empty() {
            return None;
        }

        let hash = format!("sha256:{hex}");
        let dir = self.objects_dir().join(hex);

        if !dir.is_dir() {
            return None;
        }

        Some(CachedPackage {
            path: dep_path.to_string(),
            version: version.to_string(),
            commit: commit.to_string(),
            hash,
            dir,
        })
    }

    /// Store fetched package content.
    ///
    /// Computes the content hash, copies files into `objects/<hex>/`, then
    /// atomically writes the index entry. Returns the stored `CachedPackage`.
    pub fn store(
        &self,
        dep_path: &str,
        version: &str,
        commit: &str,
        content_dir: &Path,
    ) -> anyhow::Result<CachedPackage> {
        std::fs::create_dir_all(self.objects_dir()).context("creating cache objects dir")?;
        std::fs::create_dir_all(self.index_dir()).context("creating cache index dir")?;

        let hash = compute_tree_hash(content_dir).context("computing content hash")?;
        let hex = hash.strip_prefix("sha256:").unwrap_or(&hash);

        let object_dir = self.objects_dir().join(hex);

        // Atomic write: copy to a temp dir alongside objects/, then rename.
        let tmp_name = format!(".tmp-{}", hex);
        let tmp_dir = self.objects_dir().join(&tmp_name);
        if tmp_dir.exists() {
            std::fs::remove_dir_all(&tmp_dir).context("removing stale tmp dir")?;
        }
        std::fs::create_dir_all(&tmp_dir).context("creating tmp object dir")?;

        // Copy content.
        copy_dir_all(content_dir, &tmp_dir).context("copying content to cache")?;

        // Rename into place (idempotent: if already exists, remove tmp).
        if object_dir.exists() {
            std::fs::remove_dir_all(&tmp_dir).ok();
        } else {
            std::fs::rename(&tmp_dir, &object_dir).context("renaming tmp dir to object dir")?;
        }

        // Write index entry atomically.
        let idx_path = self.index_path(dep_path, version);
        let tmp_idx = format!("{}.tmp", idx_path.display());
        let idx_content = format!("{hex}\n{commit}\n");
        std::fs::write(&tmp_idx, &idx_content).context("writing tmp index entry")?;
        std::fs::rename(&tmp_idx, &idx_path).context("renaming tmp index entry")?;

        Ok(CachedPackage {
            path: dep_path.to_string(),
            version: version.to_string(),
            commit: commit.to_string(),
            hash: format!("sha256:{hex}"),
            dir: object_dir,
        })
    }

    /// Re-hash stored content to detect corruption.
    ///
    /// On mismatch: deletes the index entry (so the next `get()` returns `None`)
    /// and returns an error. The objects directory is left in place; a future
    /// GC pass can clean orphaned objects.
    pub fn verify(&self, pkg: &CachedPackage) -> anyhow::Result<()> {
        if !pkg.dir.is_dir() {
            let idx = self.index_path(&pkg.path, &pkg.version);
            let _ = std::fs::remove_file(&idx);
            anyhow::bail!(
                "cache entry for {}@{} missing on disk",
                pkg.path,
                pkg.version
            );
        }

        let actual = compute_tree_hash(&pkg.dir).context("re-hashing cached package")?;

        if actual != pkg.hash {
            // Corrupt entry — delete index so caller re-fetches.
            let idx = self.index_path(&pkg.path, &pkg.version);
            let _ = std::fs::remove_file(&idx);
            anyhow::bail!(
                "cache corruption detected for {}@{}: expected {} got {}",
                pkg.path,
                pkg.version,
                pkg.hash,
                actual
            );
        }
        Ok(())
    }
}

/// Recursively copy `src` into `dst` (dst must exist).
fn copy_dir_all(src: &Path, dst: &Path) -> anyhow::Result<()> {
    for entry in walkdir::WalkDir::new(src).min_depth(1) {
        let entry = entry.context("walking source dir")?;
        let rel = entry.path().strip_prefix(src).context("stripping prefix")?;
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target)?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(entry.path(), &target)
                .with_context(|| format!("copying {}", entry.path().display()))?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "cache_tests.rs"]
mod tests;
