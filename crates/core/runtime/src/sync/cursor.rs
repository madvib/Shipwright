//! Cursor persistence for sync — tracks the last-synced event ID per scope.
//!
//! Stores cursors in the `kv_state` table under the "sync" namespace.
//! Key format: `cursor:platform:{project_id}` or `cursor:workspace:{workspace_id}`.

use anyhow::Result;
use serde_json::json;

const NAMESPACE: &str = "sync";

fn cursor_key(scope: &str) -> String {
    format!("cursor:{scope}")
}

/// Get the last-synced cursor for a scope.
///
/// Scope should be `platform:{project_id}` or `workspace:{workspace_id}`.
pub fn get_cursor(scope: &str) -> Result<Option<String>> {
    let val = crate::db::kv::get(NAMESPACE, &cursor_key(scope))?;
    Ok(val.and_then(|v| v.as_str().map(String::from)))
}

/// Set the last-synced cursor for a scope.
pub fn set_cursor(scope: &str, cursor: &str) -> Result<()> {
    crate::db::kv::set(NAMESPACE, &cursor_key(scope), &json!(cursor))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::ensure_db;
    use crate::project::init_project;
    use tempfile::tempdir;

    fn setup() -> tempfile::TempDir {
        let tmp = tempdir().unwrap();
        init_project(tmp.path().to_path_buf()).unwrap();
        ensure_db().unwrap();
        tmp
    }

    #[test]
    fn missing_cursor_returns_none() {
        let _tmp = setup();
        let cursor = get_cursor("platform:proj-999").unwrap();
        assert!(cursor.is_none());
    }

    #[test]
    fn set_and_get_roundtrip() {
        let _tmp = setup();
        set_cursor("platform:proj-1", "01J0000000000000000000000A").unwrap();
        let cursor = get_cursor("platform:proj-1").unwrap();
        assert_eq!(cursor.unwrap(), "01J0000000000000000000000A");
    }

    #[test]
    fn set_overwrites_previous() {
        let _tmp = setup();
        set_cursor("workspace:ws-1", "01J0000000000000000000000A").unwrap();
        set_cursor("workspace:ws-1", "01J0000000000000000000000B").unwrap();
        let cursor = get_cursor("workspace:ws-1").unwrap();
        assert_eq!(cursor.unwrap(), "01J0000000000000000000000B");
    }

    #[test]
    fn scopes_are_isolated() {
        let _tmp = setup();
        set_cursor("platform:proj-1", "AAA").unwrap();
        set_cursor("workspace:ws-1", "BBB").unwrap();
        assert_eq!(get_cursor("platform:proj-1").unwrap().unwrap(), "AAA");
        assert_eq!(get_cursor("workspace:ws-1").unwrap().unwrap(), "BBB");
    }
}
