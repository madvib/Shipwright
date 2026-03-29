use std::path::Path;
use std::process::Command;

use anyhow::Context;

/// Fetch package content for a specific commit into `dest`.
///
/// Supports two package shapes:
/// - **Ship-native**: repo has `.ship/ship.jsonc` — fetches only `.ship/` contents.
/// - **Root-manifest**: repo has `ship.jsonc` at root — fetches entire repo (dedicated
///   skill packages are small by definition).
///
/// Strategy (tried in order):
/// 1. **GitHub tarball** — download + extract relevant subtree. Fast, CDN-backed.
/// 2. **Sparse checkout** — git 2.25+, works with any host.
/// 3. **Full clone** — shallow clone + selective copy. Last resort.
///
/// `dest` must already exist (created by the caller, e.g., a tempdir).
pub fn fetch_package_content(git_url: &str, commit: &str, dest: &Path) -> anyhow::Result<()> {
    // Try GitHub tarball first (fastest, no git needed).
    if let Some((owner, repo)) = parse_github_url(git_url)
        && try_github_tarball(&owner, &repo, commit, dest).is_ok()
    {
        return Ok(());
    }

    // Try sparse checkout for .ship/ packages.
    if try_sparse_checkout(git_url, commit, dest).is_ok() {
        return Ok(());
    }

    // Full clone fallback — detect package shape and copy accordingly.
    clone_and_extract(git_url, commit, dest)
        .with_context(|| format!("fetching {git_url} @ {commit}"))
}

/// Extract `(owner, repo)` from a GitHub HTTPS URL.
fn parse_github_url(git_url: &str) -> Option<(String, String)> {
    let url = git_url
        .strip_prefix("https://github.com/")
        .or_else(|| git_url.strip_prefix("http://github.com/"))?;
    let url = url.strip_suffix(".git").unwrap_or(url);
    let mut parts = url.splitn(2, '/');
    let owner = parts.next()?.to_string();
    let repo = parts.next()?.to_string();
    if owner.is_empty() || repo.is_empty() || repo.contains('/') {
        return None;
    }
    Some((owner, repo))
}

/// Package shape detected from repo contents.
enum PackageShape {
    /// `.ship/ship.jsonc` exists — fetch only `.ship/`.
    ShipNative,
    /// `ship.jsonc` at repo root — fetch everything (dedicated skill repo).
    RootManifest,
}

/// Download a GitHub tarball and extract package content.
///
/// First pass: detect package shape (`.ship/ship.jsonc` vs root `ship.jsonc`).
/// Second pass: extract the relevant files.
fn try_github_tarball(owner: &str, repo: &str, commit: &str, dest: &Path) -> anyhow::Result<()> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/tarball/{commit}");

    let resp = ureq::get(&url)
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "ship-pkg/0.1")
        .call()
        .map_err(|e| anyhow::anyhow!("GitHub tarball request failed: {e}"))?;

    if resp.status() != 200 {
        anyhow::bail!("GitHub tarball returned HTTP {}", resp.status());
    }

    let body = resp
        .into_body()
        .read_to_vec()
        .context("reading tarball body")?;

    // First pass: detect package shape by scanning entry paths.
    let shape = detect_shape_from_tarball(&body)?;

    // Second pass: extract based on detected shape.
    let gz = flate2::read::GzDecoder::new(body.as_slice());
    let mut archive = tar::Archive::new(gz);
    let mut found_content = false;

    for entry in archive.entries().context("reading tar entries")? {
        let mut entry = entry.context("reading tar entry")?;
        let path = entry.path().context("reading entry path")?.into_owned();
        let path_str = path.to_string_lossy();

        // Determine the relative path to extract based on package shape.
        let rel = match shape {
            PackageShape::ShipNative => match find_ship_segment(&path_str) {
                Some(pos) => path_str[pos..].to_string(),
                None => continue,
            },
            PackageShape::RootManifest => match strip_tarball_prefix(&path_str) {
                Some(rel) if !rel.is_empty() => rel,
                _ => continue,
            },
        };

        found_content = true;
        let target = dest.join(&rel);

        if entry.header().entry_type().is_dir() {
            std::fs::create_dir_all(&target)
                .with_context(|| format!("creating dir {}", target.display()))?;
        } else if entry.header().entry_type().is_file() {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut file = std::fs::File::create(&target)
                .with_context(|| format!("creating file {}", target.display()))?;
            std::io::copy(&mut entry, &mut file)
                .with_context(|| format!("writing {}", target.display()))?;
        }
    }

    if !found_content {
        anyhow::bail!("no package content found in tarball");
    }

    Ok(())
}

