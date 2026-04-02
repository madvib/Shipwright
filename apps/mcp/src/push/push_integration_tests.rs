//! Integration tests proving the full event pipeline:
//! KernelRouter → actor mailbox → EventRelay → PushAdapter.
//!
//! Tests both Studio→agent and agent→agent (mesh) flows.

use crate::push::PushAdapter;
use crate::server::notification_relay::{EventRelay, PeerHandle};
use runtime::events::{ActorConfig, EventEnvelope, KernelRouter};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;

type Log = Arc<Mutex<Vec<EventEnvelope>>>;

struct RecordingAdapter {
    name: &'static str,
    events: Log,
}

impl RecordingAdapter {
    fn new(name: &'static str) -> (Self, Log) {
        let log: Log = Arc::new(Mutex::new(Vec::new()));
        (Self { name, events: log.clone() }, log)
    }
}

#[async_trait::async_trait]
impl PushAdapter for RecordingAdapter {
    async fn push_event(&self, event: &EventEnvelope) {
        self.events.lock().await.push(event.clone());
    }
    fn adapter_name(&self) -> &'static str {
        self.name
    }
}

/// Create a KernelRouter in a temp dir.
fn temp_kernel() -> KernelRouter {
    let dir = tempfile::tempdir().unwrap();
    KernelRouter::new(dir.into_path()).unwrap()
}

/// Wire: spawn actor → create relay with adapter → return (relay handle, log).
async fn wire_agent(
    kr: &mut KernelRouter,
    actor_id: &str,
    adapter_name: &'static str,
    subscribe: Vec<String>,
) -> Log {
    let config = ActorConfig {
        namespace: actor_id.to_string(),
        write_namespaces: vec!["".to_string()],
        read_namespaces: vec!["".to_string()],
        subscribe_namespaces: subscribe,
    };
    let (_store, mailbox) = kr.spawn_actor(actor_id, config).unwrap();

    let (adapter, log) = RecordingAdapter::new(adapter_name);
    let relay = EventRelay::new();
    relay
        .add_peer(PeerHandle {
            id: format!("{actor_id}-peer"),
            actor_id: actor_id.to_string(),
            adapter: Box::new(adapter),
            allowed_events: HashSet::new(), // system peer — receives all
        })
        .await;
    relay.spawn(mailbox);
    log
}

// ── Studio → Agent pipeline ────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn studio_event_reaches_agent_via_push_adapter() {
    let mut kr = temp_kernel();

    // Spawn studio actor (emitter)
    let studio_config = ActorConfig {
        namespace: "studio".to_string(),
        write_namespaces: vec!["studio.".to_string()],
        read_namespaces: vec!["studio.".to_string()],
        subscribe_namespaces: vec!["agent.".to_string()],
    };
    let (_studio_store, _studio_mailbox) = kr.spawn_actor("studio", studio_config).unwrap();

    // Spawn agent actor (receiver) subscribed to studio.*
    let agent_log = wire_agent(
        &mut kr,
        "agent.rust-lane",
        "claude-channel",
        vec!["studio.".to_string(), "mesh.".to_string()],
    )
    .await;

    // Studio emits a visual annotation
    let event = EventEnvelope::new(
        "studio.canvas.annotation",
        "v0.2.0",
        &serde_json::json!({
            "layer": "design",
            "type": "comment",
            "text": "This component needs a loading state",
            "x": 120,
            "y": 340,
        }),
    )
    .unwrap()
    .with_actor_id("studio")
    .with_context(Some("v0.2.0"), None);

    let ctx = runtime::events::EmitContext {
        caller_kind: runtime::events::CallerKind::Mcp,
        skill_id: None,
        workspace_id: Some("v0.2.0".to_string()),
        session_id: None,
    };
    kr.route(event.clone(), &ctx).await.unwrap();

    // Give the relay task time to process
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let received = agent_log.lock().await;
    assert_eq!(received.len(), 1, "agent should receive studio event");
    assert_eq!(received[0].event_type, "studio.canvas.annotation");
    assert_eq!(received[0].actor_id, Some("studio".to_string()));
}

