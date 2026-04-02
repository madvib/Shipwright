use anyhow::{Context, Result};
use std::path::PathBuf;

pub enum DaemonCommand {
    Start,
    Stop,
    Status,
}

const SHIPD_PORT: u16 = 51742;

fn ship_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("cannot determine home directory")?;
    Ok(home.join(".ship"))
}

fn pid_file() -> Result<PathBuf> {
    Ok(ship_dir()?.join("network.pid"))
}

fn log_file() -> Result<PathBuf> {
    Ok(ship_dir()?.join("shipd.log"))
}

/// Read a PID from the pid file. Returns None if the file does not exist.
pub fn read_pid(pid_path: &std::path::Path) -> Result<Option<u32>> {
    if !pid_path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(pid_path)
        .with_context(|| format!("cannot read {}", pid_path.display()))?;
    let pid: u32 = content
        .trim()
        .parse()
        .with_context(|| format!("invalid PID in {}", pid_path.display()))?;
    Ok(Some(pid))
}

/// Write a PID to the pid file.
pub fn write_pid(pid_path: &std::path::Path, pid: u32) -> Result<()> {
    if let Some(parent) = pid_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("cannot create directory {}", parent.display()))?;
    }
    std::fs::write(pid_path, format!("{}\n", pid))
        .with_context(|| format!("cannot write {}", pid_path.display()))?;
    Ok(())
}

/// Check whether a process with the given PID is alive by sending signal 0.
fn is_process_alive(pid: u32) -> bool {
    let status = std::process::Command::new("kill")
        .args(["-0", &pid.to_string()])
        .status();
    matches!(status, Ok(s) if s.success())
}

/// Locate the shipd binary. Looks for a sibling of the current executable first,
/// then falls back to PATH via `which`.
fn find_shipd() -> Result<PathBuf> {
    // Try sibling of the running ship binary.
    if let Ok(exe) = std::env::current_exe() {
        let sibling = exe.parent().unwrap_or(std::path::Path::new(".")).join("shipd");
        if sibling.exists() {
            return Ok(sibling);
        }
    }
    // Fall back to PATH.
    let output = std::process::Command::new("which")
        .arg("shipd")
        .output()
        .context("cannot invoke `which shipd`")?;
    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            return Ok(PathBuf::from(path));
        }
    }
    anyhow::bail!(
        "shipd binary not found. Make sure it is installed alongside the ship CLI or on PATH."
    )
}

pub fn run_daemon(cmd: DaemonCommand) -> Result<()> {
    match cmd {
        DaemonCommand::Start => daemon_start(),
        DaemonCommand::Stop => daemon_stop(),
        DaemonCommand::Status => daemon_status(),
    }
}

fn daemon_start() -> Result<()> {
    let pid_path = pid_file()?;
    let log_path = log_file()?;

    // Check if already running.
    if let Some(pid) = read_pid(&pid_path)? {
        if is_process_alive(pid) {
            println!("shipd already running (pid {}, port {})", pid, SHIPD_PORT);
            return Ok(());
        }
        // Stale PID file — remove it.
        let _ = std::fs::remove_file(&pid_path);
    }

    let shipd = find_shipd()?;

    // Ensure ~/.ship/ exists.
    std::fs::create_dir_all(ship_dir()?)
        .context("cannot create ~/.ship/ directory")?;

    let log_handle = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .with_context(|| format!("cannot open log file {}", log_path.display()))?;

    let stdout = log_handle.try_clone().context("cannot clone log file handle")?;
    let stderr = log_handle;

    let child = std::process::Command::new(&shipd)
        .args(["--port", &SHIPD_PORT.to_string()])
        .stdout(stdout)
        .stderr(stderr)
        .stdin(std::process::Stdio::null())
        .spawn()
        .with_context(|| format!("cannot spawn {}", shipd.display()))?;

    let pid = child.id();
    write_pid(&pid_path, pid)?;

    println!(
        "shipd started (pid {}, port {}, log {})",
        pid,
        SHIPD_PORT,
        log_path.display()
    );
    Ok(())
}

fn daemon_stop() -> Result<()> {
    let pid_path = pid_file()?;

    let pid = match read_pid(&pid_path)? {
        Some(p) => p,
        None => {
            println!("shipd not running (no PID file)");
            return Ok(());
        }
    };

    if !is_process_alive(pid) {
        println!("shipd not running (stale PID {})", pid);
        let _ = std::fs::remove_file(&pid_path);
        return Ok(());
    }

    let status = std::process::Command::new("kill")
        .args(["-TERM", &pid.to_string()])
        .status()
        .context("cannot invoke `kill`")?;

    if !status.success() {
        anyhow::bail!("failed to send SIGTERM to pid {}", pid);
    }

    let _ = std::fs::remove_file(&pid_path);
    println!("shipd stopped (pid {})", pid);
    Ok(())
}

fn daemon_status() -> Result<()> {
    let pid_path = pid_file()?;

    match read_pid(&pid_path)? {
        Some(pid) if is_process_alive(pid) => {
            println!("shipd running (pid {}, port {})", pid, SHIPD_PORT);
        }
        Some(pid) => {
            println!("shipd not running (stale pid {})", pid);
        }
        None => {
            println!("shipd not running");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_write_and_read_pid() {
        let dir = TempDir::new().unwrap();
        let pid_path = dir.path().join("network.pid");

        write_pid(&pid_path, 12345).unwrap();
        assert!(pid_path.exists());

        let pid = read_pid(&pid_path).unwrap();
        assert_eq!(pid, Some(12345));
    }

    #[test]
    fn test_read_pid_missing_file() {
        let dir = TempDir::new().unwrap();
        let pid_path = dir.path().join("network.pid");
        let pid = read_pid(&pid_path).unwrap();
        assert_eq!(pid, None);
    }

    #[test]
    fn test_read_pid_invalid_content() {
        let dir = TempDir::new().unwrap();
        let pid_path = dir.path().join("network.pid");
        std::fs::write(&pid_path, "not-a-number\n").unwrap();
        assert!(read_pid(&pid_path).is_err());
    }

    #[test]
    fn test_write_pid_creates_parent_dirs() {
        let dir = TempDir::new().unwrap();
        let pid_path = dir.path().join("nested").join("dirs").join("network.pid");
        write_pid(&pid_path, 99).unwrap();
        let pid = read_pid(&pid_path).unwrap();
        assert_eq!(pid, Some(99));
    }
}
