use std::path::Path;
use std::process::Command;

use anyhow::Context;

/// Fetch package content for a specific commit into `dest`.
///
/// Strategy:
/// 1. Try `git archive --remote=<url> <commit> | tar -x -C <dest>`.
///    Many hosts (GitHub) disable `git archive` over HTTPS; if the command fails
///    we fall back to a shallow clone.
/// 2. Fallback: `git clone --depth=1 <url> <tmp>`, checkout the exact commit,
///    then copy contents (excluding `.git/`) into `dest`.
///
/// `dest` must already exist (created by the caller, e.g., a tempdir).
pub fn fetch_package_content(git_url: &str, commit: &str, dest: &Path) -> anyhow::Result<()> {
    if try_git_archive(git_url, commit, dest).is_ok() {
        return Ok(());
    }
    clone_and_copy(git_url, commit, dest)
        .with_context(|| format!("fetching {git_url} @ {commit}"))
}

fn try_git_archive(git_url: &str, commit: &str, dest: &Path) -> anyhow::Result<()> {
    // git archive --remote=<url> <commit> | tar -x -C <dest>
    let archive = Command::new("git")
        .args(["archive", &format!("--remote={git_url}"), commit])
        .output()
        .context("running git archive")?;

    if !archive.status.success() {
        let stderr = String::from_utf8_lossy(&archive.stderr);
        anyhow::bail!("git archive failed: {}", stderr.trim());
    }

    // Pipe the tarball through `tar -x -C <dest>`.
    let tar = Command::new("tar")
        .args(["-x", "-C", dest.to_str().unwrap_or(".")])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .context("spawning tar")?;

    // Write the archive bytes to tar's stdin.
    use std::io::Write;
    let mut child = tar;
    {
        let stdin = child.stdin.as_mut().context("opening tar stdin")?;
        stdin.write_all(&archive.stdout).context("writing to tar stdin")?;
    }

    let status = child.wait().context("waiting for tar")?;
    if !status.success() {
        anyhow::bail!("tar extraction failed");
    }
    Ok(())
}

fn clone_and_copy(git_url: &str, commit: &str, dest: &Path) -> anyhow::Result<()> {
    let tmp = tempfile::tempdir().context("creating temp dir for clone")?;
    let clone_dir = tmp.path().join("clone");

    // Shallow clone — we'll check out the exact commit afterward.
    let clone_status = Command::new("git")
        .args([
            "clone",
            "--depth=1",
            "--no-single-branch",
            git_url,
            clone_dir.to_str().unwrap_or("."),
        ])
        .status()
        .context("running git clone")?;

    if !clone_status.success() {
        anyhow::bail!("git clone failed for {git_url}");
    }

    // Fetch the exact commit (in case it isn't the tip of any branch).
    let _ = Command::new("git")
        .args(["-C", clone_dir.to_str().unwrap_or("."), "fetch", "--depth=1", "origin", commit])
        .status();

    // Checkout the exact commit.
    let checkout_status = Command::new("git")
        .args([
            "-C",
            clone_dir.to_str().unwrap_or("."),
            "checkout",
            commit,
        ])
        .status()
        .context("running git checkout")?;

    if !checkout_status.success() {
        anyhow::bail!("git checkout of commit {commit} failed for {git_url}");
    }

    // Copy all files excluding .git/.
    copy_dir_excluding_git(&clone_dir, dest)
        .context("copying cloned content to cache dest")?;

    Ok(())
}

/// Recursively copy `src` → `dst`, skipping the `.git/` directory.
fn copy_dir_excluding_git(src: &Path, dst: &Path) -> anyhow::Result<()> {
    for entry in walkdir::WalkDir::new(src)
        .min_depth(1)
        .into_iter()
        .filter_entry(|e| {
            e.file_name() != ".git"
        })
    {
        let entry = entry.context("walking clone dir")?;
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
    fn test_copy_dir_excluding_git() -> anyhow::Result<()> {
        let src = tempdir()?;
        let dst = tempdir()?;

        // Create some files and a .git dir.
        fs::write(src.path().join("file.txt"), "content")?;
        fs::create_dir_all(src.path().join(".git"))?;
        fs::write(src.path().join(".git").join("HEAD"), "ref: refs/heads/main")?;
        fs::create_dir_all(src.path().join("subdir"))?;
        fs::write(src.path().join("subdir").join("other.txt"), "sub")?;

        copy_dir_excluding_git(src.path(), dst.path())?;

        assert!(dst.path().join("file.txt").exists());
        assert!(dst.path().join("subdir").join("other.txt").exists());
        assert!(!dst.path().join(".git").exists());
        Ok(())
    }
}
