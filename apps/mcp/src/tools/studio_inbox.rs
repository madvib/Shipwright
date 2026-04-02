use std::path::{Path, PathBuf};

/// Write an event payload to `.ship-session/inbox/<unix_ms>.json`.
///
/// The file contains the merged payload fields plus `type`, `timestamp`, and
/// `event_id`. If a file for the current millisecond already exists a `_1`,
/// `_2`, … suffix is appended to avoid collisions.
///
/// Returns the path of the file written.
pub fn write_inbox_file(
    project_dir: &Path,
    event_type: &str,
    payload: &serde_json::Value,
    event_id: &str,
) -> anyhow::Result<PathBuf> {
    let inbox_dir = project_dir.join(".ship-session").join("inbox");
    std::fs::create_dir_all(&inbox_dir)?;

    let ts_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis();

    let mut body = match payload {
        serde_json::Value::Object(map) => map.clone(),
        other => {
            let mut m = serde_json::Map::new();
            m.insert("payload".to_string(), other.clone());
            m
        }
    };
    body.insert("type".to_string(), serde_json::Value::String(event_type.to_string()));
    body.insert(
        "timestamp".to_string(),
        serde_json::Value::Number(serde_json::Number::from(ts_ms as u64)),
    );
    body.insert(
        "event_id".to_string(),
        serde_json::Value::String(event_id.to_string()),
    );
    let json = serde_json::to_string(&body)?;

    // Find a filename that doesn't already exist.
    let base = ts_ms.to_string();
    let candidate = inbox_dir.join(format!("{base}.json"));
    let path = if !candidate.exists() {
        candidate
    } else {
        let mut n = 1u32;
        loop {
            let p = inbox_dir.join(format!("{base}_{n}.json"));
            if !p.exists() {
                break p;
            }
            n += 1;
        }
    };

    std::fs::write(&path, json)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emit_studio_event_writes_inbox_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let project = dir.path();

        let payload = serde_json::json!({"message": "hello", "severity": "info"});

        let path1 = write_inbox_file(project, "studio.message.visual", &payload, "evt-001")
            .expect("first write");
        let path2 = write_inbox_file(project, "studio.message.visual", &payload, "evt-002")
            .expect("second write");

        // Both files must exist.
        assert!(path1.exists(), "first file missing");
        assert!(path2.exists(), "second file missing");

        // Paths must be distinct — no overwrite.
        assert_ne!(path1, path2, "second write must not overwrite the first");

        // Verify content of first file.
        let raw = std::fs::read_to_string(&path1).expect("read first");
        let v: serde_json::Value = serde_json::from_str(&raw).expect("parse first");
        assert_eq!(v["type"], "studio.message.visual");
        assert_eq!(v["event_id"], "evt-001");
        assert_eq!(v["message"], "hello");
        assert!(v["timestamp"].is_number(), "timestamp must be a number");

        // Verify content of second file.
        let raw2 = std::fs::read_to_string(&path2).expect("read second");
        let v2: serde_json::Value = serde_json::from_str(&raw2).expect("parse second");
        assert_eq!(v2["event_id"], "evt-002");
        assert_eq!(v2["message"], "hello");
    }

    #[test]
    fn write_inbox_file_creates_inbox_dir() {
        let dir = tempfile::tempdir().expect("tempdir");
        let project = dir.path();

        // Inbox dir must not exist yet.
        assert!(!project.join(".ship-session").join("inbox").exists());

        write_inbox_file(project, "studio.test", &serde_json::json!({}), "evt-x")
            .expect("write");

        assert!(project.join(".ship-session").join("inbox").is_dir());
    }

    #[test]
    fn write_inbox_file_non_object_payload_wrapped() {
        let dir = tempfile::tempdir().expect("tempdir");
        let payload = serde_json::json!("just a string");
        let path =
            write_inbox_file(dir.path(), "studio.x", &payload, "evt-y").expect("write");
        let raw = std::fs::read_to_string(path).expect("read");
        let v: serde_json::Value = serde_json::from_str(&raw).expect("parse");
        assert_eq!(v["payload"], "just a string");
        assert_eq!(v["type"], "studio.x");
    }
}
