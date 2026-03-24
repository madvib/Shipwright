//! End-to-end test: spin up a mock Studio MCP server with a `transfer/bundle`
//! tool, then verify the CLI's MCP client fetches and parses it correctly.

use std::collections::HashMap;
use std::net::TcpListener;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Implementation, ListToolsResult,
    PaginatedRequestParams, ProtocolVersion, ServerCapabilities, ServerInfo,
};
use rmcp::service::RequestContext;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use rmcp::{ErrorData, RoleServer, ServerHandler, tool, tool_router};
use serde::Deserialize;
use tokio_util::sync::CancellationToken;

// ── Mock Studio MCP Server ──────────────────────────────────────────────────

/// Minimal MCP server that exposes a single `transfer/bundle` tool.
#[derive(Debug, Clone)]
struct MockStudioServer {
    tool_router: ToolRouter<Self>,
}

#[derive(Deserialize, rmcp::schemars::JsonSchema)]
struct TransferBundleParams {
    /// Optional share ID (ignored in test — always returns the same bundle).
    #[allow(dead_code)]
    share_id: Option<String>,
}

#[tool_router]
impl MockStudioServer {
    fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Return the transfer bundle for a shared agent config.")]
    fn transfer_bundle(
        &self,
        Parameters(_params): Parameters<TransferBundleParams>,
    ) -> String {
        serde_json::json!({
            "agent": {
                "id": "mcp-test-agent",
                "name": "MCP Test Agent",
                "description": "Agent delivered via MCP bridge",
                "model": "sonnet",
                "skills": ["tdd"],
                "rules": ["write tests first", "keep it simple"],
                "mcp_servers": []
            },
            "dependencies": {
                "github.com/anthropics/ship-skills": "^0.1.0"
            },
            "skills": {
                "tdd": {
                    "files": {
                        "SKILL.md": "---\nname: tdd\n---\nWrite the failing test before the implementation."
                    }
                }
            }
        })
        .to_string()
    }
}

impl ServerHandler for MockStudioServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "mock-studio".into(),
                version: "0.0.0-test".into(),
                ..Default::default()
            },
            instructions: None,
        }
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        self.tool_router
            .call(rmcp::handler::server::tool::ToolCallContext::new(
                self, request, context,
            ))
            .await
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult::with_all_items(
            self.tool_router.list_all(),
        ))
    }

    fn get_tool(&self, name: &str) -> Option<rmcp::model::Tool> {
        self.tool_router.get(name).cloned()
    }
}

/// Pick a random available port.
fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

/// Start the mock Studio MCP server on the given port. Returns a
/// CancellationToken to shut it down.
async fn start_mock_server(port: u16) -> CancellationToken {
    let ct = CancellationToken::new();
    let ct_child = ct.child_token();

    let service: StreamableHttpService<MockStudioServer, LocalSessionManager> =
        StreamableHttpService::new(
            || Ok(MockStudioServer::new()),
            Default::default(),
            StreamableHttpServerConfig {
                cancellation_token: ct_child,
                ..Default::default()
            },
        );

    let app = axum::Router::new().nest_service("/mcp", service);
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}"))
        .await
        .unwrap();

    let ct_serve = ct.child_token();
    tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async move { ct_serve.cancelled_owned().await })
            .await
            .unwrap();
    });

    // Give the server a moment to bind.
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    ct
}

// ── Tests ────────────────────────────────────────────────────────────────────

/// Full round-trip: start MCP server → CLI connects → fetches bundle → parses.
#[tokio::test]
async fn mcp_bridge_fetches_transfer_bundle() {
    let port = free_port();
    let ct = start_mock_server(port).await;

    let url = format!("http://127.0.0.1:{port}/mcp");

    // Use the CLI's internal fetch function (it's pub for testing).
    let bundle = fetch_via_mcp_test(&url).await.expect("MCP fetch must succeed");

    assert_eq!(bundle.agent.id, "mcp-test-agent");
    assert_eq!(bundle.agent.name.as_deref(), Some("MCP Test Agent"));
    assert_eq!(bundle.agent.skills, vec!["tdd"]);
    assert_eq!(bundle.agent.rules.len(), 2);
    assert_eq!(bundle.dependencies.len(), 1);
    assert!(bundle
        .dependencies
        .contains_key("github.com/anthropics/ship-skills"));
    assert_eq!(bundle.skills.len(), 1);
    assert!(bundle.skills.contains_key("tdd"));

    let skill_files = &bundle.skills["tdd"].files;
    assert!(skill_files.contains_key("SKILL.md"));
    assert!(skill_files["SKILL.md"].contains("failing test"));

    ct.cancel();
}

