use super::*;
use crate::mcp::McpFile;
use tempfile::TempDir;

// ── getship URL detection ────────────────────────────────────────────────────

#[test]
fn is_getship_url_matches_valid_urls() {
    assert!(is_getship_url("https://getship.dev/p/rust-expert"));
    assert!(is_getship_url("https://getship.dev/profiles/cli-lane"));
    assert!(is_getship_url("https://www.getship.dev/p/test"));
    assert!(!is_getship_url("https://example.com/p/test"));
    assert!(!is_getship_url("/some/local/path"));
}

#[test]
fn extract_profile_id_from_url() {
    assert_eq!(
        extract_profile_id("https://getship.dev/p/rust-expert").unwrap(),
        "rust-expert"
    );
    assert_eq!(
        extract_profile_id("https://getship.dev/profiles/cli-lane").unwrap(),
        "cli-lane"
    );
}

#[test]
fn extract_profile_id_single_segment() {
    assert_eq!(
        extract_profile_id("https://getship.dev/my-profile").unwrap(),
        "my-profile"
    );
}

#[test]
fn extract_profile_id_empty_path_fails() {
    assert!(extract_profile_id("https://getship.dev/").is_err());
}

// ── GitHub URL detection ─────────────────────────────────────────────────────

#[test]
fn is_github_url_matches_owner_repo() {
    assert!(convert_github::is_github_url(
        "https://github.com/acme/my-repo"
    ));
    assert!(convert_github::is_github_url(
        "https://github.com/acme/my-repo.git"
    ));
    assert!(convert_github::is_github_url(
        "https://github.com/acme/my-repo/tree/main"
    ));
    assert!(convert_github::is_github_url(
        "http://github.com/acme/my-repo"
    ));
}

#[test]
fn is_github_url_rejects_incomplete_paths() {
    assert!(!convert_github::is_github_url("https://github.com/acme"));
    assert!(!convert_github::is_github_url("https://github.com/"));
    assert!(!convert_github::is_github_url("https://getship.dev/p/test"));
    assert!(!convert_github::is_github_url(
        "https://gitlab.com/acme/repo"
    ));
}

// ── extract_github_slug ──────────────────────────────────────────────────────

#[test]
fn extract_github_slug_basic() {
    assert_eq!(
        convert_github::extract_github_slug("https://github.com/acme/my-repo"),
        Some("github.com/acme/my-repo".to_string())
    );
}

#[test]
fn extract_github_slug_strips_git_suffix() {
    assert_eq!(
        convert_github::extract_github_slug("https://github.com/acme/my-repo.git"),
        Some("github.com/acme/my-repo".to_string())
    );
}

// ── sanitize_filename ────────────────────────────────────────────────────────

#[test]
fn sanitize_filename_replaces_spaces_and_slashes() {
    assert_eq!(
        convert_github::sanitize_filename("hello world"),
        "hello-world"
    );
    assert_eq!(convert_github::sanitize_filename("path/name"), "path-name");
    assert_eq!(
        convert_github::sanitize_filename("valid-name_123"),
        "valid-name_123"
    );
}

// ── convert_from_github_with_base (mocked server) ───────────────────────────

fn run_in_tmp<F: FnOnce(&std::path::Path)>(f: F) {
    let tmp = TempDir::new().unwrap();
    let orig = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();
    f(tmp.path());
    // Best-effort restore — may fail if another test changed cwd concurrently.
    let _ = std::env::set_current_dir(orig);
}

#[test]
fn github_convert_writes_profiles_rules_and_mcp() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("POST", "/api/github/import")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{
            "library": {
                "name": "test-lib",
                "modes": [{"name": "rust-expert"}],
                "rules": [{"name": "no-panics", "content": "Never use unwrap()"}],
                "mcp_servers": [{"id": "linear", "name": "Linear", "command": "npx", "args": ["-y", "@mcp/linear"]}]
            }
        }"#)
        .create();

    run_in_tmp(|tmp| {
        convert_github::convert_from_github_with_base(
            "https://github.com/acme/test-repo",
            &server.url(),
        )
        .unwrap();

        assert!(
            tmp.join(".ship/agents/rust-expert.toml").exists(),
            "agent written"
        );
        assert!(
            tmp.join(".ship/rules/no-panics.md").exists(),
            "rule written"
        );
        let rule = std::fs::read_to_string(tmp.join(".ship/rules/no-panics.md")).unwrap();
        assert_eq!(rule, "Never use unwrap()");
        assert!(tmp.join(".ship/mcp.toml").exists(), "mcp written");
    });

    mock.assert();
}

#[test]
fn github_convert_handles_server_error_field() {
    let mut server = mockito::Server::new();
    server
        .mock("POST", "/api/github/import")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "not found"}"#)
        .create();

    run_in_tmp(|_| {
        let err = convert_github::convert_from_github_with_base(
            "https://github.com/acme/missing-repo",
            &server.url(),
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("not found") || err.to_string().contains("Repository"),
            "unexpected error: {err}"
        );
    });
}

#[test]
fn github_convert_skips_duplicate_mcp_servers() {
    let mut server = mockito::Server::new();
    server
        .mock("POST", "/api/github/import")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "library": {
                "mcp_servers": [{"id": "linear", "command": "npx", "args": []}]
            }
        }"#,
        )
        .expect(2)
        .create();

    run_in_tmp(|tmp| {
        let base = server.url();
        convert_github::convert_from_github_with_base("https://github.com/acme/repo", &base)
            .unwrap();
        // Second call should not duplicate the MCP entry
        convert_github::convert_from_github_with_base("https://github.com/acme/repo", &base)
            .unwrap();

        let mcp_path = tmp.join(".ship/mcp.toml");
        let mcp = McpFile::load(&mcp_path).unwrap();
        assert_eq!(
            mcp.servers.len(),
            1,
            "duplicate MCP entry should be skipped"
        );
    });
}
