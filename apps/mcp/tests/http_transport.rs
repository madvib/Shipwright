use std::time::Duration;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

async fn start_test_server_with_token(token: Option<&str>) -> (u16, CancellationToken, Option<tempfile::TempDir>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let tmp_dir = if let Some(tok) = token {
        let dir = tempfile::TempDir::new().unwrap();
        let config_path = dir.path().join("config.toml");
        std::fs::write(&config_path, format!("[auth]\ntoken = \"{tok}\"")).unwrap();
        // SAFETY: single-threaded test setup, no concurrent env access
        unsafe { std::env::set_var("SHIP_GLOBAL_DIR", dir.path()) };
        Some(dir)
    } else {
        // Use a temp dir with no config.toml so there's no stale token
        let dir = tempfile::TempDir::new().unwrap();
        // SAFETY: single-threaded test setup, no concurrent env access
        unsafe { std::env::set_var("SHIP_GLOBAL_DIR", dir.path()) };
        Some(dir)
    };

    use axum::Router;
    use mcp::ShipServer;
    use rmcp::transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService,
        session::local::LocalSessionManager,
    };

    let ct = CancellationToken::new();
    let service: StreamableHttpService<ShipServer, LocalSessionManager> =
        StreamableHttpService::new(
            || Ok(ShipServer::new()),
            Default::default(),
            StreamableHttpServerConfig {
                cancellation_token: ct.child_token(),
                ..Default::default()
            },
        );

    let mcp_router = Router::new().nest_service("/mcp", service);

    let app = if let Some(tok) = token {
        use axum::{
            body::Body,
            extract::State,
            http::{Request, StatusCode},
            middleware::{self, Next},
            response::Response,
        };
        async fn auth(
            State(expected): State<String>,
            req: Request<Body>,
            next: Next,
        ) -> Result<Response, StatusCode> {
            let h = req
                .headers()
                .get(axum::http::header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok());
            if h == Some(&format!("Bearer {expected}")) {
                Ok(next.run(req).await)
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        mcp_router.route_layer(middleware::from_fn_with_state(tok.to_string(), auth))
    } else {
        mcp_router
    };

    let ct_clone = ct.clone();
    tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async move { ct_clone.cancelled_owned().await })
            .await
            .unwrap();
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    (port, ct, tmp_dir)
}

const INIT_BODY: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}"#;

#[tokio::test]
async fn http_server_responds_to_initialize() {
    let (port, ct, _tmp) = start_test_server_with_token(None).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://127.0.0.1:{port}/mcp"))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .body(INIT_BODY)
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status(), 200);
    let text = resp.text().await.unwrap();
    assert!(text.contains("\"result\""), "expected result in response, got: {text}");

    ct.cancel();
}

#[tokio::test]
async fn http_bearer_token_rejected_without_header() {
    let (port, ct, _tmp) = start_test_server_with_token(Some("mysecret")).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://127.0.0.1:{port}/mcp"))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .body(INIT_BODY)
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status(), 401);

    ct.cancel();
}

#[tokio::test]
async fn http_bearer_token_accepted_with_correct_header() {
    let (port, ct, _tmp) = start_test_server_with_token(Some("mysecret")).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://127.0.0.1:{port}/mcp"))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .header("Authorization", "Bearer mysecret")
        .body(INIT_BODY)
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status(), 200);

    ct.cancel();
}
