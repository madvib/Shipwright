//! Resolve dep skill refs from the package cache into [`Skill`] values.
//!
//! Dep skill refs look like `github.com/owner/pkg/skills/skill-name`.
//! Local refs (no `github.com/` prefix) are left to the caller to handle.
//!
//! Resolution path:
//!   dep ref → split package path + within-package path
//!           → look up package in ship.lock → get `sha256:<hex>` hash
//!           → cache dir `~/.ship/cache/objects/<hex>/`
//!           → read `<within-package-path>/SKILL.md`
//!           → parse as a [`Skill`]

use anyhow::{Context, Result};
use compiler::{Skill, SkillSource};
use compiler::lockfile::{LockPackage, ShipLock};
use std::path::{Path, PathBuf};

// ── Dep ref detection ─────────────────────────────────────────────────────────

/// Return `true` if `ref_str` is a dep skill ref (starts with `github.com/`).
///
/// Local refs are plain skill ids such as `my-skill` or `review-pr`.
pub fn is_dep_ref(ref_str: &str) -> bool {
    ref_str.starts_with("github.com/")
}

/// Parse a dep skill ref into `(package_path, within_package_path)`.
///
/// The package path is the first three slash-separated segments
/// (`github.com/owner/pkg`). Everything after is the within-package path
/// (`skills/skill-name`).
///
/// Returns `None` if the ref has fewer than four segments (no within-package
/// path) or does not start with `github.com/`.
pub fn parse_dep_ref(ref_str: &str) -> Option<(&str, &str)> {
    if !ref_str.starts_with("github.com/") {
        return None;
    }
    // Find the third slash (after "github.com/owner/pkg")
    let after_scheme = &ref_str["github.com/".len()..];
    // after_scheme is "owner/pkg/skills/skill-name"
    // We need two more slashes for owner and pkg
    let first_slash = after_scheme.find('/')?;
    let after_owner = &after_scheme[first_slash + 1..];
    let second_slash = after_owner.find('/')?;
    let pkg_end = "github.com/".len() + first_slash + 1 + second_slash;

    let package_path = &ref_str[..pkg_end];
    let within_path = &ref_str[pkg_end + 1..]; // skip the third slash

    if within_path.is_empty() {
        return None;
    }

    Some((package_path, within_path))
}

// ── Cache-based resolution ────────────────────────────────────────────────────

/// Look up the cache hash for `package_path` in the given lockfile.
///
/// Returns the hex digest (without the `sha256:` prefix) on success.
/// Returns an error if the package is absent — the caller should surface this
/// as a cache-miss requiring `ship install`.
pub fn hash_from_lock(lock: &ShipLock, package_path: &str) -> Result<String> {
    let pkg: Option<&LockPackage> = lock
        .packages
        .iter()
        .find(|p| p.path == package_path);

    let pkg = pkg.ok_or_else(|| {
        anyhow::anyhow!(
            "dependency {} not in cache — run ship install",
            package_path
        )
    })?;

    let hex = pkg
        .hash
        .strip_prefix("sha256:")
        .ok_or_else(|| anyhow::anyhow!("malformed hash '{}' for {}", pkg.hash, package_path))?
        .to_string();

    Ok(hex)
}

/// Build the filesystem path to a within-package path under the cache.
///
/// `cache_root` is `~/.ship/cache/` (or a test override).
/// Returns `<cache_root>/objects/<hex>/<within_path>`.
pub fn cache_skill_path(cache_root: &Path, hex: &str, within_path: &str) -> PathBuf {
    cache_root.join("objects").join(hex).join(within_path)
}

/// Resolve a single dep skill ref to a [`Skill`].
///
/// `ref_str`    — full dep ref, e.g. `github.com/owner/pkg/skills/name`
/// `lock`       — parsed ship.lock
/// `cache_root` — `~/.ship/cache/`
pub fn resolve_dep_skill(ref_str: &str, lock: &ShipLock, cache_root: &Path) -> Result<Skill> {
    let (package_path, within_path) = parse_dep_ref(ref_str)
        .ok_or_else(|| anyhow::anyhow!("invalid dep skill ref: '{}'", ref_str))?;

    let hex = hash_from_lock(lock, package_path)?;

    // Skill files live at <within_path>/SKILL.md
    // Spec: ~/.ship/cache/objects/<sha256>/skills/name/SKILL.md
    let skill_md = cache_skill_path(cache_root, &hex, within_path).join("SKILL.md");

    if !skill_md.exists() {
        // Build a list of available skills for a helpful error
        let skills_dir = cache_skill_path(cache_root, &hex, "skills");
        let available = list_available_skills(&skills_dir);
        let available_str = if available.is_empty() {
            "(none found)".to_string()
        } else {
            available.join(", ")
        };
        anyhow::bail!(
            "dep skill '{}': path '{}' not found in cached package '{}'; \
             available skills: {}",
            ref_str,
            within_path,
            package_path,
            available_str
        );
    }

    let raw = std::fs::read_to_string(&skill_md)
        .with_context(|| format!("reading cached skill file {}", skill_md.display()))?;

    // Parse the skill, using the full dep ref as the id so mode filters match.
    let skill = parse_dep_skill(ref_str, &raw);
    Ok(skill)
}