/// Scan tarball entries to detect if this is a Ship-native or root-manifest package.
fn detect_shape_from_tarball(tarball_bytes: &[u8]) -> anyhow::Result<PackageShape> {
    let gz = flate2::read::GzDecoder::new(tarball_bytes);
    let mut archive = tar::Archive::new(gz);

    let mut has_ship_manifest = false;
    let mut has_root_manifest = false;

    for entry in archive.entries().context("scanning tar entries")? {
        let entry = entry.context("reading tar entry")?;
        let path = entry.path().context("reading entry path")?;
        let path_str = path.to_string_lossy();

        // GitHub tarball paths: `owner-repo-sha1234/path/to/file`
        if path_str.contains("/.ship/ship.jsonc") {
            has_ship_manifest = true;
            break; // Ship-native wins — no need to scan further.
        }

        // Root-level ship.jsonc: `owner-repo-sha1234/ship.jsonc`
        // Exactly one `/` separator and ends with `ship.jsonc`.
        if path_str.ends_with("/ship.jsonc") && path_str.matches('/').count() == 1 {
            has_root_manifest = true;
        }
    }

    if has_ship_manifest {
        Ok(PackageShape::ShipNative)
    } else if has_root_manifest {
        Ok(PackageShape::RootManifest)
    } else {
        anyhow::bail!(
            "no ship.jsonc found — packages must have .ship/ship.jsonc or ship.jsonc at root"
        )
    }
}

/// Find the start of the `.ship/` segment in a tarball path.
///
/// `"owner-repo-abc1234/.ship/agents/backend.jsonc"` → offset of `.ship/agents/...`
fn find_ship_segment(path: &str) -> Option<usize> {
    if let Some(pos) = path.find("/.ship/") {
        return Some(pos + 1);
    }
    if let Some(pos) = path.find("/.ship")
        && (path.len() == pos + 6 || path.as_bytes().get(pos + 6) == Some(&b'/'))
    {
        return Some(pos + 1);
    }
    None
}

/// Strip the GitHub tarball prefix directory from a path.
///
/// `"owner-repo-sha1234/skills/tdd/SKILL.md"` → `"skills/tdd/SKILL.md"`
/// `"owner-repo-sha1234/"` → `""`
fn strip_tarball_prefix(path: &str) -> Option<String> {
    let slash = path.find('/')?;
    Some(path[slash + 1..].to_string())
}

/// Sparse checkout: only fetch `.ship/` from the remote.
///
/// Requires git 2.25+. Falls back silently on failure.
fn try_sparse_checkout(git_url: &str, commit: &str, dest: &Path) -> anyhow::Result<()> {
    let tmp = tempfile::tempdir().context("creating temp dir for sparse checkout")?;
    let work = tmp.path().join("repo");
    std::fs::create_dir_all(&work)?;

    let steps: Vec<Vec<&str>> = vec![
        vec!["git", "init"],
        vec!["git", "remote", "add", "origin", git_url],
        vec!["git", "sparse-checkout", "init", "--cone"],
        vec!["git", "sparse-checkout", "set", ".ship"],
    ];

    for cmd_args in &steps {
        let status = Command::new(cmd_args[0])
            .args(&cmd_args[1..])
            .current_dir(&work)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .with_context(|| format!("running {}", cmd_args.join(" ")))?;
        if !status.success() {
            anyhow::bail!("sparse checkout step failed: {}", cmd_args.join(" "));
        }
    }

    let fetch_status = Command::new("git")
        .args(["fetch", "--depth=1", "origin", commit])
        .current_dir(&work)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .context("running git fetch for sparse checkout")?;

    if !fetch_status.success() {
        anyhow::bail!("git fetch failed during sparse checkout");
    }

    let checkout_status = Command::new("git")
        .args(["checkout", "FETCH_HEAD"])
        .current_dir(&work)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .context("running git checkout FETCH_HEAD")?;

    if !checkout_status.success() {
        anyhow::bail!("git checkout failed during sparse checkout");
    }

    let ship_dir = work.join(".ship");
    if !ship_dir.is_dir() {
        anyhow::bail!("no .ship/ directory after sparse checkout");
    }

    copy_dir_recursive(&ship_dir, &dest.join(".ship"))
        .context("copying .ship/ from sparse checkout")?;

    Ok(())
}

