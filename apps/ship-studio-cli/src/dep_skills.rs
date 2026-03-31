//! Resolve dep skill refs from the package cache into [`Skill`] values.
//!
//! Dep skill refs look like `host.tld/owner/pkg/skill-name` (any git host).
//! Local refs (no hostname prefix) are left to the caller to handle.
//!
//! Resolution path:
//!   dep ref → split package path + within-package path
//!           → look up package in ship.lock → get `sha256:<hex>` hash
//!           → cache dir `~/.ship/cache/objects/<hex>/`
//!           → if `<within-path>/SKILL.md` exists → single skill
//!           → if `<within-path>/` is a namespace dir (sub-dirs with SKILL.md)
//!             → expand to all leaf skills within

use anyhow::{Context, Result};
use compiler::lockfile::{LockPackage, ShipLock};
use compiler::{Skill, SkillSource};
use std::path::{Path, PathBuf};

// ── Dep ref detection ─────────────────────────────────────────────────────────

/// Return `true` if `ref_str` is a dep skill ref (`host.tld/owner/pkg/…`).
///
/// A dep ref has a first slash-separated segment containing a dot (the hostname)
/// and at least two more slashes (owner + package + within-package path).
/// Local refs are plain skill ids such as `my-skill` or `review-pr`.
pub fn is_dep_ref(ref_str: &str) -> bool {
    let Some(first_slash) = ref_str.find('/') else {
        return false;
    };
    let host = &ref_str[..first_slash];
    // Host must contain a dot (e.g. github.com, git.example.com)
    if !host.contains('.') {
        return false;
    }
    // Need at least two more slashes after host: owner/pkg/within
    let after_host = &ref_str[first_slash + 1..];
    after_host.matches('/').count() >= 2
}

/// Parse a dep skill ref into `(package_path, within_package_path)`.
///
/// The package path is the first three slash-separated segments
/// (`host.tld/owner/pkg`). Everything after is the within-package path
/// (`skills/skill-name`).
///
/// Returns `None` if the ref has fewer than four segments (no within-package
/// path) or if the first segment contains no dot (not a hostname).
pub fn parse_dep_ref(ref_str: &str) -> Option<(&str, &str)> {
    // Split into segments: host / owner / pkg / within...
    let first_slash = ref_str.find('/')?;
    let host = &ref_str[..first_slash];
    if !host.contains('.') {
        return None;
    }
    // Find owner and pkg slashes
    let after_host = &ref_str[first_slash + 1..];
    let owner_slash = after_host.find('/')?;
    let after_owner = &after_host[owner_slash + 1..];
    let pkg_slash = after_owner.find('/')?;

    let pkg_end = first_slash + 1 + owner_slash + 1 + pkg_slash;
    let package_path = &ref_str[..pkg_end];
    let within_path = &ref_str[pkg_end + 1..];

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
    let pkg: Option<&LockPackage> = lock.packages.iter().find(|p| p.path == package_path);

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

/// Resolve a dep skill ref to one or more [`Skill`] values.
///
/// If the ref points to a directory with a `SKILL.md`, returns a single skill.
/// If it points to a **namespace directory** (no `SKILL.md` but contains
/// sub-directories that each have `SKILL.md`), expands to all leaf skills.
///
/// `ref_str`    — full dep ref, e.g. `github.com/owner/pkg/better-auth`
/// `lock`       — parsed ship.lock
/// `cache_root` — `~/.ship/cache/`
pub fn resolve_dep_skill(ref_str: &str, lock: &ShipLock, cache_root: &Path) -> Result<Vec<Skill>> {
    let (package_path, within_path) = parse_dep_ref(ref_str)
        .ok_or_else(|| anyhow::anyhow!("invalid dep skill ref: '{}'", ref_str))?;

    let hex = hash_from_lock(lock, package_path)?;

    let target_dir = cache_skill_path(cache_root, &hex, within_path);
    let skill_md = target_dir.join("SKILL.md");

    // Direct skill — the ref points to a directory with SKILL.md.
    if skill_md.exists() {
        let raw = std::fs::read_to_string(&skill_md)
            .with_context(|| format!("reading cached skill file {}", skill_md.display()))?;
        return Ok(vec![parse_dep_skill(ref_str, &raw)]);
    }

    // Namespace expansion — the ref points to a directory containing sub-skills.
    if target_dir.is_dir() {
        let leaf_skills = find_leaf_skills(&target_dir);
        if !leaf_skills.is_empty() {
            let mut skills = Vec::new();
            for (sub_name, sub_path) in &leaf_skills {
                let raw = std::fs::read_to_string(sub_path)
                    .with_context(|| format!("reading {}", sub_path.display()))?;
                let sub_ref = format!("{}/{}", ref_str, sub_name);
                skills.push(parse_dep_skill(&sub_ref, &raw));
            }
            return Ok(skills);
        }
    }

    // Neither a skill nor a namespace — build a helpful error.
    let parent = Path::new(within_path).parent().unwrap_or(Path::new(""));
    let skills_dir = cache_skill_path(cache_root, &hex, &parent.to_string_lossy());
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
                allowed_tools = v.split_whitespace().map(str::to_string).collect();
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
        stable_id: None,
        description,
        license,
        compatibility,
        allowed_tools,
        metadata,
        content,
        source: SkillSource::Community,
        vars: Default::default(),
        artifacts: vec![],
    }
}

/// Find all leaf skill sub-directories within a namespace directory.
///
/// Returns `(sub_name, path_to_SKILL.md)` for each sub-directory that contains
/// a `SKILL.md`. Sorted by name for deterministic output.
fn find_leaf_skills(namespace_dir: &Path) -> Vec<(String, PathBuf)> {
    let Ok(entries) = std::fs::read_dir(namespace_dir) else {
        return vec![];
    };
    let mut results: Vec<(String, PathBuf)> = entries
        .flatten()
        .filter_map(|e| {
            let skill_md = e.path().join("SKILL.md");
            if e.path().is_dir() && skill_md.exists() {
                Some((e.file_name().to_string_lossy().to_string(), skill_md))
            } else {
                None
            }
        })
        .collect();
    results.sort_by(|a, b| a.0.cmp(&b.0));
    results
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
/// - Refs without a hostname prefix are skipped (they are local).
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
        let skills = resolve_dep_skill(ref_str, &lock, cache_root)
            .with_context(|| format!("resolving dep skill '{}'", ref_str))?;
        for skill in skills {
            if !existing_ids.contains(skill.id.as_str()) {
                dep_skills.push(skill);
            }
        }
    }

    Ok(dep_skills)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[path = "dep_skills_tests_resolve.rs"]
mod tests;
