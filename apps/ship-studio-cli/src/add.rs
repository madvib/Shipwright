//! `ship add <package>[@version]` — add a registry dependency to ship.toml.
//!
//! Parses the package spec, validates it's not already present, appends to
//! `.ship/ship.toml`, resolves and fetches only the new dep, updates
//! `.ship/ship.lock`, and re-compiles.
//!
//! On any error after modifying ship.toml, the file is restored from backup.

use std::path::Path;

use anyhow::{Context, Result};
use compiler::manifest::ShipManifest;
use runtime::registry::{
    PackageCache,
    install::{InstallOptions, resolve_and_fetch},
    types::{Dependency, ShipManifest as RegistryManifest},
};

// ── Public API ────────────────────────────────────────────────────────────────

/// Run `ship add <package>[@version]`.
///
/// 1. Parse package spec (path[@version]).
/// 2. Check dep not already in ship.toml.
/// 3. Backup ship.toml.
/// 4. Append dep to ship.toml [dependencies].
/// 5. Resolve + fetch the new dep only.
/// 6. Update ship.lock.
/// 7. Compile.
/// 8. On error after step 3, restore backup.
pub fn run_add(project_root: &Path, package_spec: &str) -> Result<()> {
    let ship_dir = project_root.join(".ship");
    // Prefer ship.jsonc over ship.toml
    let jsonc_path = ship_dir.join("ship.jsonc");
    let manifest_path = if jsonc_path.exists() { jsonc_path } else { ship_dir.join("ship.toml") };

    if !manifest_path.exists() {
        anyhow::bail!(
            "No .ship/ship.jsonc or .ship/ship.toml found. Run `ship init` first, then add \
             a [module]/\"module\" section to use ship add."
        );
    }

    let (pkg_path, version) = parse_package_spec(package_spec);

    let is_jsonc = crate::paths::is_jsonc_ext(&manifest_path);

    // Read and parse current manifest to check for duplicates.
    let raw_content = std::fs::read_to_string(&manifest_path)
        .context("reading ship manifest")?;
    let manifest = ShipManifest::from_file(&manifest_path).with_context(|| {
        "Failed to parse ship manifest. Ensure it has a module section with name and version."
    })?;

    // Check duplicate.
    if manifest.dependencies.contains_key(&pkg_path) {
        anyhow::bail!(
            "{} is already in dependencies",
            pkg_path
        );
    }

    // Backup before modifying.
    let backup = raw_content.clone();

    // Append dep (non-destructive — preserve existing content).
    let updated = if is_jsonc {
        append_dependency_jsonc(&raw_content, &pkg_path, &version)
    } else {
        append_dependency(&raw_content, &pkg_path, &version)
    };
    if let Err(e) = std::fs::write(&manifest_path, &updated) {
        anyhow::bail!("Failed to write {}: {e}", manifest_path.display());
    }

    // Resolve + fetch + lock + compile. Restore on failure.
    match do_add(project_root, &pkg_path, &version) {
        Ok(()) => Ok(()),
        Err(e) => {
            // Restore backup.
            let _ = std::fs::write(&manifest_path, &backup);
            Err(e.context("ship add failed; restored manifest"))
        }
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Parse `"github.com/owner/repo"` or `"github.com/owner/repo@^1.0.0"`.
///
/// Returns `(path, version)` where version defaults to `"main"`.
fn parse_package_spec(spec: &str) -> (String, String) {
    // Split on last `@` to allow `github.com/owner/repo@1.0.0`
    match spec.rfind('@') {
        Some(pos) => {
            let path = spec[..pos].to_string();
            let ver = spec[pos + 1..].to_string();
            (path, ver)
        }
        None => (spec.to_string(), "main".to_string()),
    }
}

/// Append a `"path" = "version"` entry to the [dependencies] section of a
/// TOML string, preserving all existing content.
///
/// If no [dependencies] section exists, appends one.
fn append_dependency(raw: &str, path: &str, version: &str) -> String {
    let dep_line = format!("\"{}\" = \"{}\"\n", path, version);

    // Check if [dependencies] section exists.
    if let Some(pos) = raw.find("[dependencies]") {
        // Find the end of the [dependencies] section: next top-level `[` header
        // or end of file.
        let after_header = pos + "[dependencies]".len();
        let rest = &raw[after_header..];

        // Find next section header `\n[` after the [dependencies] line.
        let section_end = rest.find("\n[").map(|p| after_header + p + 1);

        match section_end {
            Some(end) => {
                // Insert before next section.
                let mut out = raw[..end].to_string();
                out.push_str(&dep_line);
                out.push_str(&raw[end..]);
                out
            }
            None => {
                // [dependencies] is the last section — append at end.
                let mut out = raw.to_string();
                if !out.ends_with('\n') {
                    out.push('\n');
                }
                out.push_str(&dep_line);
                out
            }
        }
    } else {
        // No [dependencies] section — append one.
        let mut out = raw.to_string();
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str("\n[dependencies]\n");
        out.push_str(&dep_line);
        out
    }
}

/// Resolve, fetch, lock, and compile after modifying the manifest.
fn do_add(project_root: &Path, pkg_path: &str, _version: &str) -> Result<()> {
    let ship_dir = project_root.join(".ship");
    // Prefer ship.jsonc over ship.toml
    let jsonc_path = ship_dir.join("ship.jsonc");
    let manifest_path = if jsonc_path.exists() { jsonc_path } else { ship_dir.join("ship.toml") };
    let lock_path = ship_dir.join("ship.lock");

    // Re-parse updated manifest.
    let compiler_manifest = ShipManifest::from_file(&manifest_path)?;

    // Build registry manifest (only the new dep needs resolving — but
    // resolve_and_fetch handles partial re-resolution via lock comparison).
    let mut registry_manifest = RegistryManifest::default();
    for (path, dep_val) in &compiler_manifest.dependencies {
        let dep = dep_val.clone().into_dep();
        registry_manifest.dependencies.insert(
            path.clone(),
            Dependency {
                version: dep.version,
                grant: dep.grant,
            },
        );
    }

    let cache = PackageCache::new().context("initializing package cache")?;
    let opts = InstallOptions { frozen: false };

    let result = resolve_and_fetch(&registry_manifest, &lock_path, &cache, &opts)
        .with_context(|| format!("resolving {pkg_path}"))?;

    // Compile.
    let state = crate::profile::WorkspaceState::load(&ship_dir);
    crate::compile::run_compile(crate::compile::CompileOptions {
        project_root,
        provider: None,
        dry_run: false,
        active_agent: state.active_agent.as_deref(),
    })
    .context("compiling after add")?;

    let n = result.packages.len();
    println!(
        "added {}, {} package{} installed",
        pkg_path,
        n,
        if n == 1 { "" } else { "s" }
    );

    Ok(())
}

/// Append a dependency to a JSONC manifest string.
///
/// If a `"dependencies"` key exists, inserts before its closing `}`.
/// Otherwise, inserts a `"dependencies"` section before the final `}`.
fn append_dependency_jsonc(raw: &str, path: &str, version: &str) -> String {
    let dep_entry = format!("    \"{}\": \"{}\"", path, version);

    if let Some(deps_pos) = raw.find("\"dependencies\"") {
        // Find the closing brace of the dependencies object
        let after = &raw[deps_pos..];
        if let Some(open) = after.find('{') {
            let abs_open = deps_pos + open;
            // Find the matching closing brace
            let mut depth = 0;
            let mut close_pos = None;
            for (i, ch) in raw[abs_open..].char_indices() {
                match ch {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            close_pos = Some(abs_open + i);
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if let Some(close) = close_pos {
                let before_close = &raw[..close];
                let trimmed = before_close.trim_end();
                let needs_comma = trimmed.ends_with('"')
                    || trimmed.ends_with('}')
                    || trimmed.ends_with(']');
                let comma = if needs_comma { "," } else { "" };
                return format!("{}{}\n{}\n{}", before_close, comma, dep_entry, &raw[close..]);
            }
        }
    }

    // No "dependencies" section — insert one before the final }
    if let Some(last_brace) = raw.rfind('}') {
        let before = &raw[..last_brace];
        let trimmed = before.trim_end();
        let needs_comma = trimmed.ends_with('"')
            || trimmed.ends_with('}')
            || trimmed.ends_with(']');
        let comma = if needs_comma { "," } else { "" };
        return format!("{}{}\n  \"dependencies\": {{\n{}\n  }}\n}}", before, comma, dep_entry);
    }

    raw.to_string()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write(dir: &Path, rel: &str, content: &str) {
        let p = dir.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, content).unwrap();
    }

    #[test]
    fn parse_spec_no_at() {
        let (p, v) = parse_package_spec("github.com/owner/repo");
        assert_eq!(p, "github.com/owner/repo");
        assert_eq!(v, "main");
    }

    #[test]
    fn parse_spec_with_version() {
        let (p, v) = parse_package_spec("github.com/owner/repo@^1.0.0");
        assert_eq!(p, "github.com/owner/repo");
        assert_eq!(v, "^1.0.0");
    }

    #[test]
    fn parse_spec_with_commit_sha() {
        let sha = "abc1234567890abcdef1234567890abcdef1234";
        let spec = format!("github.com/owner/repo@{}", sha);
        let (p, v) = parse_package_spec(&spec);
        assert_eq!(p, "github.com/owner/repo");
        assert_eq!(v, sha);
    }

    #[test]
    fn append_dependency_creates_section() {
        let raw = "[module]\nname = \"x\"\nversion = \"1.0.0\"\n";
        let out = append_dependency(raw, "github.com/a/b", "main");
        assert!(out.contains("[dependencies]"), "must create [dependencies] section");
        assert!(out.contains("\"github.com/a/b\" = \"main\""));
    }

    #[test]
    fn append_dependency_adds_to_existing_section() {
        let raw = "[module]\nname = \"x\"\nversion = \"1.0.0\"\n\n[dependencies]\n\"github.com/a/b\" = \"main\"\n";
        let out = append_dependency(raw, "github.com/c/d", "^1.0.0");
        assert!(out.contains("\"github.com/a/b\" = \"main\""), "existing dep must remain");
        assert!(out.contains("\"github.com/c/d\" = \"^1.0.0\""), "new dep must be added");
    }

    #[test]
    fn append_dependency_section_not_last() {
        let raw = "[module]\nname = \"x\"\nversion = \"1.0.0\"\n\n[dependencies]\n\"github.com/a/b\" = \"main\"\n\n[exports]\nskills = []\n";
        let out = append_dependency(raw, "github.com/c/d", "main");
        assert!(out.contains("\"github.com/c/d\" = \"main\""), "new dep must be added");
        // exports section should still be present after
        assert!(out.contains("[exports]"), "exports section must be preserved");
        let dep_pos = out.find("\"github.com/c/d\"").unwrap();
        let exp_pos = out.find("[exports]").unwrap();
        assert!(dep_pos < exp_pos, "new dep must appear before [exports]");
    }

    #[test]
    fn add_no_ship_toml_errors() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".ship")).unwrap();
        let err = run_add(tmp.path(), "github.com/owner/repo").unwrap_err();
        assert!(err.to_string().contains("No .ship/ship.jsonc or .ship/ship.toml"), "got: {err}");
    }

    #[test]
    fn add_duplicate_dep_errors_without_modifying() {
        let tmp = TempDir::new().unwrap();
        write(
            tmp.path(),
            ".ship/ship.toml",
            "[module]\nname = \"github.com/test/repo\"\nversion = \"0.1.0\"\n\n[dependencies]\n\"github.com/owner/pkg\" = \"main\"\n",
        );
        let original = std::fs::read_to_string(tmp.path().join(".ship/ship.toml")).unwrap();
        let err = run_add(tmp.path(), "github.com/owner/pkg").unwrap_err();
        assert!(
            err.to_string().contains("already in dependencies"),
            "got: {err}"
        );
        // ship.toml must be unchanged
        let after = std::fs::read_to_string(tmp.path().join(".ship/ship.toml")).unwrap();
        assert_eq!(original, after, "ship.toml must be unchanged on duplicate error");
    }

    #[test]
    fn add_invalid_manifest_errors_without_modifying() {
        let tmp = TempDir::new().unwrap();
        // No [module] section — will fail manifest parsing
        write(
            tmp.path(),
            ".ship/ship.toml",
            "[defaults]\nproviders = [\"claude\"]\n",
        );
        let original = std::fs::read_to_string(tmp.path().join(".ship/ship.toml")).unwrap();
        let err = run_add(tmp.path(), "github.com/owner/pkg").unwrap_err();
        assert!(
            err.to_string().contains("[module]") || err.to_string().contains("module section"),
            "got: {err}"
        );
        let after = std::fs::read_to_string(tmp.path().join(".ship/ship.toml")).unwrap();
        assert_eq!(original, after, "ship.toml must be unchanged on parse error");
    }
}
