use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Write `content` to `path` atomically by writing to a sibling temp file first,
/// then renaming into place. On POSIX, rename(2) is atomic; on Windows it uses
/// MoveFileExW which replaces the destination atomically.
///
/// This prevents a crash mid-write from leaving a zero-byte or partial file.
pub fn write_atomic(path: &Path, content: impl AsRef<[u8]>) -> Result<()> {
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();
    let tmp_path = dir.join(format!(".tmp-{}", file_name));

    fs::write(&tmp_path, content.as_ref())
        .with_context(|| format!("Failed to write temp file: {}", tmp_path.display()))?;
    fs::rename(&tmp_path, path)
        .with_context(|| format!("Failed to rename temp file to: {}", path.display()))?;
    Ok(())
}
