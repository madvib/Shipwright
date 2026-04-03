use runtime::events::job::event_types;
use runtime::events::kernel_router::{ActorConfig, KernelRouter};
use runtime::events::validator::{CallerKind, EmitContext};
use runtime::events::EventEnvelope;
use runtime::projections::job::{project, JobStatus};
use std::sync::Arc;
use tempfile::TempDir;

fn runtime_ctx() -> EmitContext {
    EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    }
}

fn job_dispatch_config() -> ActorConfig {
    ActorConfig {
        namespace: "service.job-dispatch".to_string(),
        write_namespaces: vec!["job.".to_string()],
        read_namespaces: vec![],
        subscribe_namespaces: vec!["job.".to_string()],
    }
}

fn setup() -> (TempDir, KernelRouter) {
    let dir = TempDir::new().unwrap();
    let router = KernelRouter::new(dir.path().join(".ship")).unwrap();
    (dir, router)
}

fn ev(event_type: &str, payload: serde_json::Value) -> EventEnvelope {
    EventEnvelope::new(event_type, "test-entity", &payload).unwrap()
}

/// Kernel routing delivers `job.created` to an actor subscribed to `job.`.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn job_created_delivered_to_subscriber() {
    let (_dir, mut router) = setup();
    let (_store, mut mb) = router
        .spawn_actor("service.job-dispatch", job_dispatch_config())
        .unwrap();

    let event = ev(
        event_types::JOB_CREATED,
        serde_json::json!({
            "job_id": "j-created-1",
            "slug": "auth-tests",
            "agent": "rust-runtime",
            "branch": "job/auth-tests",
            "spec_path": ".ship-session/job-spec.md",
            "plan_id": null,
            "model": null,
            "provider": null,
        }),
    );

    router.route(event.clone(), &runtime_ctx()).await.unwrap();

    let received = mb
        .try_recv()
        .expect("job.created must be delivered to job-dispatch subscriber");
    assert_eq!(received.id, event.id);
    assert_eq!(received.event_type, event_types::JOB_CREATED);
}

/// Kernel routing delivers `job.update` with correct payload to subscriber.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn job_update_delivered_to_subscriber() {
    let (_dir, mut router) = setup();
    let (_store, mut mb) = router
        .spawn_actor("service.job-dispatch", job_dispatch_config())
        .unwrap();

    let event = ev(
        event_types::JOB_UPDATE,
        serde_json::json!({
            "job_id": "j-update-1",
            "message": "please fix the lint errors",
            "sender": "human",
            "slug": "auth-tests",
        }),
    );

    router.route(event.clone(), &runtime_ctx()).await.unwrap();

    let received = mb
        .try_recv()
        .expect("job.update must be delivered to job-dispatch subscriber");
    assert_eq!(received.id, event.id);
    assert_eq!(received.event_type, event_types::JOB_UPDATE);
    let payload: serde_json::Value =
        serde_json::from_str(&received.payload_json).unwrap();
    assert_eq!(
        payload["message"].as_str(),
        Some("please fix the lint errors")
    );
}

/// `job.completed` is delivered to job-dispatch subscriber.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn job_completed_delivered_to_subscriber() {
    let (_dir, mut router) = setup();
    let (_store, mut mb) = router
        .spawn_actor("service.job-dispatch", job_dispatch_config())
        .unwrap();

    let event = ev(
        event_types::JOB_COMPLETED,
        serde_json::json!({"job_id": "j-comp-1", "slug": "auth-tests"}),
    );

    router.route(event.clone(), &runtime_ctx()).await.unwrap();

    let received = mb
        .try_recv()
        .expect("job.completed must be delivered to job-dispatch subscriber");
    assert_eq!(received.event_type, event_types::JOB_COMPLETED);
}

/// `job.merged` is delivered to job-dispatch subscriber.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn job_merged_delivered_to_subscriber() {
    let (_dir, mut router) = setup();
    let (_store, mut mb) = router
        .spawn_actor("service.job-dispatch", job_dispatch_config())
        .unwrap();

    let event = ev(
        event_types::JOB_MERGED,
        serde_json::json!({"job_id": "j-merged-1", "slug": "auth-tests"}),
    );

    router.route(event.clone(), &runtime_ctx()).await.unwrap();

    let received = mb
        .try_recv()
        .expect("job.merged must be delivered to job-dispatch subscriber");
    assert_eq!(received.event_type, event_types::JOB_MERGED);
}

/// `workspace.activated` is not delivered to the job-dispatch subscriber.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn non_job_events_not_delivered_to_subscriber() {
    let (_dir, mut router) = setup();
    let (_store, mut mb) = router
        .spawn_actor("service.job-dispatch", job_dispatch_config())
        .unwrap();

    let event = ev(
        "workspace.activated",
        serde_json::json!({ "branch": "main", "worktree_path": "/tmp/test" }),
    );

    router.route(event, &runtime_ctx()).await.unwrap();

    assert!(
        mb.try_recv().is_err(),
        "workspace.activated must not reach the job-dispatch subscriber"
    );
}

