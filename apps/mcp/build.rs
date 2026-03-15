use std::env;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let base_version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_string());
    let git_hash =
        git_output(&["rev-parse", "--short", "HEAD"]).unwrap_or_else(|| "unknown".to_string());
    let commit_count =
        git_output(&["rev-list", "--count", "HEAD"]).unwrap_or_else(|| "0".to_string());
    let dirty = git_is_dirty();
    let build_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    let version_string = build_version_string(&base_version, &commit_count, &git_hash, dirty);

    println!("cargo:rustc-env=SHIP_MCP_VERSION_STRING={}", version_string);
    println!("cargo:rustc-env=SHIP_MCP_GIT_SHA={}", git_hash);
    println!("cargo:rustc-env=SHIP_MCP_GIT_COMMIT_COUNT={}", commit_count);
    println!(
        "cargo:rustc-env=SHIP_MCP_GIT_DIRTY={}",
        if dirty { "1" } else { "0" }
    );
    println!("cargo:rustc-env=SHIP_MCP_BUILD_TIMESTAMP={}", build_time);
    println!("cargo:rerun-if-changed=../../.git/HEAD");
}

fn git_output(args: &[&str]) -> Option<String> {
    Command::new("git")
        .args(args)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn git_is_dirty() -> bool {
    Command::new("git")
        .args(["status", "--porcelain", "--untracked-files=no"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| !output.stdout.is_empty())
        .unwrap_or(false)
}

fn build_version_string(base: &str, count: &str, hash: &str, dirty: bool) -> String {
    if hash == "unknown" {
        return base.to_string();
    }
    if dirty {
        format!("{}+rev.{}.{}.dirty", base, count, hash)
    } else {
        format!("{}+rev.{}.{}", base, count, hash)
    }
}
