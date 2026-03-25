use crate::resource_resolver::resolve_resource_uri;
use runtime::db::{ensure_db, jobs, targets};
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

// ── Job resources ─────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn jobs_list_resource_empty() {
    let (_tmp, ship_dir) = setup();
    let result = resolve("ship://jobs", &ship_dir).await.unwrap();
    assert_eq!(result, "No jobs found.");
}

#[tokio::test(flavor = "multi_thread")]
async fn jobs_list_and_get_round_trip() {
    let (_tmp, ship_dir) = setup();
    let job = jobs::create_job("compile", Some("main"), None, None, None, 0, None, vec![], vec![])
        .unwrap();

    let list = resolve("ship://jobs", &ship_dir).await.unwrap();
    assert!(list.contains(&job.id), "job list should contain id: {list}");
    assert!(list.contains("compile"), "job list should contain kind");

    let detail = resolve(&format!("ship://jobs/{}", job.id), &ship_dir)
        .await
        .unwrap();
    assert!(detail.contains(&job.id));
    assert!(detail.contains("\"kind\": \"compile\""));
}

#[tokio::test(flavor = "multi_thread")]
async fn job_get_nonexistent_returns_none() {
    let (_tmp, ship_dir) = setup();
    assert!(resolve("ship://jobs/nonexistent", &ship_dir).await.is_none());
}

// ── Target resources ──────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn targets_list_resource_empty() {
    let (_tmp, ship_dir) = setup();
    let result = resolve("ship://targets", &ship_dir).await.unwrap();
    assert_eq!(result, "No targets found.");
}

#[tokio::test(flavor = "multi_thread")]
async fn target_get_round_trip() {
    let (_tmp, ship_dir) = setup();
    let target = targets::create_target("surface", "Compiler", None, None, None).unwrap();

    let list = resolve("ship://targets", &ship_dir).await.unwrap();
    assert!(list.contains("Compiler"));

    let detail = resolve(&format!("ship://targets/{}", target.id), &ship_dir)
        .await
        .unwrap();
    assert!(detail.contains("\"title\": \"Compiler\""));
    assert!(detail.contains("\"kind\": \"surface\""));
}

#[tokio::test(flavor = "multi_thread")]
async fn target_get_nonexistent_returns_none() {
    let (_tmp, ship_dir) = setup();
    assert!(resolve("ship://targets/nonexistent", &ship_dir).await.is_none());
}

// ── Capability resources ──────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn capability_get_round_trip() {
    let (_tmp, ship_dir) = setup();
    let target = targets::create_target("surface", "Runtime", None, None, None).unwrap();
    let cap = targets::create_capability(&target.id, "MCP robustness", None).unwrap();

    let detail = resolve(&format!("ship://capabilities/{}", cap.id), &ship_dir)
        .await
        .unwrap();
    assert!(detail.contains("\"title\": \"MCP robustness\""));
    assert!(detail.contains(&target.id));
}

#[tokio::test(flavor = "multi_thread")]
async fn capability_get_nonexistent_returns_none() {
    let (_tmp, ship_dir) = setup();
    assert!(resolve("ship://capabilities/nonexistent", &ship_dir).await.is_none());
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
    std::fs::write(specs_path.join("design-v1.md"), "# Design V1\n\nSpec content here.").unwrap();

    let list = resolve("ship://specs", &ship_dir).await.unwrap();
    assert!(list.contains("design-v1"), "spec list: {list}");

    let detail = resolve("ship://specs/design-v1", &ship_dir).await.unwrap();
    assert!(detail.contains("# Design V1"));
    assert!(detail.contains("Spec content here."));
}

#[tokio::test(flavor = "multi_thread")]
async fn spec_get_nonexistent_returns_none() {
    let (_tmp, ship_dir) = setup();
    assert!(resolve("ship://specs/nonexistent", &ship_dir).await.is_none());
}

// ── Existing resources still work ─────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn notes_resource_round_trip() {
    let (_tmp, ship_dir) = setup();
    let result = resolve("ship://notes", &ship_dir).await.unwrap();
    assert_eq!(result, "No notes found.");

    runtime::db::notes::create_note("Test Note", "body", vec![], None).unwrap();
    let list = resolve("ship://notes", &ship_dir).await.unwrap();
    assert!(list.contains("Test Note"));
}

#[tokio::test(flavor = "multi_thread")]
async fn adrs_resource_round_trip() {
    let (_tmp, ship_dir) = setup();
    let result = resolve("ship://adrs", &ship_dir).await.unwrap();
    assert_eq!(result, "No ADRs found.");

    runtime::db::adrs::create_adr("Test ADR", "context", "decision", "proposed").unwrap();
    let list = resolve("ship://adrs", &ship_dir).await.unwrap();
    assert!(list.contains("Test ADR"));
}
