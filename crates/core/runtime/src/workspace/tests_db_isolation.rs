#[cfg(test)]
mod tests {
    use crate::workspace::*;
    use anyhow::Result;
    use tempfile::tempdir;

    // ---- 1. guard: get_global_dir refuses production paths in test context ----

    /// Verifies that get_global_dir() never silently returns a production path
    /// in test context.  Two valid outcomes:
    ///
    /// - Clean env (SHIP_GLOBAL_DIR unset / temp): returns a temp-based path.
    /// - Polluted env (SHIP_GLOBAL_DIR=~/.ship): guard panics — test isolation
    ///   is enforced.
    #[test]
    fn get_global_dir_never_silently_returns_production_path_in_tests() {
        let result = std::panic::catch_unwind(|| crate::project::get_global_dir());
        match result {
            Ok(Ok(path)) => {
                // auto_test_global_dir() provided isolation — path must be temp-based.
                assert!(
                    path.starts_with(std::env::temp_dir()),
                    "global dir in test context must be under temp dir; got: {}",
                    path.display()
                );
            }
            Err(_) => {
                // Guard fired: correct behavior when SHIP_GLOBAL_DIR points to a
                // production path.  Contamination was prevented.
            }
            Ok(Err(e)) => panic!("unexpected error from get_global_dir: {e}"),
        }
    }

    // ---- 2. db isolation: db_path() never points to home DB in tests ---------

    /// db_path() in test context must either return a temp-based path or panic
    /// (guard fires).  Either outcome prevents production DB contamination.
    #[test]
    fn db_path_in_test_context_never_points_to_home_db() {
        let result = std::panic::catch_unwind(|| crate::db::db_path().unwrap());
        match result {
            Ok(path) => {
                let home_db = home::home_dir().map(|h| h.join(".ship/platform.db"));
                if let Some(home_path) = home_db {
                    assert_ne!(
                        path, home_path,
                        "db_path() must not resolve to the production database"
                    );
                }
            }
            Err(_) => {
                // Guard fired — contamination prevented.
            }
        }
    }

    // ---- 3. git worktree porcelain parser ------------------------------------

    #[test]
    fn parse_git_worktree_path_finds_matching_branch() {
        let porcelain = "\
worktree /home/user/dev/ship
HEAD aaaaaaa
branch refs/heads/main

worktree /home/user/dev/ship-worktrees/v0.2.0
HEAD bbbbbbb
branch refs/heads/v0.2.0

worktree /home/user/dev/ship-worktrees/feat-x
HEAD ccccccc
branch refs/heads/feature/fix-x
";
        let path =
            crate::workspace::helpers::parse_git_worktree_path(porcelain, "v0.2.0");
        assert_eq!(
            path.as_deref(),
            Some("/home/user/dev/ship-worktrees/v0.2.0")
        );

        let feature_path =
            crate::workspace::helpers::parse_git_worktree_path(porcelain, "feature/fix-x");
        assert_eq!(
            feature_path.as_deref(),
            Some("/home/user/dev/ship-worktrees/feat-x")
        );

        let missing =
            crate::workspace::helpers::parse_git_worktree_path(porcelain, "feature/nonexistent");
        assert!(missing.is_none());
    }

    // ---- 4. activate_workspace uses real git worktree path ------------------

    /// Requires a clean environment (SHIP_GLOBAL_DIR unset or pointing to temp).
    /// In a polluted env the test is skipped via the guard that panics on init.
    #[test]
    fn activate_workspace_picks_up_real_git_worktree_path() -> Result<()> {
        use std::process::Command;

        let repo_dir = tempdir()?;
        let worktree_dir = tempdir()?;

        let run = |cmd: &mut Command| -> Result<()> {
            let status = cmd.status()?;
            if !status.success() {
                anyhow::bail!("command failed: {:?}", cmd);
            }
            Ok(())
        };

        run(Command::new("git")
            .args(["-C", repo_dir.path().to_str().unwrap(), "init", "-b", "main"]))?;
        run(Command::new("git")
            .args(["-C", repo_dir.path().to_str().unwrap(), "config", "user.email", "test@test.com"]))?;
        run(Command::new("git")
            .args(["-C", repo_dir.path().to_str().unwrap(), "config", "user.name", "Test"]))?;
        let readme = repo_dir.path().join("README.md");
        std::fs::write(&readme, "test")?;
        run(Command::new("git")
            .args(["-C", repo_dir.path().to_str().unwrap(), "add", "."]))?;
        run(Command::new("git")
            .args(["-C", repo_dir.path().to_str().unwrap(), "commit", "-m", "init"]))?;

        let branch = "feature/git-worktree-test";
        run(Command::new("git").args([
            "-C",
            repo_dir.path().to_str().unwrap(),
            "worktree",
            "add",
            "-b",
            branch,
            worktree_dir.path().to_str().unwrap(),
        ]))?;

        // Run activate_workspace from inside the repo so git worktree list works.
        let original_dir = std::env::current_dir()?;
        std::env::set_current_dir(repo_dir.path())?;

        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;
        let result = activate_workspace(&ship_dir, branch);

        std::env::set_current_dir(original_dir)?;

        let workspace = result?;
        assert_eq!(
            workspace.worktree_path.as_deref(),
            Some(worktree_dir.path().to_str().unwrap()),
            "activate_workspace should use real git worktree path, not a slug-derived fallback"
        );
        assert!(workspace.is_worktree);
        Ok(())
    }
}