// ── Agent → Agent via mesh ─────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn mesh_message_reaches_target_agent() {
    let mut kr = temp_kernel();

    // Spawn agent A (sender)
    let _a_log = wire_agent(
        &mut kr,
        "agent.react-designer",
        "adapter-a",
        vec!["mesh.".to_string(), "studio.".to_string()],
    )
    .await;

    // Spawn agent B (receiver)
    let b_log = wire_agent(
        &mut kr,
        "agent.rust-lane",
        "adapter-b",
        vec!["mesh.".to_string(), "studio.".to_string()],
    )
    .await;

    // Agent A sends a mesh message to agent B
    let msg = EventEnvelope::new(
        "mesh.message",
        "agent.rust-lane",
        &serde_json::json!({
            "from_agent_id": "agent.react-designer",
            "to_agent_id": "agent.rust-lane",
            "body": "Auth component is ready for integration. Branch: feature/auth-ui",
        }),
    )
    .unwrap()
    .with_actor_id("agent.react-designer")
    .with_context(Some("v0.2.0"), None);

    let ctx = runtime::events::EmitContext {
        caller_kind: runtime::events::CallerKind::Mcp,
        skill_id: None,
        workspace_id: Some("v0.2.0".to_string()),
        session_id: None,
    };
    kr.route(msg, &ctx).await.unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let received = b_log.lock().await;
    assert_eq!(received.len(), 1, "agent B should receive mesh message");
    assert_eq!(received[0].event_type, "mesh.message");

    // Verify payload integrity
    let payload: serde_json::Value =
        serde_json::from_str(&received[0].payload_json).unwrap();
    assert_eq!(
        payload["from_agent_id"].as_str().unwrap(),
        "agent.react-designer"
    );
    assert_eq!(
        payload["body"].as_str().unwrap(),
        "Auth component is ready for integration. Branch: feature/auth-ui"
    );
}

// ── Multi-agent broadcast ──────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn studio_event_fans_out_to_multiple_agents() {
    let mut kr = temp_kernel();

    // Studio emitter
    let studio_config = ActorConfig {
        namespace: "studio".to_string(),
        write_namespaces: vec!["studio.".to_string()],
        read_namespaces: vec![],
        subscribe_namespaces: vec![],
    };
    let _ = kr.spawn_actor("studio", studio_config).unwrap();

    // Three agents subscribed to studio.*
    let log_a = wire_agent(
        &mut kr,
        "agent.commander",
        "adapter-cmd",
        vec!["studio.".to_string()],
    )
    .await;
    let log_b = wire_agent(
        &mut kr,
        "agent.rust-lane",
        "adapter-rust",
        vec!["studio.".to_string()],
    )
    .await;
    let log_c = wire_agent(
        &mut kr,
        "agent.react-designer",
        "adapter-react",
        vec!["studio.".to_string()],
    )
    .await;

    // Studio broadcasts a message
    let event = EventEnvelope::new(
        "studio.message.visual",
        "v0.2.0",
        &serde_json::json!({
            "type": "annotation",
            "message": "Ship it!",
        }),
    )
    .unwrap()
    .with_actor_id("studio");

    let ctx = runtime::events::EmitContext {
        caller_kind: runtime::events::CallerKind::Mcp,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    };
    kr.route(event, &ctx).await.unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    assert_eq!(log_a.lock().await.len(), 1, "commander should receive");
    assert_eq!(log_b.lock().await.len(), 1, "rust-lane should receive");
    assert_eq!(log_c.lock().await.len(), 1, "react-designer should receive");
}

// ── Content formatting ─────────────────────────────────────

#[test]
fn claude_channel_formats_mesh_message_correctly() {
    use crate::push::claude_channel::format_event_content;

    let event = EventEnvelope::new(
        "mesh.message",
        "agent.rust-lane",
        &serde_json::json!({
            "from_agent_id": "agent.react-designer",
            "body": "PR is up for review",
        }),
    )
    .unwrap();

    let payload: serde_json::Value =
        serde_json::from_str(&event.payload_json).unwrap();
    let content = format_event_content(&event, &payload);
    assert_eq!(content, "Message from agent.react-designer: PR is up for review");
}

