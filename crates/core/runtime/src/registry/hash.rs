use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Context;

/// Files excluded from content hashing.
fn should_exclude(rel_path: &str) -> bool {
    let parts: Vec<&str> = rel_path.split('/').collect();
    // Exclude anything under .git/
    if parts.first() == Some(&".git") {
        return true;
    }
    // Exclude ship.lock (registry lockfile, not part of package content)
    if rel_path == "ship.lock" {
        return true;
    }
    // Exclude .ship/state/ — user/project variable state, not package content.
    // state files carry local config and must not affect the published content hash.
    if parts.first() == Some(&"state") {
        return true;
    }
    // Exclude OS / editor noise files.
    let filename = parts.last().unwrap_or(&"");
    matches!(*filename, ".DS_Store" | "Thumbs.db") || filename.ends_with(".swp")
}

/// Compute a deterministic SHA-256 content hash for the file tree at `root`.
///
/// Algorithm:
/// 1. Collect all files recursively, excluding `.git/`, `.DS_Store`,
///    `Thumbs.db`, `*.swp`, and `ship.lock`.
/// 2. Sort file paths lexicographically (relative to root, using `/` separators).
/// 3. For each file accumulate: `"<rel-path>\0<byte-length>\0<sha256-of-content>"`.
/// 4. SHA-256 of the full accumulated string.
/// 5. Return `"sha256:<lowercase-hex>"`.
pub fn compute_tree_hash(root: &Path) -> anyhow::Result<String> {
    let mut file_entries: Vec<(String, u64, String)> = Vec::new();

    for entry in walkdir::WalkDir::new(root)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|e| e.file_name() != ".git")
    {
        let entry = entry.context("walking package tree")?;
        if !entry.file_type().is_file() {
            continue;
        }

        let rel = entry
            .path()
            .strip_prefix(root)
            .context("stripping root prefix")?;

        // Normalize to forward-slash separators for cross-platform determinism.
        let rel_str = rel
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("/");

        if should_exclude(&rel_str) {
            continue;
        }

        let content = std::fs::read(entry.path())
            .with_context(|| format!("reading {}", entry.path().display()))?;

        let byte_len = content.len() as u64;
        let file_hash = hex::encode_sha256(&content);
        file_entries.push((rel_str, byte_len, file_hash));
    }

    // Lexicographic sort by relative path (walkdir sorts by filename within each
    // directory; re-sort globally to ensure full-path ordering).
    file_entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut accumulator = String::new();
    for (rel_path, byte_len, file_hash) in &file_entries {
        accumulator.push_str(rel_path);
        accumulator.push('\0');
        accumulator.push_str(&byte_len.to_string());
        accumulator.push('\0');
        accumulator.push_str(file_hash);
    }

    let tree_hash = hex::encode_sha256(accumulator.as_bytes());
    Ok(format!("sha256:{tree_hash}"))
}

/// Compute SHA-256 hash of a single file.
/// Returns `"sha256:<lowercase-hex>"`.
pub fn compute_file_hash(path: &Path) -> anyhow::Result<String> {
    let content = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    Ok(format!("sha256:{}", hex::encode_sha256(&content)))
}

/// Per-export content hashes and a combined top-level hash.
#[derive(Debug, Clone, PartialEq)]
pub struct ExportHashes {
    /// Map from export path (e.g. `"agents/skills/my-skill"`) to `"sha256:<hex>"`.
    pub per_export: BTreeMap<String, String>,
    /// Combined hash: SHA-256 of the sorted per-export hashes concatenated.
    pub combined: String,
}

/// Compute per-export content hashes for a package.
///
/// For each export path in `skill_exports` and `agent_exports`, hashes the
/// corresponding subtree or file under `ship_dir`. Returns per-export hashes
/// plus a combined hash derived only from exported artifacts.
///
/// Export paths are relative to `.ship/` (e.g. `"agents/skills/my-skill"`).
/// Skill exports are directories (hashed with `compute_tree_hash`).
/// Agent exports are single files (hashed with `compute_file_hash`).
pub fn compute_export_hashes(
    ship_dir: &Path,
    skill_exports: &[String],
    agent_exports: &[String],
) -> anyhow::Result<ExportHashes> {
    let mut per_export = BTreeMap::new();

    for skill_path in skill_exports {
        let full = ship_dir.join(skill_path);
        if !full.is_dir() {
            anyhow::bail!(
                "exported skill '{}' not found at {}",
                skill_path,
                full.display()
            );
        }
        let hash = compute_tree_hash(&full)
            .with_context(|| format!("hashing exported skill '{}'", skill_path))?;
        per_export.insert(skill_path.clone(), hash);
    }

    for agent_path in agent_exports {
        let full = ship_dir.join(agent_path);
        if !full.is_file() {
            anyhow::bail!(
                "exported agent '{}' not found at {}",
                agent_path,
                full.display()
            );
        }
        let hash = compute_file_hash(&full)
            .with_context(|| format!("hashing exported agent '{}'", agent_path))?;
        per_export.insert(agent_path.clone(), hash);
    }

    let combined = combine_export_hashes(&per_export);
    Ok(ExportHashes {
        per_export,
        combined,
    })
}

/// Derive a combined hash from sorted per-export hashes.
///
/// Concatenates `"<path>\0<hash>"` for each export in sorted order,
/// then SHA-256s the result. Returns `"sha256:<hex>"`.
fn combine_export_hashes(hashes: &BTreeMap<String, String>) -> String {
    let mut acc = String::new();
    for (path, hash) in hashes {
        acc.push_str(path);
        acc.push('\0');
        acc.push_str(hash);
    }
    if acc.is_empty() {
        return format!("sha256:{}", hex::encode_sha256(b""));
    }
    format!("sha256:{}", hex::encode_sha256(acc.as_bytes()))
}

mod hex {
    use sha2::{Digest, Sha256};

    pub fn encode_sha256(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        result.iter().map(|b| format!("{b:02x}")).collect()
    }
}

#[cfg(test)]
#[path = "hash_tests.rs"]
mod tests;