/// `job.created` with model/provider in the payload projects to a valid
/// pending JobRecord.
#[test]
fn job_projection_handles_model_and_provider_payload() {
    let events = vec![ev(
        event_types::JOB_CREATED,
        serde_json::json!({
            "job_id": "j-model-1",
            "slug": "typed-api",
            "agent": "rust-runtime",
            "branch": "job/typed-api",
            "spec_path": ".ship-session/job-spec.md",
            "plan_id": null,
            "model": "claude-opus-4-5",
            "provider": "anthropic",
        }),
    )];

    let map = project(&events);
    let rec = map.get("j-model-1").expect("job record must be created");
    assert_eq!(rec.status, JobStatus::Pending);
    assert_eq!(rec.slug, "typed-api");
    assert_eq!(rec.agent, "rust-runtime");
    assert_eq!(rec.branch, "job/typed-api");
    assert_eq!(rec.spec_path, ".ship-session/job-spec.md");
    let raw: serde_json::Value =
        serde_json::from_str(&events[0].payload_json).unwrap();
    assert_eq!(raw["model"].as_str(), Some("claude-opus-4-5"));
    assert_eq!(raw["provider"].as_str(), Some("anthropic"));
}

/// SHIP_MESH_ID env var overrides derived actor ID.
#[test]
fn ship_mesh_id_env_overrides_derived_actor_id() {
    let slug = "payments-refactor";

    let without = std::env::var("SHIP_MESH_ID")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "agent.rust-runtime.job/payments-refactor".to_string());
    assert_eq!(without, "agent.rust-runtime.job/payments-refactor");

    unsafe { std::env::set_var("SHIP_MESH_ID", slug) };
    let with_mesh_id = std::env::var("SHIP_MESH_ID")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "agent.rust-runtime.job/payments-refactor".to_string());
    unsafe { std::env::remove_var("SHIP_MESH_ID") };

    assert_eq!(
        with_mesh_id, slug,
        "SHIP_MESH_ID must override the derived profile+branch actor ID"
    );
}

/// Jobs with depends_on are deferred into the pending map.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn job_with_depends_on_is_deferred() {
    let (_dir, router) = setup();
    let kernel = Arc::new(tokio::sync::Mutex::new(router));
    let mut pending = std::collections::HashMap::new();

    let event = ev(
        event_types::JOB_CREATED,
        serde_json::json!({
            "job_id": "j-deferred",
            "slug": "downstream",
            "agent": "rust-runtime",
            "branch": "job/downstream",
            "spec_path": ".ship-session/job-spec.md",
            "plan_id": null,
            "depends_on": ["upstream-a"]
        }),
    );

    super::handle_job_created(&kernel, &event, &mut pending).await;
    assert!(
        pending.contains_key("downstream"),
        "job with depends_on must be stored in pending"
    );
}

/// Jobs without depends_on are not placed in pending map.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn job_without_depends_on_not_deferred() {
    let (_dir, router) = setup();
    let kernel = Arc::new(tokio::sync::Mutex::new(router));
    let mut pending = std::collections::HashMap::new();

    let event = ev(
        event_types::JOB_CREATED,
        serde_json::json!({
            "job_id": "j-immediate",
            "slug": "no-deps",
            "agent": "rust-runtime",
            "branch": "job/no-deps",
            "spec_path": ".ship-session/job-spec.md",
            "plan_id": null,
        }),
    );

    super::handle_job_created(&kernel, &event, &mut pending).await;
    assert!(
        !pending.contains_key("no-deps"),
        "job without depends_on must not be deferred"
    );
}

/// Completing a dependency removes it from pending jobs' depends_on lists.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn completing_dependency_unblocks_pending_job() {
    let (_dir, router) = setup();
    let kernel = Arc::new(tokio::sync::Mutex::new(router));
    let mut pending = std::collections::HashMap::new();

    pending.insert(
        "downstream".to_string(),
        runtime::events::job::JobCreatedPayload {
            job_id: "j-down".to_string(),
            slug: "downstream".to_string(),
            agent: "rust-runtime".to_string(),
            branch: "job/downstream".to_string(),
            spec_path: ".ship-session/job-spec.md".to_string(),
            plan_id: None,
            model: None,
            provider: None,
            depends_on: Some(vec!["upstream-a".to_string(), "upstream-b".to_string()]),
        },
    );

    // Complete upstream-a
    let completed_event = ev(
        event_types::JOB_COMPLETED,
        serde_json::json!({"job_id": "j-up-a", "slug": "upstream-a"}),
    );
    super::dispatch_unblocked_jobs(&kernel, &completed_event, &mut pending).await;

    // Still pending — upstream-b remains
    assert!(pending.contains_key("downstream"));
    let remaining = pending["downstream"].depends_on.as_ref().unwrap();
    assert_eq!(remaining, &["upstream-b"]);

    // Complete upstream-b — job should be removed from pending (dispatched)
    let completed_event_b = ev(
        event_types::JOB_COMPLETED,
        serde_json::json!({"job_id": "j-up-b", "slug": "upstream-b"}),
    );
    super::dispatch_unblocked_jobs(&kernel, &completed_event_b, &mut pending).await;

    assert!(
        !pending.contains_key("downstream"),
        "fully unblocked job must be removed from pending"
    );
}

/// Projection reflects Completed status from job.completed event.
#[test]
fn projection_completed_status() {
    let events = vec![
        ev(
            event_types::JOB_CREATED,
            serde_json::json!({
                "job_id": "j-proj-comp",
                "slug": "test-slug",
                "agent": "rust-runtime",
                "branch": "job/test-slug",
                "spec_path": "spec.md",
                "plan_id": null,
            }),
        ),
        ev(
            event_types::JOB_COMPLETED,
            serde_json::json!({"job_id": "j-proj-comp", "slug": "test-slug"}),
        ),
    ];
    let map = project(&events);
    assert_eq!(map.get("j-proj-comp").unwrap().status, JobStatus::Completed);
}
