use std::sync::Arc;

use rmcp::{ClientHandler, ServiceExt};
use tokio::sync::Notify;

/// Minimal client that records when it receives a resource list changed notification.
struct NotificationClient {
    resource_list_changed: Arc<Notify>,
}

impl ClientHandler for NotificationClient {
    async fn on_resource_list_changed(
        &self,
        _context: rmcp::service::NotificationContext<rmcp::RoleClient>,
    ) {
        self.resource_list_changed.notify_one();
    }
}

#[tokio::test]
async fn peer_captured_on_initialized() {
    let (server_transport, client_transport) = tokio::io::duplex(4096);

    let server = mcp::ShipServer::new();
    let peer_ref = server.notification_peer.clone();

    tokio::spawn(async move {
        let running = server.serve(server_transport).await.unwrap();
        running.waiting().await.unwrap();
    });

    let signal = Arc::new(Notify::new());
    let client = NotificationClient {
        resource_list_changed: signal.clone(),
    }
    .serve(client_transport)
    .await
    .unwrap();

    // After the client initializes, the server's on_initialized fires and stores the peer.
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    assert!(
        peer_ref.lock().await.is_some(),
        "notification_peer should be captured after client initialization"
    );

    client.cancel().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn state_modifying_tool_sends_resource_notification() {
    let tmp = tempfile::tempdir().unwrap();
    let project_dir =
        runtime::project::init_project(tmp.path().to_path_buf()).expect("init project");

    let (server_transport, client_transport) = tokio::io::duplex(65536);

    let server = mcp::StudioServer::new();
    *server.active_project.lock().await = Some(project_dir.clone());

    tokio::spawn(async move {
        let running = server.serve(server_transport).await.unwrap();
        running.waiting().await.unwrap();
    });

    let signal = Arc::new(Notify::new());
    let client = NotificationClient {
        resource_list_changed: signal.clone(),
    }
    .serve(client_transport)
    .await
    .unwrap();

    // Wait for initialization to complete so the peer is captured
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Create a skill directory so write_skill_file has somewhere to write
    let skill_dir = project_dir.join(".ship/skills/test-skill");
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(skill_dir.join("SKILL.md"), "# Test Skill\nA test.").unwrap();

    // Call write_skill_file which should trigger a notification
    let result = client
        .call_tool(rmcp::model::CallToolRequestParams {
            name: "write_skill_file".into(),
            arguments: Some(
                serde_json::json!({
                    "skill_id": "test-skill",
                    "file_path": "hello.txt",
                    "content": "world"
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
            meta: None,
            task: None,
        })
        .await
        .expect("call_tool should succeed");

    // Verify the tool succeeded
    let text: String = result
        .content
        .iter()
        .filter_map(|c| c.as_text().map(|t| t.text.as_str()))
        .collect();
    assert!(
        !text.starts_with("Error"),
        "write_skill_file failed: {text}"
    );

    // Wait for the resource list changed notification
    let received = tokio::time::timeout(std::time::Duration::from_secs(2), signal.notified()).await;

    assert!(
        received.is_ok(),
        "should have received resource list changed notification after write_skill_file"
    );

    client.cancel().await.unwrap();
}
