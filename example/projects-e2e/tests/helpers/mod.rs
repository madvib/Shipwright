use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

/// Simulated existing-project file trees for testing `ship init` on non-empty directories.
/// Each entry is (relative path, content).
pub const EXISTING_JS_PROJECT: &[(&str, &str)] = &[
    (
        "package.json",
        r#"{"name":"my-app","version":"0.1.0","scripts":{"dev":"next dev","build":"next build"}}"#,
    ),
    (
        "src/index.js",
        "import React from 'react';\nexport default function App() { return <div>Hello</div>; }\n",
    ),
    (
        "src/components/Button.js",
        "export const Button = ({children}) => <button>{children}</button>;\n",
    ),
    (
        "public/index.html",
        "<!DOCTYPE html><html><body><div id=\"root\"></div></body></html>\n",
    ),
    (".gitignore", "node_modules/\n.env\n.next/\nbuild/\n"),
    ("README.md", "# My App\n\nA sample project.\n"),
];

pub const EXISTING_RUST_PROJECT: &[(&str, &str)] = &[
    (
        "Cargo.toml",
        "[package]\nname = \"my-app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    ),
    (
        "src/main.rs",
        "fn main() { println!(\"Hello, world!\"); }\n",
    ),
    (".gitignore", "target/\n"),
    ("README.md", "# My App\n"),
];

/// A temporary project with a real .ship/ directory and optional git repo.
pub struct TestProject {
    pub dir: TempDir,
    pub ship_dir: PathBuf,
    pub global_dir: PathBuf,
}

/// A git worktree checked out from a TestProject.
pub struct TestWorktree {
    pub path: PathBuf,
    pub ship_dir: PathBuf,
    pub global_dir: PathBuf,
}

impl TestProject {
    fn shared_global_dir() -> Result<PathBuf> {
        let global_dir = runtime::project::get_global_dir()?;
        std::fs::create_dir_all(&global_dir)?;
        Ok(global_dir)
    }

    fn init_with_cli(base_dir: &Path, global_dir: &Path) -> Result<PathBuf> {
        let bin = std::env::var("SHIP_BIN").unwrap_or_else(|_| ship_bin_path());
        let out = Command::new(bin)
            .args(["init", "."])
            .current_dir(base_dir)
            .env("SHIP_GLOBAL_DIR", global_dir)
            .output()?;
        if !out.status.success() {
            anyhow::bail!(
                "ship init failed\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
        }
        Ok(base_dir.join(".ship"))
    }

    /// Create a new temp dir, run `ship init`, return the project.
    pub fn new() -> Result<Self> {
        let dir = TempDir::new()?;
        let global_dir = Self::shared_global_dir()?;
        let ship_dir = Self::init_with_cli(dir.path(), &global_dir)?;
        Ok(Self {
            dir,
            ship_dir,
            global_dir,
        })
    }

    /// Create a temp dir with pre-existing files (simulating an existing project),
    /// then run `ship init`. Files are created before init runs.
    pub fn with_existing_files(files: &[(&str, &str)]) -> Result<Self> {
        let dir = TempDir::new()?;
        let global_dir = Self::shared_global_dir()?;
        for (rel, content) in files {
            let path = dir.path().join(rel);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, content)?;
        }
        let ship_dir = Self::init_with_cli(dir.path(), &global_dir)?;
        Ok(Self {
            dir,
            ship_dir,
            global_dir,
        })
    }

    /// Create a temp dir with a real git repo, pre-existing files, and run `ship init`.
    pub fn with_git_and_files(files: &[(&str, &str)]) -> Result<Self> {
        let dir = TempDir::new()?;
        let global_dir = Self::shared_global_dir()?;
        let root = dir.path();
        for (rel, content) in files {
            let path = root.join(rel);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, content)?;
        }
        Self::init_git(root)?;
        let ship_dir = Self::init_with_cli(root, &global_dir)?;
        Ok(Self {
            dir,
            ship_dir,
            global_dir,
        })
    }

    /// Create a temp dir with a real git repo and run `ship init`.
    pub fn with_git() -> Result<Self> {
        let dir = TempDir::new()?;
        let global_dir = Self::shared_global_dir()?;
        let root = dir.path();
        Self::init_git(root)?;
        let ship_dir = Self::init_with_cli(root, &global_dir)?;
        Ok(Self {
            dir,
            ship_dir,
            global_dir,
        })
    }

    fn init_git(root: &Path) -> Result<()> {
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
        Ok(())
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
            .env("SHIP_DIR", &self.ship_dir)
            .env("SHIP_GLOBAL_DIR", &self.global_dir);
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
        let bin = std::env::var("SHIP_BIN").unwrap_or_else(|_| ship_bin_path());
        let mut path_env = std::env::var("PATH").unwrap_or_default();
        if let Some(target_dir) = std::path::Path::new(&bin).parent() {
            path_env = format!("{}:{}", target_dir.display(), path_env);
        }
        Ok(Command::new("git")
            .args(["checkout", branch])
            .env("PATH", path_env)
            .current_dir(self.dir.path())
            .output()?)
    }

    /// Git create and checkout new branch.
    pub fn checkout_new(&self, branch: &str) -> Result<Output> {
        let bin = std::env::var("SHIP_BIN").unwrap_or_else(|_| ship_bin_path());
        let mut path_env = std::env::var("PATH").unwrap_or_default();
        if let Some(target_dir) = std::path::Path::new(&bin).parent() {
            path_env = format!("{}:{}", target_dir.display(), path_env);
        }

        Ok(Command::new("git")
            .args(["checkout", "-b", branch])
            .env("PATH", path_env)
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

    /// Add a git worktree inside the project's temp dir and return a TestWorktree.
    /// The branch must already exist. The worktree's .ship/ is resolved via SHIP_DIR.
    pub fn add_worktree(&self, branch: &str) -> Result<TestWorktree> {
        // Place worktrees inside the project's unique temp dir to avoid cross-test conflicts
        // when tests run in parallel (multiple tests may use the same branch name).
        let worktree_path = self
            .root()
            .join(".worktrees")
            .join(branch.replace('/', "-"));
        // Note: git worktree add creates the directory itself — do not pre-create it.
        let out = Command::new("git")
            .args(["worktree", "add", worktree_path.to_str().unwrap(), branch])
            .current_dir(self.root())
            .output()?;
        if !out.status.success() {
            anyhow::bail!(
                "git worktree add {} failed: {}",
                branch,
                String::from_utf8_lossy(&out.stderr)
            );
        }
        Ok(TestWorktree {
            ship_dir: self.ship_dir.clone(),
            path: worktree_path,
            global_dir: self.global_dir.clone(),
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
        let content =
            std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("{} not readable", name));
        assert!(
            content.contains(needle),
            "{} should contain {:?}\n--- content ---\n{}",
            name,
            needle,
            content
        );
    }

    /// Assert a file at project root does NOT contain a substring.
    pub fn assert_root_file_not_contains(&self, name: &str, needle: &str) {
        let path = self.root().join(name);
        let content =
            std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("{} not readable", name));
        assert!(
            !content.contains(needle),
            "{} should NOT contain {:?}\n--- content ---\n{}",
            name,
            needle,
            content
        );
    }

    /// Read a file at project root.
    pub fn read_root_file(&self, name: &str) -> String {
        std::fs::read_to_string(self.root().join(name))
            .unwrap_or_else(|_| panic!("{} not readable", name))
    }

    /// Read a file under .ship/.
    pub fn read_ship_file(&self, rel: &str) -> String {
        std::fs::read_to_string(self.ship_dir.join(rel))
            .unwrap_or_else(|_| panic!(".ship/{} not readable", rel))
    }

    /// Write a file at project root (for test setup).
    pub fn write_root_file(&self, name: &str, content: &str) {
        let path = self.root().join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, content).unwrap();
    }

    /// Stage a specific file in git. Returns exit code.
    pub fn git_stage(&self, rel: &str) -> bool {
        Command::new("git")
            .args(["add", rel])
            .current_dir(self.root())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Attempt to commit staged changes. Returns (success, stderr).
    pub fn git_commit(&self, msg: &str) -> (bool, String) {
        let out = Command::new("git")
            .args(["commit", "-m", msg])
            .current_dir(self.root())
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
        (out.status.success(), stderr)
    }

    /// Return the current git branch name.
    pub fn current_branch(&self) -> String {
        let out = Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(self.root())
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    /// Return staged file list.
    pub fn git_staged_files(&self) -> Vec<String> {
        let out = Command::new("git")
            .args(["diff", "--cached", "--name-only"])
            .current_dir(self.root())
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(str::to_string)
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Create a release branch off the current branch.
    pub fn create_release_branch(&self, name: &str) -> Result<()> {
        let out = Command::new("git")
            .args(["checkout", "-b", name])
            .current_dir(self.root())
            .output()?;
        if !out.status.success() {
            anyhow::bail!(
                "git checkout -b {} failed: {}",
                name,
                String::from_utf8_lossy(&out.stderr)
            );
        }
        Ok(())
    }

    /// Create a feature branch off the current branch.
    pub fn create_feature_branch(&self, name: &str) -> Result<()> {
        self.create_release_branch(name)
    }

    pub fn install_hooks_and_hooks(&self) -> Result<()> {
        ship_module_git::install_hooks(&self.root().join(".git"))?;
        ship_module_git::write_root_gitignore(self.root())?;
        Ok(())
    }
}

impl TestWorktree {
    /// Run the ship CLI from the worktree directory with SHIP_DIR set to the main .ship/.
    pub fn cli(&self, args: &[&str]) -> Command {
        let bin = std::env::var("SHIP_BIN").unwrap_or_else(|_| ship_bin_path());
        let mut cmd = Command::new(bin);
        cmd.current_dir(&self.path)
            .env("SHIP_DIR", &self.ship_dir)
            .env("SHIP_GLOBAL_DIR", &self.global_dir);
        cmd.args(args);
        cmd
    }

    pub fn root(&self) -> &Path {
        &self.path
    }

    pub fn assert_file(&self, name: &str) {
        let path = self.path.join(name);
        assert!(path.exists(), "worktree/{} should exist but doesn't", name);
    }

    pub fn assert_no_file(&self, name: &str) {
        let path = self.path.join(name);
        assert!(
            !path.exists(),
            "worktree/{} should not exist but does",
            name
        );
    }

    pub fn assert_file_contains(&self, name: &str, needle: &str) {
        let path = self.path.join(name);
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("worktree/{} not readable", name));
        assert!(
            content.contains(needle),
            "worktree/{} should contain {:?}\n--- content ---\n{}",
            name,
            needle,
            content
        );
    }

    pub fn current_branch(&self) -> String {
        let out = Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(&self.path)
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).trim().to_string()
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

// ── Migration Primitives Shims ───────────────────────────────────────────────
// These shims preserve the old `runtime` interface (returning PathBuf) for tests
// without needing to rewrite dozens of existing E2E assertions.

pub fn create_feature(
    ship_dir: PathBuf,
    title: &str,
    body: &str,
    release_id: Option<&str>,
    spec_id: Option<&str>,
    branch: Option<&str>,
) -> Result<(String, PathBuf)> {
    let entry =
        ship_module_project::create_feature(&ship_dir, title, body, release_id, spec_id, branch)?;
    Ok((entry.id, PathBuf::from(entry.path)))
}

pub fn create_release(ship_dir: PathBuf, version: &str, notes: &str) -> Result<PathBuf> {
    let entry = ship_module_project::create_release(&ship_dir, version, notes)?;
    Ok(PathBuf::from(entry.path))
}

pub fn init_project(base_dir: PathBuf) -> Result<PathBuf> {
    ship_module_project::init_project(base_dir)
}

pub fn create_spec(ship_dir: PathBuf, title: &str, body: &str, status: &str) -> Result<PathBuf> {
    let branch = format!(
        "feature/spec-{}",
        runtime::project::sanitize_file_name(title)
    );
    runtime::create_workspace(
        &ship_dir,
        runtime::CreateWorkspaceRequest {
            branch,
            status: Some(runtime::WorkspaceStatus::Active),
            ..Default::default()
        },
    )?;
    let entry = ship_module_project::create_spec(&ship_dir, title, body, None)?;
    if status != "draft" {
        anyhow::bail!("unsupported e2e helper status for create_spec: {}", status);
    }
    Ok(PathBuf::from(entry.path))
}

pub fn create_issue(
    ship_dir: PathBuf,
    title: &str,
    description: &str,
    status: &str,
) -> Result<PathBuf> {
    let status = status.parse::<ship_module_project::IssueStatus>()?;
    let entry = ship_module_project::create_issue(
        &ship_dir,
        title,
        description,
        status,
        None,
        None,
        None,
        None,
    )?;
    Ok(PathBuf::from(entry.path))
}

pub fn move_issue(
    ship_dir: PathBuf,
    path: PathBuf,
    _from_status: &str,
    to_status: &str,
) -> Result<PathBuf> {
    let new_status = to_status.parse::<ship_module_project::IssueStatus>()?;
    let reference = path
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or_else(|| anyhow::anyhow!("invalid issue path: {}", path.display()))?;
    let entry = ship_module_project::move_issue(&ship_dir, reference, new_status)?;
    Ok(PathBuf::from(entry.path))
}
