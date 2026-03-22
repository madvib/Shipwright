use std::collections::HashMap;
use std::process::Command;

use anyhow::Context;
use semver::Version;

use super::constraint::{VersionConstraint, normalize_version};

/// Resolved version: original tag string plus exact commit SHA.
#[derive(Debug, Clone)]
pub struct ResolvedVersion {
    /// Original tag string (with `v` prefix if present in the remote).
    pub tag: String,
    /// 40-char SHA-1 commit hash.
    pub commit: String,
}

/// Raw refs returned by `git ls-remote`.
#[derive(Debug, Default)]
pub struct RemoteRefs {
    /// tag name (without `refs/tags/` prefix) → commit SHA
    pub tags: HashMap<String, String>,
    /// branch name (without `refs/heads/` prefix) → commit SHA
    pub heads: HashMap<String, String>,
}

/// Build an HTTPS clone URL from a package path.
///
/// `github.com/owner/repo` → `https://github.com/owner/repo.git`
fn build_git_url(package_path: &str) -> String {
    if package_path.starts_with("https://") || package_path.starts_with("git@") {
        return package_path.to_string();
    }
    format!("https://{}.git", package_path)
}

/// Run `git ls-remote --tags --heads <url>` and parse the output.
pub fn list_remote_refs(git_url: &str) -> anyhow::Result<RemoteRefs> {
    let output = Command::new("git")
        .args(["ls-remote", "--tags", "--heads", git_url])
        .output()
        .with_context(|| format!("failed to run git ls-remote for {git_url}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git ls-remote failed for {git_url}: {}", stderr.trim());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut refs = RemoteRefs::default();

    for line in stdout.lines() {
        let mut parts = line.splitn(2, '\t');
        let sha = match parts.next() {
            Some(s) => s.trim().to_string(),
            None => continue,
        };
        let refname = match parts.next() {
            Some(r) => r.trim(),
            None => continue,
        };

        // Skip peeled tag refs (e.g. refs/tags/v1.0.0^{})
        if refname.ends_with("^{}") {
            continue;
        }

        if let Some(tag) = refname.strip_prefix("refs/tags/") {
            refs.tags.insert(tag.to_string(), sha);
        } else if let Some(head) = refname.strip_prefix("refs/heads/") {
            refs.heads.insert(head.to_string(), sha);
        }
    }

    Ok(refs)
}

/// Resolve a version constraint to an exact commit SHA.
pub fn resolve_version(
    package_path: &str,
    constraint: &VersionConstraint,
) -> anyhow::Result<ResolvedVersion> {
    match constraint {
        VersionConstraint::Commit(sha) => {
            // Use verbatim — no network call needed.
            Ok(ResolvedVersion {
                tag: sha.clone(),
                commit: sha.clone(),
            })
        }
        VersionConstraint::Branch(branch) => {
            let git_url = build_git_url(package_path);
            let refs = list_remote_refs(&git_url)
                .with_context(|| format!("listing refs for {package_path}"))?;
            let sha = refs.heads.get(branch).ok_or_else(|| {
                let available: Vec<_> = refs.heads.keys().collect();
                anyhow::anyhow!(
                    "branch {:?} not found for {package_path}; available: {:?}",
                    branch,
                    available
                )
            })?;
            Ok(ResolvedVersion {
                tag: branch.clone(),
                commit: sha.clone(),
            })
        }
        VersionConstraint::Semver(req) => {
            let git_url = build_git_url(package_path);
            let refs = list_remote_refs(&git_url)
                .with_context(|| format!("listing refs for {package_path}"))?;

            // Collect all tags that parse as semver and match the requirement.
            let mut candidates: Vec<(Version, String, String)> = Vec::new();
            for (tag, sha) in &refs.tags {
                let normalized = normalize_version(tag);
                if let Ok(version) = Version::parse(normalized)
                    && req.matches(&version)
                {
                    candidates.push((version, tag.clone(), sha.clone()));
                }
            }

            if candidates.is_empty() {
                let available: Vec<&String> = refs.tags.keys().collect();
                anyhow::bail!(
                    "no tag matches {:?} for {package_path}; available tags: {:?}",
                    req.to_string(),
                    available
                );
            }

            // Select the highest matching version (deterministic).
            candidates.sort_by(|a, b| b.0.cmp(&a.0));
            let (_, tag, sha) = candidates.remove(0);

            Ok(ResolvedVersion { tag, commit: sha })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_git_url_github() {
        assert_eq!(
            build_git_url("github.com/owner/repo"),
            "https://github.com/owner/repo.git"
        );
    }

    #[test]
    fn test_build_git_url_passthrough() {
        assert_eq!(
            build_git_url("https://github.com/owner/repo.git"),
            "https://github.com/owner/repo.git"
        );
    }

    #[test]
    fn test_resolve_commit_no_network() {
        let sha = "a".repeat(40);
        let constraint = VersionConstraint::Commit(sha.clone());
        let result = resolve_version("github.com/owner/repo", &constraint).unwrap();
        assert_eq!(result.commit, sha);
        assert_eq!(result.tag, sha);
    }

    #[test]
    fn test_parse_remote_refs_output() {
        // Simulate parsing ls-remote output directly.
        let raw = "abc123def456abc123def456abc123def456abc1\trefs/tags/v1.0.0\n\
                   bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\trefs/tags/v1.0.0^{}\n\
                   cccccccccccccccccccccccccccccccccccccccc\trefs/heads/main\n";

        let mut refs = RemoteRefs::default();
        for line in raw.lines() {
            let mut parts = line.splitn(2, '\t');
            let sha = parts.next().unwrap().trim().to_string();
            let refname = parts.next().unwrap().trim();
            if refname.ends_with("^{}") {
                continue;
            }
            if let Some(tag) = refname.strip_prefix("refs/tags/") {
                refs.tags.insert(tag.to_string(), sha);
            } else if let Some(head) = refname.strip_prefix("refs/heads/") {
                refs.heads.insert(head.to_string(), sha);
            }
        }

        assert_eq!(refs.tags.len(), 1);
        assert!(refs.tags.contains_key("v1.0.0"));
        assert!(refs.heads.contains_key("main"));
        // Peeled ref must be excluded.
        assert!(!refs.tags.contains_key("v1.0.0^{}"));
    }
}