/// Full clone fallback — detect package shape and copy accordingly.
fn clone_and_extract(git_url: &str, commit: &str, dest: &Path) -> anyhow::Result<()> {
    let tmp = tempfile::tempdir().context("creating temp dir for clone")?;
    let clone_dir = tmp.path().join("clone");

    let clone_status = Command::new("git")
        .args([
            "clone",
            "--depth=1",
            "--no-single-branch",
            git_url,
            clone_dir.to_str().unwrap_or("."),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .context("running git clone")?;

    if !clone_status.success() {
        anyhow::bail!("git clone failed for {git_url}");
    }

    let _ = Command::new("git")
        .args([
            "-C",
            clone_dir.to_str().unwrap_or("."),
            "fetch",
            "--depth=1",
            "origin",
            commit,
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    let checkout_status = Command::new("git")
        .args(["-C", clone_dir.to_str().unwrap_or("."), "checkout", commit])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .context("running git checkout")?;

    if !checkout_status.success() {
        anyhow::bail!("git checkout of commit {commit} failed for {git_url}");
    }

    // Detect package shape from cloned content.
    let ship_dir = clone_dir.join(".ship");
    let root_manifest = clone_dir.join("ship.jsonc");

    if ship_dir.join("ship.jsonc").exists() {
        // Ship-native: copy only .ship/.
        copy_dir_recursive(&ship_dir, &dest.join(".ship")).context("copying .ship/ from clone")?;
    } else if root_manifest.exists() {
        // Root-manifest: copy everything except .git/.
        copy_dir_recursive_excluding(&clone_dir, dest, &[".git"])
            .context("copying package from clone")?;
    } else {
        anyhow::bail!(
            "no ship.jsonc found in {git_url} @ {commit}. \
             Packages must have .ship/ship.jsonc or ship.jsonc at repo root."
        );
    }

    Ok(())
}

/// Recursively copy `src` → `dst`, creating directories as needed.
fn copy_dir_recursive(src: &Path, dst: &Path) -> anyhow::Result<()> {
    copy_dir_recursive_excluding(src, dst, &[".git"])
}

/// Recursively copy `src` → `dst`, skipping directories in `exclude`.
fn copy_dir_recursive_excluding(src: &Path, dst: &Path, exclude: &[&str]) -> anyhow::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in walkdir::WalkDir::new(src)
        .min_depth(1)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !exclude.iter().any(|ex| name == *ex)
        })
    {
        let entry = entry.context("walking source dir")?;
        let rel = entry.path().strip_prefix(src).context("stripping prefix")?;
        let target = dst.join(rel);

        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target)
                .with_context(|| format!("creating dir {}", target.display()))?;
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
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn parse_github_url_standard() {
        let (owner, repo) = parse_github_url("https://github.com/madvib/ship.git").unwrap();
        assert_eq!(owner, "madvib");
        assert_eq!(repo, "ship");
    }

    #[test]
    fn parse_github_url_no_git_suffix() {
        let (owner, repo) = parse_github_url("https://github.com/acme/toolkit").unwrap();
        assert_eq!(owner, "acme");
        assert_eq!(repo, "toolkit");
    }

    #[test]
    fn parse_github_url_non_github() {
        assert!(parse_github_url("https://gitlab.com/owner/repo.git").is_none());
    }

    #[test]
    fn find_ship_segment_in_tarball_path() {
        let path = "madvib-ship-abc1234/.ship/agents/backend.jsonc";
        let pos = find_ship_segment(path).unwrap();
        assert_eq!(&path[pos..], ".ship/agents/backend.jsonc");
    }

    #[test]
    fn find_ship_segment_dir_entry() {
        let path = "owner-repo-sha/.ship/";
        let pos = find_ship_segment(path).unwrap();
        assert_eq!(&path[pos..], ".ship/");
    }

    #[test]
    fn find_ship_segment_no_ship() {
        assert!(find_ship_segment("owner-repo-sha/src/main.rs").is_none());
    }

    #[test]
    fn strip_tarball_prefix_normal() {
        let result = strip_tarball_prefix("owner-repo-sha1234/skills/tdd/SKILL.md").unwrap();
        assert_eq!(result, "skills/tdd/SKILL.md");
    }

    #[test]
    fn strip_tarball_prefix_dir_only() {
        let result = strip_tarball_prefix("owner-repo-sha1234/").unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn strip_tarball_prefix_no_slash() {
        assert!(strip_tarball_prefix("no-slash-here").is_none());
    }

    #[test]
    fn copy_dir_recursive_works() -> anyhow::Result<()> {
        let src = tempdir()?;
        let dst = tempdir()?;

        fs::create_dir_all(src.path().join("sub"))?;
        fs::write(src.path().join("file.txt"), "content")?;
        fs::write(src.path().join("sub/other.txt"), "sub")?;

        copy_dir_recursive(src.path(), &dst.path().join("out"))?;

        assert!(dst.path().join("out/file.txt").exists());
        assert!(dst.path().join("out/sub/other.txt").exists());
        Ok(())
    }

    #[test]
    fn copy_dir_recursive_skips_git() -> anyhow::Result<()> {
        let src = tempdir()?;
        let dst = tempdir()?;

        fs::write(src.path().join("file.txt"), "content")?;
        fs::create_dir_all(src.path().join(".git"))?;
        fs::write(src.path().join(".git/HEAD"), "ref: refs/heads/main")?;

        copy_dir_recursive(src.path(), &dst.path().join("out"))?;

        assert!(dst.path().join("out/file.txt").exists());
        assert!(!dst.path().join("out/.git").exists());
        Ok(())
    }
}