/// Parse a SKILL.md from a dep package, using `dep_ref` as the skill id.
///
/// Parses per the agentskills.io spec: `name`, `description`, `license`,
/// `compatibility`, `allowed-tools`, `metadata`. Legacy `version` and `author`
/// top-level keys are folded into `metadata`.
fn parse_dep_skill(dep_ref: &str, raw: &str) -> Skill {
    use std::collections::HashMap;

    let mut name = dep_ref.to_string();
    let mut description = None;
    let mut license = None;
    let mut compatibility = None;
    let mut allowed_tools = vec![];
    let mut metadata: HashMap<String, String> = HashMap::new();
    let mut content_start = 0usize;

    if let Some(rest) = raw.strip_prefix("---\n")
        && let Some(end) = rest.find("\n---\n")
    {
        let fm = &rest[..end];
        let mut in_metadata = false;
        for line in fm.lines() {
            if in_metadata {
                if line.starts_with("  ") || line.starts_with('\t') {
                    let trimmed = line.trim();
                    if let Some((k, v)) = trimmed.split_once(':') {
                        metadata.insert(k.trim().to_string(), v.trim().to_string());
                    }
                    continue;
                } else {
                    in_metadata = false;
                }
            }
            if let Some(v) = line.strip_prefix("name:") {
                name = v.trim().to_string();
            } else if let Some(v) = line.strip_prefix("description:") {
                description = Some(v.trim().to_string());
            } else if let Some(v) = line.strip_prefix("license:") {
                license = Some(v.trim().to_string());
            } else if let Some(v) = line.strip_prefix("compatibility:") {
                compatibility = Some(v.trim().to_string());
            } else if let Some(v) = line.strip_prefix("allowed-tools:") {
                allowed_tools = v.trim().split_whitespace().map(str::to_string).collect();
            } else if line.trim_end() == "metadata:" {
                in_metadata = true;
            } else if let Some(v) = line.strip_prefix("version:") {
                metadata.insert("version".to_string(), v.trim().to_string());
            } else if let Some(v) = line.strip_prefix("author:") {
                metadata.insert("author".to_string(), v.trim().to_string());
            }
        }
        content_start = 4 + end + 5; // "---\n" + fm + "\n---\n"
    }

    let content = raw[content_start..].trim().to_string();
    Skill {
        id: dep_ref.to_string(),
        name,
        description,
        license,
        compatibility,
        allowed_tools,
        metadata,
        content,
        source: SkillSource::Community,
    }
}

/// List skill names available in `skills_dir` (best-effort, for error messages).
fn list_available_skills(skills_dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(skills_dir) else {
        return vec![];
    };
    entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect()
}

// ── Batch resolution entry point ──────────────────────────────────────────────

/// Resolve all dep skill refs found in `skill_refs`, merging them with
/// `local_skills` (which remain unchanged).
///
/// - Refs without `github.com/` prefix are skipped (they are local).
/// - Refs already present by id in `local_skills` are skipped (no duplicates).
/// - A missing lock or cache hit causes a hard error with an actionable message.
///
/// `lock_path`  — `.ship/ship.lock`
/// `cache_root` — `~/.ship/cache/` (pass `None` to use the default)
pub fn resolve_dep_skills(
    skill_refs: &[String],
    local_skills: &[Skill],
    lock_path: &Path,
    cache_root: Option<&Path>,
) -> Result<Vec<Skill>> {
    // Collect only dep refs
    let dep_refs: Vec<&str> = skill_refs
        .iter()
        .filter(|r| is_dep_ref(r))
        .map(String::as_str)
        .collect();

    if dep_refs.is_empty() {
        return Ok(vec![]);
    }

    // Need the lockfile
    let lock = ShipLock::from_file(lock_path).with_context(|| {
        format!(
            "cannot read {} — run ship install to populate the lock file",
            lock_path.display()
        )
    })?;

    // Determine cache root
    let default_cache: PathBuf;
    let cache_root = match cache_root {
        Some(p) => p,
        None => {
            let home = dirs::home_dir().context("cannot determine home directory")?;
            default_cache = home.join(".ship").join("cache");
            &default_cache
        }
    };

    // Build existing id set to deduplicate
    let existing_ids: std::collections::HashSet<&str> =
        local_skills.iter().map(|s| s.id.as_str()).collect();

    let mut dep_skills = Vec::new();
    for ref_str in dep_refs {
        if existing_ids.contains(ref_str) {
            continue; // already present locally, skip
        }
        let skill = resolve_dep_skill(ref_str, &lock, cache_root)
            .with_context(|| format!("resolving dep skill '{}'", ref_str))?;
        dep_skills.push(skill);
    }

    Ok(dep_skills)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[path = "dep_skills_tests.rs"]
mod tests;
