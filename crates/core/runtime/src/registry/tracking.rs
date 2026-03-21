//! Fire-and-forget install tracking.
//!
//! After a package is freshly fetched from the registry (not a cache hit),
//! we POST to the registry's install endpoint so download counts are tracked.
//! This is non-blocking: a detached thread runs `curl` in the background.
//! Failures are silently ignored — tracking must never slow down or break installs.

use std::process::{Command, Stdio};

/// Percent-encode a package path for use in a URL.
///
/// Encodes everything except unreserved characters (RFC 3986):
/// ALPHA / DIGIT / "-" / "." / "_" / "~"
fn percent_encode(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(byte as char);
            }
            b'/' => {
                // Keep path separators readable — registry routes expect them encoded.
                encoded.push_str("%2F");
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}

/// Fire-and-forget POST to track a package install.
///
/// Spawns a detached thread that runs `curl` with a 5-second timeout.
/// Returns immediately. Never errors — all failures are silently swallowed.
///
/// Skipped when:
/// - `offline` is true (--offline flag)
/// - `SHIP_REGISTRY_URL` is set to an empty string (opt-out)
pub fn track_install(package_path: &str, offline: bool) {
    if offline {
        return;
    }

    // Check opt-out: SHIP_REGISTRY_URL set to empty string.
    match std::env::var("SHIP_REGISTRY_URL") {
        Ok(val) if val.is_empty() => return,
        _ => {}
    }

    let base_url = std::env::var("SHIP_REGISTRY_URL")
        .unwrap_or_else(|_| "https://getship.dev".to_string());
    let encoded = percent_encode(package_path);
    let endpoint = format!("{}/api/registry/{}/install", base_url, encoded);

    std::thread::spawn(move || {
        let _ = Command::new("curl")
            .args([
                "-s",          // silent
                "-o", "/dev/null", // discard response body
                "-X", "POST",
                "--max-time", "5",
                &endpoint,
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_encode_simple_path() {
        assert_eq!(
            percent_encode("github.com/owner/repo"),
            "github.com%2Fowner%2Frepo"
        );
    }

    #[test]
    fn percent_encode_preserves_unreserved() {
        assert_eq!(percent_encode("a-b_c.d~e"), "a-b_c.d~e");
    }

    #[test]
    fn percent_encode_encodes_special_chars() {
        assert_eq!(percent_encode("a b@c"), "a%20b%40c");
    }

    #[test]
    fn track_install_skipped_when_offline() {
        // Should return immediately without spawning anything.
        // No assertion needed — this just verifies it doesn't panic.
        track_install("github.com/test/pkg", true);
    }

    #[test]
    fn track_install_skipped_when_opted_out() {
        // Set SHIP_REGISTRY_URL to empty to opt out.
        // SAFETY: test is single-threaded; env var is restored immediately after.
        let prev = std::env::var("SHIP_REGISTRY_URL").ok();
        unsafe { std::env::set_var("SHIP_REGISTRY_URL", "") };
        track_install("github.com/test/pkg", false);
        match prev {
            Some(val) => unsafe { std::env::set_var("SHIP_REGISTRY_URL", val) },
            None => unsafe { std::env::remove_var("SHIP_REGISTRY_URL") },
        }
    }

    #[test]
    fn track_install_does_not_block_on_bad_url() {
        // Point to a URL that won't resolve — should still return quickly.
        // SAFETY: test is single-threaded; env var is restored immediately after.
        let prev = std::env::var("SHIP_REGISTRY_URL").ok();
        unsafe { std::env::set_var("SHIP_REGISTRY_URL", "http://127.0.0.1:1") };
        track_install("github.com/test/pkg", false);
        // Give the spawned thread a moment, but the main thread should not block.
        std::thread::sleep(std::time::Duration::from_millis(50));
        match prev {
            Some(val) => unsafe { std::env::set_var("SHIP_REGISTRY_URL", val) },
            None => unsafe { std::env::remove_var("SHIP_REGISTRY_URL") },
        }
    }
}
