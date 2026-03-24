use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Context;
use compiler::manifest::ManifestExports;

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

/// Compute per-export hashes for all exported artifacts.
///
/// Skills (directories) are hashed with `compute_tree_hash`.
/// Agents (files) are hashed with `compute_file_hash`.
/// Returns a map of `{ "skills/foo": "sha256:...", "agents/bar.toml": "sha256:..." }`.
pub fn compute_export_hashes(
    ship_dir: &Path,
    exports: &ManifestExports,
) -> anyhow::Result<BTreeMap<String, String>> {
    let mut hashes = BTreeMap::new();

    for skill_path in &exports.skills {
        let full = ship_dir.join(skill_path);
        let hash = compute_tree_hash(&full)
            .with_context(|| format!("hashing exported skill '{skill_path}'"))?;
        hashes.insert(skill_path.clone(), hash);
    }

    for agent_path in &exports.agents {
        let full = ship_dir.join(agent_path);
        let hash = compute_file_hash(&full)
            .with_context(|| format!("hashing exported agent '{agent_path}'"))?;
        hashes.insert(agent_path.clone(), hash);
    }

    Ok(hashes)
}

/// Compute a combined hash from per-export hashes.
///
/// SHA-256 of sorted `"{path}:{hash}"` entries concatenated with newlines.
/// Returns `"sha256:<hex>"`.
pub fn compute_combined_hash(export_hashes: &BTreeMap<String, String>) -> String {
    let mut acc = String::new();
    for (path, hash) in export_hashes {
        acc.push_str(path);
        acc.push(':');
        acc.push_str(hash);
        acc.push('\n');
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
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_compute_tree_hash_deterministic() -> anyhow::Result<()> {
        let dir = tempdir()?;
        fs::write(dir.path().join("a.txt"), "hello")?;
        fs::create_dir_all(dir.path().join("sub"))?;
        fs::write(dir.path().join("sub").join("b.txt"), "world")?;

        let h1 = compute_tree_hash(dir.path())?;
        let h2 = compute_tree_hash(dir.path())?;
        assert_eq!(h1, h2);
        assert!(h1.starts_with("sha256:"));
        Ok(())
    }

    #[test]
    fn test_compute_tree_hash_excludes_git() -> anyhow::Result<()> {
        let dir = tempdir()?;
        fs::write(dir.path().join("file.txt"), "content")?;
        let hash_without_git = compute_tree_hash(dir.path())?;

        fs::create_dir_all(dir.path().join(".git"))?;
        fs::write(dir.path().join(".git").join("HEAD"), "ref: refs/heads/main")?;
        let hash_with_git = compute_tree_hash(dir.path())?;

        assert_eq!(
            hash_without_git, hash_with_git,
            ".git/ must not affect hash"
        );
        Ok(())
    }

    #[test]
    fn test_compute_tree_hash_excludes_ship_lock() -> anyhow::Result<()> {
        let dir = tempdir()?;
        fs::write(dir.path().join("file.txt"), "content")?;
        let hash_without_lock = compute_tree_hash(dir.path())?;

        fs::write(dir.path().join("ship.lock"), "[[package]]\n")?;
        let hash_with_lock = compute_tree_hash(dir.path())?;

        assert_eq!(
            hash_without_lock, hash_with_lock,
            "ship.lock must not affect hash"
        );
        Ok(())
    }

    #[test]
    fn test_compute_tree_hash_excludes_ds_store() -> anyhow::Result<()> {
        let dir = tempdir()?;
        fs::write(dir.path().join("file.txt"), "content")?;
        let hash_without_ds = compute_tree_hash(dir.path())?;

        fs::write(dir.path().join(".DS_Store"), "binary garbage")?;
        let hash_with_ds = compute_tree_hash(dir.path())?;

        assert_eq!(
            hash_without_ds, hash_with_ds,
            ".DS_Store must not affect hash"
        );
        Ok(())
    }

    #[test]
    fn test_compute_tree_hash_different_content() -> anyhow::Result<()> {
        let dir1 = tempdir()?;
        let dir2 = tempdir()?;
        fs::write(dir1.path().join("file.txt"), "content-a")?;
        fs::write(dir2.path().join("file.txt"), "content-b")?;

        let h1 = compute_tree_hash(dir1.path())?;
        let h2 = compute_tree_hash(dir2.path())?;
        assert_ne!(h1, h2);
        Ok(())
    }

    #[test]
    fn test_compute_file_hash() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let path = dir.path().join("test.txt");
        fs::write(&path, "hello")?;
        let h = compute_file_hash(&path)?;
        assert!(h.starts_with("sha256:"));
        // SHA-256 of "hello" is known.
        assert!(h.contains("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"));
        Ok(())
    }

    #[test]
    fn test_compute_export_hashes_skills_and_agents() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let ship = dir.path();

        // Create an exported skill directory
        let skill = ship.join("agents/skills/my-skill");
        fs::create_dir_all(&skill)?;
        fs::write(skill.join("SKILL.md"), "# My Skill")?;

        // Create an exported agent file
        let agents = ship.join("agents/profiles");
        fs::create_dir_all(&agents)?;
        fs::write(agents.join("bar.toml"), "[agent]\nname = \"bar\"")?;

        let exports = ManifestExports {
            skills: vec!["agents/skills/my-skill".into()],
            agents: vec!["agents/profiles/bar.toml".into()],
        };

        let hashes = compute_export_hashes(ship, &exports)?;
        assert_eq!(hashes.len(), 2);
        assert!(hashes["agents/skills/my-skill"].starts_with("sha256:"));
        assert!(hashes["agents/profiles/bar.toml"].starts_with("sha256:"));
        Ok(())
    }

    #[test]
    fn test_internal_change_does_not_affect_export_hashes() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let ship = dir.path();

        // Create an exported skill
        let skill = ship.join("agents/skills/exported");
        fs::create_dir_all(&skill)?;
        fs::write(skill.join("SKILL.md"), "# Exported")?;

        // Create an internal-only skill (not in exports)
        let internal = ship.join("agents/skills/internal");
        fs::create_dir_all(&internal)?;
        fs::write(internal.join("SKILL.md"), "# Internal v1")?;

        let exports = ManifestExports {
            skills: vec!["agents/skills/exported".into()],
            agents: vec![],
        };

        let hashes_before = compute_export_hashes(ship, &exports)?;
        let combined_before = compute_combined_hash(&hashes_before);

        // Modify internal-only skill
        fs::write(internal.join("SKILL.md"), "# Internal v2 changed")?;

        let hashes_after = compute_export_hashes(ship, &exports)?;
        let combined_after = compute_combined_hash(&hashes_after);

        assert_eq!(hashes_before, hashes_after, "internal change must not affect export hashes");
        assert_eq!(combined_before, combined_after, "internal change must not affect combined hash");
        Ok(())
    }

    #[test]
    fn test_exported_change_does_affect_hashes() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let ship = dir.path();

        let skill = ship.join("agents/skills/foo");
        fs::create_dir_all(&skill)?;
        fs::write(skill.join("SKILL.md"), "# Foo v1")?;

        let exports = ManifestExports {
            skills: vec!["agents/skills/foo".into()],
            agents: vec![],
        };

        let hashes_before = compute_export_hashes(ship, &exports)?;
        let combined_before = compute_combined_hash(&hashes_before);

        // Modify the exported skill
        fs::write(skill.join("SKILL.md"), "# Foo v2 changed")?;

        let hashes_after = compute_export_hashes(ship, &exports)?;
        let combined_after = compute_combined_hash(&hashes_after);

        assert_ne!(hashes_before, hashes_after, "exported change must affect export hashes");
        assert_ne!(combined_before, combined_after, "exported change must affect combined hash");
        Ok(())
    }

    #[test]
    fn test_combined_hash_deterministic_and_sorted() {
        let mut map = BTreeMap::new();
        map.insert("z/last".into(), "sha256:aaa".to_string());
        map.insert("a/first".into(), "sha256:bbb".to_string());

        let h1 = compute_combined_hash(&map);
        let h2 = compute_combined_hash(&map);
        assert_eq!(h1, h2);
        assert!(h1.starts_with("sha256:"));
    }

    #[test]
    fn test_combined_hash_empty_exports() {
        let map = BTreeMap::new();
        let h = compute_combined_hash(&map);
        assert!(h.starts_with("sha256:"));
    }

    #[test]
    fn test_should_exclude() {
        assert!(should_exclude(".git/config"));
        assert!(should_exclude(".DS_Store"));
        assert!(should_exclude("ship.lock"));
        assert!(should_exclude("foo.swp"));
        assert!(should_exclude("Thumbs.db"));
        assert!(!should_exclude("README.md"));
        assert!(!should_exclude("src/main.rs"));
    }
}
