use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

/// A temporary project with a real .ship/ directory and optional git repo.
pub struct TestProject {
    pub dir: TempDir,
    pub ship_dir: PathBuf,
}

/// A git worktree checked out from a TestProject.
pub struct TestWorktree {
    pub path: PathBuf,
    pub ship_dir: PathBuf,
}

impl TestProject {
    /// Create a new temp dir, run `ship init`, return the project.
    pub fn new() -> Result<Self> {
        let dir = TempDir::new()?;
        let ship_dir = runtime::init_project(dir.path().to_path_buf())?;
        Ok(Self { dir, ship_dir })
    }

    /// Create a temp dir with a real git repo and run `ship init`.
    pub fn with_git() -> Result<Self> {
        let dir = TempDir::new()?;
        let root = dir.path();
        Command::new("git")
            .args(["init", "-b", "main"])
            .current_dir(root)
            .output()?;
        Command::new("git")
            .args(["config", "user.email", "test@ship.dev"])
            .current_dir(root)
            .output()?;
        Command::new("git")
            .args(["config", "user.name", "Ship Test"])
            .current_dir(root)
            .output()?;
        let ship_dir = runtime::init_project(root.to_path_buf())?;
        Ok(Self { dir, ship_dir })
    }

    pub fn root(&self) -> &Path {
        self.dir.path()
    }

    /// Run the `ship` CLI binary against this project's directory.
    /// Returns the Command pre-configured; call `.output()` or `.status()`.
    pub fn cli(&self, args: &[&str]) -> Command {
        let bin = std::env::var("SHIP_BIN").unwrap_or_else(|_| ship_bin_path());
        let mut cmd = Command::new(bin);
        cmd.current_dir(self.dir.path())
            .env("SHIP_DIR", &self.ship_dir);
        cmd.args(args);
        cmd
    }

    pub fn cli_output(&self, args: &[&str]) -> Result<Output> {
        Ok(self.cli(args).output()?)
    }

    pub fn cli_stdout(&self, args: &[&str]) -> Result<String> {
        let out = self.cli_output(args)?;
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }

    /// Assert a file exists under .ship/
    pub fn assert_ship_file(&self, rel: &str) {
        let path = self.ship_dir.join(rel);
        assert!(path.exists(), ".ship/{} should exist but doesn't", rel);
    }

    /// Assert a file does NOT exist under .ship/
    pub fn assert_no_ship_file(&self, rel: &str) {
        let path = self.ship_dir.join(rel);
        assert!(!path.exists(), ".ship/{} should not exist but does", rel);
    }

    /// Assert a file under .ship/ contains a substring.
    pub fn assert_ship_file_contains(&self, rel: &str, needle: &str) {
        let path = self.ship_dir.join(rel);
        let content =
            std::fs::read_to_string(&path).unwrap_or_else(|_| panic!(".ship/{} not readable", rel));
        assert!(
            content.contains(needle),
            ".ship/{} should contain {:?}\n--- content ---\n{}",
            rel,
            needle,
            content
        );
    }

    /// Git checkout — fires hooks if installed.
    pub fn checkout(&self, branch: &str) -> Result<Output> {
        Ok(Command::new("git")
            .args(["checkout", branch])
            .current_dir(self.dir.path())
            .output()?)
    }

    /// Git create and checkout new branch.
    pub fn checkout_new(&self, branch: &str) -> Result<Output> {
        Ok(Command::new("git")
            .args(["checkout", "-b", branch])
            .current_dir(self.dir.path())
            .output()?)
    }

    /// Install git post-checkout hooks so real `git checkout` fires `ship git post-checkout`.
    pub fn install_hooks(&self) -> Result<()> {
        ship_module_git::install_hooks(&self.root().join(".git"))
    }

    /// Create an initial commit so branches can be created and worktrees can be added.
    pub fn initial_commit(&self) -> Result<Output> {
        Command::new("git")
            .args(["add", "-A"])
            .current_dir(self.root())
            .output()?;
        Ok(Command::new("git")
            .args(["commit", "--allow-empty", "-m", "init"])
            .current_dir(self.root())
            .output()?)
    }

    /// Add a git worktree at a sibling directory and return a TestWorktree.
    /// The branch must already exist. The worktree's .ship/ is resolved via SHIP_DIR.
    pub fn add_worktree(&self, branch: &str) -> Result<TestWorktree> {
        let worktree_path = self.root().parent().unwrap()
            .join(format!("worktree-{}", branch.replace('/', "-")));
        Command::new("git")
            .args(["worktree", "add", worktree_path.to_str().unwrap(), branch])
            .current_dir(self.root())
            .output()?;
        Ok(TestWorktree {
            ship_dir: self.ship_dir.clone(),
            path: worktree_path,
        })
    }

    /// Assert a file exists at project root (not .ship/).
    pub fn assert_root_file(&self, name: &str) {
        let path = self.root().join(name);
        assert!(path.exists(), "{} should exist at project root", name);
    }

    /// Assert a file does NOT exist at project root.
    pub fn assert_no_root_file(&self, name: &str) {
        let path = self.root().join(name);
        assert!(!path.exists(), "{} should not exist at project root", name);
    }

    /// Assert a file at project root contains a substring.
    pub fn assert_root_file_contains(&self, name: &str, needle: &str) {
        let path = self.root().join(name);
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("{} not readable", name));
        assert!(
            content.contains(needle),
            "{} should contain {:?}\n--- content ---\n{}",
            name, needle, content
        );
    }

    /// Assert a file at project root does NOT contain a substring.
    pub fn assert_root_file_not_contains(&self, name: &str, needle: &str) {
        let path = self.root().join(name);
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("{} not readable", name));
        assert!(
            !content.contains(needle),
            "{} should NOT contain {:?}\n--- content ---\n{}",
            name, needle, content
        );
    }
}

impl TestWorktree {
    /// Run the ship CLI from the worktree directory with SHIP_DIR set to the main .ship/.
    pub fn cli(&self, args: &[&str]) -> Command {
        let bin = std::env::var("SHIP_BIN").unwrap_or_else(|_| ship_bin_path());
        let mut cmd = Command::new(bin);
        cmd.current_dir(&self.path)
            .env("SHIP_DIR", &self.ship_dir);
        cmd.args(args);
        cmd
    }
}

fn ship_bin_path() -> String {
    // Walk up from the test binary to find the workspace target dir.
    let mut dir = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    // target/debug/deps → target/debug
    if dir.ends_with("deps") {
        dir = dir.parent().unwrap().to_path_buf();
    }
    dir.join("ship").to_string_lossy().to_string()
}