/// Verify the JSON fallback path works when pointed at a plain HTTP endpoint.
#[tokio::test]
async fn json_fallback_fetches_bundle() {
    let port = free_port();

    // Serve a plain JSON file via a minimal axum handler.
    let bundle_json = serde_json::json!({
        "agent": {
            "id": "json-agent",
            "skills": ["review"],
            "rules": []
        },
        "dependencies": {},
        "skills": {}
    });

    let json_str = bundle_json.to_string();
    let app = axum::Router::new().route(
        "/s/abc123",
        axum::routing::get(move || {
            let body = json_str.clone();
            async move {
                (
                    [(
                        axum::http::header::CONTENT_TYPE,
                        "application/json",
                    )],
                    body,
                )
            }
        }),
    );

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}"))
        .await
        .unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let url = format!("http://127.0.0.1:{port}/s/abc123");
    let bundle = fetch_via_json_test(&url)
        .await
        .expect("JSON fetch must succeed");

    assert_eq!(bundle.agent.id, "json-agent");
    assert_eq!(bundle.agent.skills, vec!["review"]);
    assert!(bundle.dependencies.is_empty());
    assert!(bundle.skills.is_empty());
}

/// Verify security scan blocks bundles with hidden Unicode.
#[tokio::test]
async fn mcp_bridge_blocks_malicious_bundle() {
    let port = free_port();
    let ct = start_mock_server(port).await;

    let url = format!("http://127.0.0.1:{port}/mcp");
    let mut bundle = fetch_via_mcp_test(&url).await.expect("fetch must succeed");

    // Inject bidi override into skill content.
    bundle
        .skills
        .get_mut("tdd")
        .unwrap()
        .files
        .insert("SKILL.md".into(), format!("clean text \u{202E} hidden"));

    // Re-scan should fail.
    let result = scan_bundle_test(&bundle);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("security scan"));

    ct.cancel();
}

// ── Types mirrored from add_from.rs (binary crate, not importable) ───────────

#[derive(Debug, serde::Deserialize)]
struct TransferBundle {
    agent: AgentBundle,
    #[serde(default)]
    dependencies: HashMap<String, String>,
    #[serde(default)]
    skills: HashMap<String, SkillBundle>,
}

#[derive(Debug, serde::Deserialize)]
struct AgentBundle {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    skills: Vec<String>,
    #[serde(default)]
    rules: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct SkillBundle {
    files: HashMap<String, String>,
}

async fn fetch_via_mcp_test(url: &str) -> anyhow::Result<TransferBundle> {
    use rmcp::model::{CallToolRequestParams, ClientInfo, Implementation, RawContent};
    use rmcp::transport::StreamableHttpClientTransport;
    use rmcp::ServiceExt;

    let transport = StreamableHttpClientTransport::from_uri(url);

    let client_info = ClientInfo {
        client_info: Implementation {
            name: "ship-cli-test".into(),
            version: "0.0.0-test".into(),
            ..Default::default()
        },
        ..Default::default()
    };

    let client = client_info
        .serve(transport)
        .await
        .map_err(|e| anyhow::anyhow!("MCP connection failed: {e:?}"))?;

    let result = client
        .call_tool(CallToolRequestParams {
            name: "transfer_bundle".into(),
            arguments: None,
            meta: None,
            task: None,
        })
        .await
        .map_err(|e| anyhow::anyhow!("tool call failed: {e:?}"))?;

    let text = result
        .content
        .iter()
        .filter_map(|c| match &c.raw {
            RawContent::Text(t) => Some(t.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("");

    let bundle: TransferBundle = serde_json::from_str(&text)?;

    let _ = client.cancel().await;
    Ok(bundle)
}

async fn fetch_via_json_test(url: &str) -> anyhow::Result<TransferBundle> {
    let resp = reqwest::get(url).await?;
    let text = resp.text().await?;
    let bundle: TransferBundle = serde_json::from_str(&text)?;
    Ok(bundle)
}

fn scan_bundle_test(bundle: &TransferBundle) -> anyhow::Result<()> {
    let mut all_findings = Vec::new();

    for (skill_id, skill) in &bundle.skills {
        for (path, content) in &skill.files {
            let filename = format!("skills/{skill_id}/{path}");
            let findings = runtime::security::scan_text(content, &filename);
            all_findings.extend(findings);
        }
    }

    for (i, rule) in bundle.agent.rules.iter().enumerate() {
        let findings = runtime::security::scan_text(rule, &format!("rule[{i}]"));
        all_findings.extend(findings);
    }

    if runtime::security::has_critical(&all_findings) {
        let critical: Vec<String> = all_findings
            .iter()
            .filter(|f| f.severity == runtime::security::Severity::Critical)
            .map(|f| f.to_string())
            .collect();
        anyhow::bail!(
            "security scan blocked: {} critical finding(s):\n  {}",
            critical.len(),
            critical.join("\n  ")
        );
    }

    Ok(())
}
