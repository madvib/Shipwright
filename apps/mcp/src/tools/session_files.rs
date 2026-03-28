use std::path::Path;

use base64::Engine;
use serde::Serialize;

use crate::requests::{ReadSessionFileRequest, WriteSessionFileRequest};

const SESSION_DIR: &str = ".ship-session";

#[derive(Serialize)]
#[cfg_attr(test, derive(serde::Deserialize))]
struct SessionFileEntry {
    path: String,
    size: u64,
    modified: String,
    #[serde(rename = "type")]
    file_type: String,
}

/// Validate that a relative path does not escape the session directory.
fn validate_path(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("Path must not be empty".into());
    }
    if path.starts_with('/') || path.starts_with('\\') {
        return Err("Absolute paths are not allowed".into());
    }
    if path.contains("..") {
        return Err("Path traversal (..) is not allowed".into());
    }
    Ok(())
}

/// Derive a file type string from the file extension.
fn file_type_from_extension(path: &str) -> String {
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "html" | "htm" => "html".into(),
        "md" | "markdown" => "md".into(),
        "json" => "json".into(),
        "png" | "jpg" | "jpeg" | "gif" | "webp" => "image".into(),
        _ => "other".into(),
    }
}

fn is_binary_type(file_type: &str) -> bool {
    file_type == "image"
}

fn mime_from_extension(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    }
}

/// Recursively collect all files under a directory.
fn walk_dir(root: &Path, dir: &Path, entries: &mut Vec<SessionFileEntry>) {
    let Ok(read) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in read.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_dir(root, &path, entries);
        } else if path.is_file() {
            let Ok(rel) = path.strip_prefix(root) else {
                continue;
            };
            let rel_str = rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/");
            let meta = std::fs::metadata(&path);
            let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
            let modified = meta
                .ok()
                .and_then(|m| m.modified().ok())
                .map(|t| {
                    let dt: chrono::DateTime<chrono::Utc> = t.into();
                    dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
                })
                .unwrap_or_default();
            let file_type = file_type_from_extension(&rel_str);
            entries.push(SessionFileEntry {
                path: rel_str,
                size,
                modified,
                file_type,
            });
        }
    }
}

pub fn list_session_files(project_dir: &Path) -> String {
    let session_dir = project_dir.join(SESSION_DIR);
    if !session_dir.is_dir() {
        return "[]".into();
    }
    let mut entries = Vec::new();
    walk_dir(&session_dir, &session_dir, &mut entries);
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    serde_json::to_string(&entries).unwrap_or_else(|e| format!("Error: {}", e))
}

pub fn read_session_file(project_dir: &Path, req: ReadSessionFileRequest) -> String {
    if let Err(e) = validate_path(&req.path) {
        return format!("Error: {}", e);
    }
    let file_path = project_dir.join(SESSION_DIR).join(&req.path);
    if !file_path.is_file() {
        return format!("Error: file not found: {}", req.path);
    }
    let file_type = file_type_from_extension(&req.path);
    if is_binary_type(&file_type) {
        let bytes = match std::fs::read(&file_path) {
            Ok(b) => b,
            Err(e) => return format!("Error: {}", e),
        };
        let mime = mime_from_extension(&req.path);
        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
        format!("data:{};base64,{}", mime, b64)
    } else {
        match std::fs::read_to_string(&file_path) {
            Ok(content) => content,
            Err(e) => format!("Error: {}", e),
        }
    }
}

pub fn write_session_file(project_dir: &Path, req: WriteSessionFileRequest) -> String {
    if let Err(e) = validate_path(&req.path) {
        return format!("Error: {}", e);
    }
    let file_path = project_dir.join(SESSION_DIR).join(&req.path);
    if let Some(parent) = file_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return format!("Error creating directories: {}", e);
        }
    }
    let file_type = file_type_from_extension(&req.path);
    if is_binary_type(&file_type) {
        // Try to decode as data URI or raw base64
        let b64_data = req
            .content
            .find(";base64,")
            .map(|i| &req.content[i + 8..])
            .unwrap_or(&req.content);
        match base64::engine::general_purpose::STANDARD.decode(b64_data) {
            Ok(bytes) => match std::fs::write(&file_path, bytes) {
                Ok(()) => format!("Wrote {}", req.path),
                Err(e) => format!("Error writing file: {}", e),
            },
            Err(e) => format!("Error decoding base64: {}", e),
        }
    } else {
        match std::fs::write(&file_path, &req.content) {
            Ok(()) => format!("Wrote {}", req.path),
            Err(e) => format!("Error writing file: {}", e),
        }
    }
}

#[cfg(test)]
#[path = "session_files_tests.rs"]
mod tests;
