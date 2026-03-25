//! Registry hash verification for first-time installs.
//!
//! When a package is installed without an existing lockfile entry, we query the
//! Ship registry for the expected content hash. If the registry reports a hash
//! that differs from what we computed locally, we warn (but do not block) the
//! install. This catches cases where a package has been modified after
//! publication without a version bump.
//!
//! Advisory only for v0.1 — installs proceed even on mismatch or registry
//! unavailability.

use super::tracking::percent_encode;

/// Query the Ship registry for the expected content hash of a package version.
///
/// Returns `Some(hash)` if the registry has a `contentHash` for the given
/// version, `None` on any failure (network error, missing field, registry
/// down). This function never errors — all failures are silently swallowed
/// because registry verification is advisory.
///
/// The registry endpoint is `GET /api/registry/{url_encoded_path}` which
/// returns package detail including a `versions` array with `contentHash`.
pub fn fetch_registry_hash(package_path: &str, version: &str) -> Option<String> {
    let base_url = registry_base_url()?;
    let encoded = percent_encode(package_path);
    let url = format!("{}/api/registry/{}", base_url, encoded);

    let resp = ureq::get(&url)
        .header("User-Agent", "ship-pkg/0.1")
        .call()
        .ok()?;

    if resp.status() != 200 {
        return None;
    }

    let body: serde_json::Value = resp.into_body().read_json().ok()?;
    find_version_hash(&body, version)
}

/// Resolve the registry base URL from env or default.
///
/// Returns `None` if `SHIP_REGISTRY_URL` is explicitly set to empty (opt-out).
fn registry_base_url() -> Option<String> {
    match std::env::var("SHIP_REGISTRY_URL") {
        Ok(val) if val.is_empty() => None,
        Ok(val) => Some(val),
        Err(_) => Some("https://getship.dev".to_string()),
    }
}

/// Search the registry JSON response for a matching version's `contentHash`.
///
/// The response shape is:
/// ```json
/// {
///   "versions": [
///     { "tag": "v1.0.0", "contentHash": "sha256:abc..." },
///     ...
///   ]
/// }
/// ```
fn find_version_hash(body: &serde_json::Value, version: &str) -> Option<String> {
    let versions = body.get("versions")?.as_array()?;
    for v in versions {
        let tag = v.get("tag").and_then(|t| t.as_str()).unwrap_or("");
        if tag == version {
            return v
                .get("contentHash")
                .and_then(|h| h.as_str())
                .map(|s| s.to_string());
        }
    }
    None
}

/// Compare a locally computed hash against the registry's expected hash.
///
/// Returns a warning message if they differ, `None` if they match or if the
/// registry hash is unavailable.
pub fn check_registry_hash(
    package_path: &str,
    version: &str,
    local_hash: &str,
    offline: bool,
) -> Option<String> {
    if offline {
        return None;
    }

    let registry_hash = fetch_registry_hash(package_path, version)?;

    if registry_hash != local_hash {
        Some(format!(
            "warning: content hash for {}@{} differs from registry \
             (local={}, registry={}) — the package may have been modified \
             since publication",
            package_path, version, local_hash, registry_hash
        ))
    } else {
        None
    }
}

#[cfg(test)]
#[path = "verify_tests.rs"]
mod tests;
