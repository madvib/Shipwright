use std::path::PathBuf;

/// Read the worktree base directory from `~/.ship/config.toml [worktrees] dir`.
/// Falls back to `~/dev/<project>-worktrees/` when the setting is absent.
pub fn configured_worktree_dir(project_root: &std::path::Path) -> PathBuf {
    let home = std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_default();
    configured_worktree_dir_impl(project_root, &home)
}

/// Testable inner implementation that accepts an explicit home path.
pub fn configured_worktree_dir_impl(
    project_root: &std::path::Path,
    home: &std::path::Path,
) -> PathBuf {
    let config_path = home.join(".ship").join("config.toml");
    if let Ok(content) = std::fs::read_to_string(&config_path) {
        let mut in_worktrees = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed == "[worktrees]" {
                in_worktrees = true;
            } else if trimmed.starts_with('[') {
                in_worktrees = false;
            } else if in_worktrees
                && let Some(rest) = trimmed.strip_prefix("dir")
                && let Some(val) = rest.trim().strip_prefix('=')
            {
                let dir = val.trim().trim_matches('"');
                if !dir.is_empty() {
                    let expanded = if let Some(rest) = dir.strip_prefix("~/") {
                        home.join(rest)
                    } else {
                        std::path::PathBuf::from(dir)
                    };
                    return expanded;
                }
            }
        }
    }
    // Fallback: ~/dev/<project>-worktrees/
    let project_name = project_root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "project".to_string());
    home.join("dev").join(format!("{}-worktrees", project_name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_absent_falls_back_to_project_name() {
        let tmp = tempfile::tempdir().unwrap();
        let project_root = std::path::PathBuf::from("/home/user/my-project");
        let result = configured_worktree_dir_impl(&project_root, tmp.path());
        assert!(
            result.to_string_lossy().ends_with("my-project-worktrees"),
            "expected fallback ending in my-project-worktrees, got {}",
            result.display()
        );
    }

    #[test]
    fn config_present_with_absolute_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        std::fs::write(
            ship_dir.join("config.toml"),
            "[worktrees]\ndir = \"/opt/worktrees\"\n",
        )
        .unwrap();
        let project_root = std::path::PathBuf::from("/home/user/myproject");
        let result = configured_worktree_dir_impl(&project_root, tmp.path());
        assert_eq!(result, std::path::PathBuf::from("/opt/worktrees"));
    }

    #[test]
    fn config_present_with_tilde_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        std::fs::write(
            ship_dir.join("config.toml"),
            "[worktrees]\ndir = \"~/dev/worktrees\"\n",
        )
        .unwrap();
        let project_root = std::path::PathBuf::from("/home/user/myproject");
        let result = configured_worktree_dir_impl(&project_root, tmp.path());
        let expected = tmp.path().join("dev").join("worktrees");
        assert_eq!(result, expected);
    }

    #[test]
    fn config_present_without_worktrees_section_falls_back() {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        std::fs::write(
            ship_dir.join("config.toml"),
            "[identity]\nname = \"Alice\"\n",
        )
        .unwrap();
        let project_root = std::path::PathBuf::from("/home/user/cool-project");
        let result = configured_worktree_dir_impl(&project_root, tmp.path());
        assert!(
            result.to_string_lossy().ends_with("cool-project-worktrees"),
            "expected fallback, got {}",
            result.display()
        );
    }
}
