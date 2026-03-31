use crate::resource_resolver::resolve_resource_uri;
use runtime::db::ensure_db;
use runtime::project::init_project;
use std::future;
use tempfile::tempdir;

fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = tempdir().unwrap();
    let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
    ensure_db().unwrap();
    (tmp, ship_dir)
}

async fn resolve(uri: &str, dir: &std::path::Path) -> Option<String> {
    resolve_resource_uri(uri, dir, future::ready(String::new())).await
}

// ── Spec resources (file-based) ───────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn specs_list_empty() {
    let (_tmp, ship_dir) = setup();
    let result = resolve("ship://specs", &ship_dir).await.unwrap();
    assert_eq!(result, "No specs found.");
}

#[tokio::test(flavor = "multi_thread")]
async fn specs_list_and_get_round_trip() {
    let (_tmp, ship_dir) = setup();
    let specs_path = runtime::project::specs_dir(&ship_dir);
    std::fs::create_dir_all(&specs_path).unwrap();
    std::fs::write(
        specs_path.join("design-v1.md"),
        "# Design V1\n\nSpec content here.",
    )
    .unwrap();

    let list = resolve("ship://specs", &ship_dir).await.unwrap();
    assert!(list.contains("design-v1"), "spec list: {list}");

    let detail = resolve("ship://specs/design-v1", &ship_dir).await.unwrap();
    assert!(detail.contains("# Design V1"));
    assert!(detail.contains("Spec content here."));
}

#[tokio::test(flavor = "multi_thread")]
async fn spec_get_nonexistent_returns_none() {
    let (_tmp, ship_dir) = setup();
    assert!(
        resolve("ship://specs/nonexistent", &ship_dir)
            .await
            .is_none()
    );
}

// ── ADR resources ─────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn adrs_resource_round_trip() {
    let (_tmp, ship_dir) = setup();
    let result = resolve("ship://adrs", &ship_dir).await.unwrap();
    assert_eq!(result, "No ADRs found.");

    runtime::db::adrs::create_adr("Test ADR", "context", "decision", "proposed").unwrap();
    let list = resolve("ship://adrs", &ship_dir).await.unwrap();
    assert!(list.contains("Test ADR"));
}
