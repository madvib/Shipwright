use super::*;

#[test]
fn validate_path_rejects_traversal() {
    assert!(validate_path("../etc/passwd").is_err());
    assert!(validate_path("foo/../../bar").is_err());
    assert!(validate_path("..").is_err());
}

#[test]
fn validate_path_rejects_absolute() {
    assert!(validate_path("/etc/passwd").is_err());
    assert!(validate_path("\\Windows\\System32").is_err());
}

#[test]
fn validate_path_rejects_empty() {
    assert!(validate_path("").is_err());
}

#[test]
fn validate_path_accepts_valid() {
    assert!(validate_path("canvas.html").is_ok());
    assert!(validate_path("screenshots/issue-001.png").is_ok());
    assert!(validate_path("deep/nested/file.json").is_ok());
}

#[test]
fn file_type_classification() {
    assert_eq!(file_type_from_extension("test.html"), "html");
    assert_eq!(file_type_from_extension("test.htm"), "html");
    assert_eq!(file_type_from_extension("test.md"), "md");
    assert_eq!(file_type_from_extension("test.json"), "json");
    assert_eq!(file_type_from_extension("test.png"), "image");
    assert_eq!(file_type_from_extension("test.jpg"), "image");
    assert_eq!(file_type_from_extension("test.jpeg"), "image");
    assert_eq!(file_type_from_extension("test.gif"), "image");
    assert_eq!(file_type_from_extension("test.webp"), "image");
    assert_eq!(file_type_from_extension("test.rs"), "other");
    assert_eq!(file_type_from_extension("noext"), "other");
}

#[test]
fn list_empty_session_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let result = list_session_files(tmp.path());
    assert_eq!(result, "[]");
}

#[test]
fn list_with_files() {
    let tmp = tempfile::tempdir().unwrap();
    let session = tmp.path().join(SESSION_DIR);
    std::fs::create_dir_all(session.join("screenshots")).unwrap();
    std::fs::write(session.join("canvas.html"), "<html></html>").unwrap();
    std::fs::write(session.join("screenshots/s.png"), &[0u8; 10]).unwrap();

    let result = list_session_files(tmp.path());
    let entries: Vec<SessionFileEntry> = serde_json::from_str(&result).unwrap();
    assert_eq!(entries.len(), 2);

    let html = entries.iter().find(|e| e.path == "canvas.html").unwrap();
    assert_eq!(html.file_type, "html");

    let img = entries
        .iter()
        .find(|e| e.path == "screenshots/s.png")
        .unwrap();
    assert_eq!(img.file_type, "image");
    assert_eq!(img.size, 10);
}

#[test]
fn read_text_file() {
    let tmp = tempfile::tempdir().unwrap();
    let session = tmp.path().join(SESSION_DIR);
    std::fs::create_dir_all(&session).unwrap();
    std::fs::write(session.join("notes.md"), "# Notes").unwrap();

    let req = ReadSessionFileRequest {
        path: "notes.md".into(),
    };
    let result = read_session_file(tmp.path(), req);
    assert_eq!(result, "# Notes");
}

#[test]
fn read_binary_file_returns_data_uri() {
    let tmp = tempfile::tempdir().unwrap();
    let session = tmp.path().join(SESSION_DIR);
    std::fs::create_dir_all(&session).unwrap();
    let bytes = vec![0x89, 0x50, 0x4E, 0x47]; // PNG magic bytes
    std::fs::write(session.join("img.png"), &bytes).unwrap();

    let req = ReadSessionFileRequest {
        path: "img.png".into(),
    };
    let result = read_session_file(tmp.path(), req);
    assert!(result.starts_with("data:image/png;base64,"));
}

#[test]
fn read_nonexistent_file_returns_error() {
    let tmp = tempfile::tempdir().unwrap();
    let req = ReadSessionFileRequest {
        path: "missing.txt".into(),
    };
    let result = read_session_file(tmp.path(), req);
    assert!(result.starts_with("Error:"));
}

#[test]
fn read_rejects_path_traversal() {
    let tmp = tempfile::tempdir().unwrap();
    let req = ReadSessionFileRequest {
        path: "../secret.txt".into(),
    };
    let result = read_session_file(tmp.path(), req);
    assert!(result.starts_with("Error:"));
    assert!(result.contains("traversal"));
}

#[test]
fn write_creates_text_file() {
    let tmp = tempfile::tempdir().unwrap();
    let req = WriteSessionFileRequest {
        path: "output.html".into(),
        content: "<h1>Hello</h1>".into(),
    };
    let result = write_session_file(tmp.path(), req);
    assert_eq!(result, "Wrote output.html");

    let content =
        std::fs::read_to_string(tmp.path().join(SESSION_DIR).join("output.html")).unwrap();
    assert_eq!(content, "<h1>Hello</h1>");
}

#[test]
fn write_creates_parent_directories() {
    let tmp = tempfile::tempdir().unwrap();
    let req = WriteSessionFileRequest {
        path: "deep/nested/file.json".into(),
        content: "{}".into(),
    };
    let result = write_session_file(tmp.path(), req);
    assert_eq!(result, "Wrote deep/nested/file.json");
    assert!(tmp
        .path()
        .join(SESSION_DIR)
        .join("deep/nested/file.json")
        .exists());
}

#[test]
fn write_binary_file_from_base64() {
    let tmp = tempfile::tempdir().unwrap();
    let bytes = vec![0x89, 0x50, 0x4E, 0x47];
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    let req = WriteSessionFileRequest {
        path: "test.png".into(),
        content: b64,
    };
    let result = write_session_file(tmp.path(), req);
    assert_eq!(result, "Wrote test.png");

    let written = std::fs::read(tmp.path().join(SESSION_DIR).join("test.png")).unwrap();
    assert_eq!(written, bytes);
}

#[test]
fn write_binary_file_from_data_uri() {
    let tmp = tempfile::tempdir().unwrap();
    let bytes = vec![0xFF, 0xD8, 0xFF];
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    let data_uri = format!("data:image/jpeg;base64,{}", b64);
    let req = WriteSessionFileRequest {
        path: "photo.jpg".into(),
        content: data_uri,
    };
    let result = write_session_file(tmp.path(), req);
    assert_eq!(result, "Wrote photo.jpg");

    let written = std::fs::read(tmp.path().join(SESSION_DIR).join("photo.jpg")).unwrap();
    assert_eq!(written, bytes);
}

#[test]
fn write_rejects_path_traversal() {
    let tmp = tempfile::tempdir().unwrap();
    let req = WriteSessionFileRequest {
        path: "../escape.txt".into(),
        content: "evil".into(),
    };
    let result = write_session_file(tmp.path(), req);
    assert!(result.starts_with("Error:"));
    assert!(result.contains("traversal"));
}