#[test]
fn claude_channel_formats_studio_annotation() {
    use crate::push::claude_channel::format_event_content;

    let event = EventEnvelope::new(
        "studio.canvas.annotation",
        "v0.2.0",
        &serde_json::json!({
            "text": "Needs loading state",
        }),
    )
    .unwrap();

    let payload: serde_json::Value =
        serde_json::from_str(&event.payload_json).unwrap();
    let content = format_event_content(&event, &payload);
    // Falls through to generic handler
    assert!(content.starts_with("[studio.canvas.annotation]"));
    assert!(content.contains("Needs loading state"));
}

// ── Full automated multi-agent flow ────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn agents_auto_register_discover_and_communicate() {
    // SETUP: Kernel + mesh service
    let mut kr = temp_kernel();
    let mesh_config = runtime::events::ActorConfig {
        namespace: "service.mesh".to_string(),
        write_namespaces: vec!["mesh.".to_string()],
        read_namespaces: vec!["mesh.".to_string()],
        subscribe_namespaces: vec!["mesh.".to_string()],
    };
    let handler: Box<dyn runtime::services::ServiceHandler> =
        Box::new(runtime::services::mesh::MeshService::new(
            tokio::sync::mpsc::unbounded_channel().0,
        ));
    runtime::services::spawn_service(&mut kr, "service.mesh", mesh_config, handler).unwrap();

    // AGENTS: Both subscribe to mesh.*
    let agent_a_log = wire_agent(
        &mut kr,
        "agent.rust-lane",
        "adapter-a",
        vec!["mesh.".to_string(), "studio.".to_string()],
    )
    .await;
    let agent_b_log = wire_agent(
        &mut kr,
        "agent.react-designer",
        "adapter-b",
        vec!["mesh.".to_string(), "studio.".to_string()],
    )
    .await;

    // AUTO-REGISTER: Both agents emit mesh.register (simulating on_initialized)
    let ctx = runtime::events::EmitContext {
        caller_kind: runtime::events::CallerKind::Mcp,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    };

    let reg_a = EventEnvelope::new(
        "mesh.register",
        "agent.rust-lane",
        &serde_json::json!({
            "agent_id": "agent.rust-lane",
            "capabilities": ["rust", "compiler"]
        }),
    )
    .unwrap()
    .with_actor_id("agent.rust-lane");

    let reg_b = EventEnvelope::new(
        "mesh.register",
        "agent.react-designer",
        &serde_json::json!({
            "agent_id": "agent.react-designer",
            "capabilities": ["react", "ui"]
        }),
    )
    .unwrap()
    .with_actor_id("agent.react-designer");

    kr.route(reg_a, &ctx).await.unwrap();
    kr.route(reg_b, &ctx).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Both agents receive register events (broadcast to mesh subscribers)
    let a_events = agent_a_log.lock().await;
    let b_events = agent_b_log.lock().await;
    assert!(a_events.iter().any(|e| e.event_type == "mesh.register"),
        "Agent A should receive register events");
    assert!(b_events.iter().any(|e| e.event_type == "mesh.register"),
        "Agent B should receive register events");
    drop(a_events);
    drop(b_events);

    // AGENT A SENDS TO AGENT B: mesh_send simulation
    let msg = EventEnvelope::new(
        "mesh.message",
        "agent.react-designer",
        &serde_json::json!({
            "from_agent_id": "agent.rust-lane",
            "to_agent_id": "agent.react-designer",
            "body": "Rust implementation complete. Ready for UI integration.",
        }),
    )
    .unwrap()
    .with_actor_id("agent.rust-lane");

    kr.route(msg, &ctx).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // VERIFY B RECEIVES: No polling, no tool calls. Just appears in mailbox → relay → adapter.
    let b_messages = agent_b_log.lock().await;
    let msg_event = b_messages
        .iter()
        .find(|e| e.event_type == "mesh.message")
        .expect("Agent B should receive the directed mesh.message");
    let payload: serde_json::Value =
        serde_json::from_str(&msg_event.payload_json).unwrap();
    assert_eq!(
        payload["from_agent_id"].as_str().unwrap(),
        "agent.rust-lane"
    );
    assert_eq!(
        payload["body"].as_str().unwrap(),
        "Rust implementation complete. Ready for UI integration."
    );
    println!("✓ Agent A → B message delivery works. Agent B received via push adapter.");
}
