use anyhow::{Context, Result};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Write `content` to `path` atomically by writing to a sibling temp file first,
/// then renaming into place. On POSIX, rename(2) is atomic; on Windows it uses
/// MoveFileExW which replaces the destination atomically.
///
/// This prevents a crash mid-write from leaving a zero-byte or partial file.
pub fn write_atomic(path: &Path, content: impl AsRef<[u8]>) -> Result<()> {
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(dir)
        .with_context(|| format!("Failed to create parent dir: {}", dir.display()))?;
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();
    let pid = std::process::id();
    let bytes = content.as_ref();

    for attempt in 0..16u8 {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let tmp_path = dir.join(format!(".tmp-{}-{}-{}-{}", file_name, pid, nonce, attempt));

        let mut file = match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp_path)
        {
            Ok(file) => file,
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => {
                return Err(err).with_context(|| {
                    format!("Failed to create temp file: {}", tmp_path.display())
                });
            }
        };

        file.write_all(bytes)
            .with_context(|| format!("Failed to write temp file: {}", tmp_path.display()))?;
        drop(file);

        fs::rename(&tmp_path, path)
            .with_context(|| format!("Failed to rename temp file to: {}", path.display()))?;
        return Ok(());
    }

    Err(anyhow::anyhow!(
        "Failed to allocate unique temp file for atomic write: {}",
        path.display()
    ))
}
