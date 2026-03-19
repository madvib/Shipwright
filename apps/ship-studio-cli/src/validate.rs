//! `ship validate` — check .ship/ config for errors before compile.

use anyhow::Result;
use serde::Serialize;
use std::path::Path;

use crate::loader::load_permission_preset;
use crate::mcp::{McpEntry, McpFile};
use crate::mode::Profile;
use crate::profile::find_profile_file;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ValidationError {
    pub profile: String,
    pub file: String,
    pub error: String,
}

/// Per-profile result: name + collected errors.
#[derive(Debug)]
pub struct ProfileReport {
    pub profile_id: String,
    pub errors: Vec<ValidationError>,
}

// ── Public entry points ───────────────────────────────────────────────────────

/// Run validation for one or all profiles. Print results. Return Err if any error found.
pub fn run_validate(profile_id: Option<&str>, json: bool, project_root: &Path) -> Result<()> {
    let ship_dir = project_root.join(".ship");
    if !ship_dir.exists() {
        anyhow::bail!(".ship/ not found. Run: ship init");
    }
    let agents_dir = ship_dir.join("agents");

    let reports = if let Some(id) = profile_id {
        let profile_path = find_profile_file(id, project_root)
            .ok_or_else(|| anyhow::anyhow!("Profile '{}' not found", id))?;
        vec![validate_profile(id, &profile_path, &agents_dir)]
    } else {
        validate_all(&agents_dir, project_root)
    };

    let all_errors: Vec<&ValidationError> = reports.iter()
        .flat_map(|r| r.errors.iter())
        .collect();

    if json {
        println!("{}", serde_json::to_string_pretty(&all_errors)?);
        if !all_errors.is_empty() {
            anyhow::bail!("");
        }
        return Ok(());
    }

    let mut any_errors = false;
    for report in &reports {
        if report.errors.is_empty() {
            println!("✓ profile {} — valid", report.profile_id);
        } else {
            any_errors = true;
            println!("✗ profile {} — {} error{}", report.profile_id, report.errors.len(),
                if report.errors.len() == 1 { "" } else { "s" });
            for e in &report.errors {
                println!("  {} — {}", e.file, e.error);
            }
        }
    }

    if any_errors {
        anyhow::bail!("validation failed");
    }
    Ok(())
}

// ── Core validation ───────────────────────────────────────────────────────────

/// Validate all profiles found in agents/profiles/.
fn validate_all(agents_dir: &Path, project_root: &Path) -> Vec<ProfileReport> {
    let profiles_dir = agents_dir.join("profiles");
    if !profiles_dir.exists() {
        return vec![];
    }
    let mut reports = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&profiles_dir) {
        let mut paths: Vec<_> = entries.flatten()
            .filter(|e| e.path().extension().is_some_and(|x| x == "toml"))
            .collect();
        paths.sort_by_key(|e| e.file_name());
        for entry in paths {
            let path = entry.path();
            let id = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
            reports.push(validate_profile(&id, &path, agents_dir));
        }
    }
    // Also check any compat presets dir
    let presets_dir = agents_dir.join("presets");
    if presets_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&presets_dir) {
            let mut paths: Vec<_> = entries.flatten()
                .filter(|e| e.path().extension().is_some_and(|x| x == "toml"))
                .collect();
            paths.sort_by_key(|e| e.file_name());
            for entry in paths {
                let path = entry.path();
                let id = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                // Skip if already found in agents/profiles/
                if find_profile_file(&id, project_root)
                    .is_some_and(|p| p.to_string_lossy().contains("agents/profiles"))
                {
                    continue;
                }
                reports.push(validate_profile(&id, &path, agents_dir));
            }
        }
    }
    reports
}

/// Validate a single profile at `profile_path` against `agents_dir`.
pub fn validate_profile(profile_id: &str, profile_path: &Path, agents_dir: &Path) -> ProfileReport {
    let mut errors = Vec::new();

    // 1. Parse TOML
    let profile = match std::fs::read_to_string(profile_path)
        .map_err(|e| e.to_string())
        .and_then(|s| toml::from_str::<Profile>(&s).map_err(|e| e.to_string()))
    {
        Ok(p) => p,
        Err(e) => {
            errors.push(ValidationError {
                profile: profile_id.to_string(),
                file: profile_path.display().to_string(),
                error: format!("TOML parse error: {}", e),
            });
            return ProfileReport { profile_id: profile_id.to_string(), errors };
        }
    };

    let profile_file = profile_path.display().to_string();

    // 2. Skill refs exist in agents/skills/
    let skills_dir = agents_dir.join("skills");
    for skill_id in &profile.skills.refs {
        if !skill_ref_exists(&skills_dir, skill_id) {
            errors.push(ValidationError {
                profile: profile_id.to_string(),
                file: profile_file.clone(),
                error: format!("skill '{}' not found in agents/skills/", skill_id),
            });
        }
    }

    // 3. MCP server refs exist in mcp.toml AND have required fields
    let mcp_path = agents_dir.join("mcp.toml");
    let mcp_file = load_mcp_file(&mcp_path);
    for server_id in &profile.mcp.servers {
        match mcp_file.servers.iter().find(|s| &s.id == server_id) {
            None => errors.push(ValidationError {
                profile: profile_id.to_string(),
                file: "agents/mcp.toml".to_string(),
                error: format!("mcp server '{}' not found in mcp.toml", server_id),
            }),
            Some(s) => {
                if let Some(e) = check_mcp_entry(s) {
                    errors.push(ValidationError {
                        profile: profile_id.to_string(),
                        file: "agents/mcp.toml".to_string(),
                        error: format!("mcp.servers.{} — {}", server_id, e),
                    });
                }
            }
        }
    }

    // 4. Validate all mcp.toml entries (regardless of profile refs)
    for server in &mcp_file.servers {
        if let Some(e) = check_mcp_entry(server) {
            errors.push(ValidationError {
                profile: profile_id.to_string(),
                file: "agents/mcp.toml".to_string(),
                error: format!("mcp.servers.{} — {}", server.id, e),
            });
        }
    }
    // Deduplicate errors from mcp entry checks (a server might be caught twice)
    errors.dedup_by(|a, b| a.file == b.file && a.error == b.error);

    // 5. permissions.preset references a known preset
    if let Some(preset_name) = &profile.permissions.preset {
        if preset_name.trim().is_empty() {
            errors.push(ValidationError {
                profile: profile_id.to_string(),
                file: profile_file.clone(),
                error: "permissions.preset must be a non-empty string".to_string(),
            });
        } else {
            let perm_path = agents_dir.join("permissions.toml");
            if perm_path.exists() && load_permission_preset(agents_dir, preset_name).is_none() {
                errors.push(ValidationError {
                    profile: profile_id.to_string(),
                    file: "agents/permissions.toml".to_string(),
                    error: format!("preset '{}' not found in permissions.toml", preset_name),
                });
            }
        }
    }

    ProfileReport { profile_id: profile_id.to_string(), errors }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn skill_ref_exists(skills_dir: &Path, skill_id: &str) -> bool {
    if !skills_dir.exists() { return false; }
    // Flat: skills/<id>.md
    if skills_dir.join(format!("{}.md", skill_id)).exists() { return true; }
    // Dir: skills/<id>/SKILL.md
    if skills_dir.join(skill_id).join("SKILL.md").exists() { return true; }
    false
}

fn load_mcp_file(path: &Path) -> McpFile {
    McpFile::load(path).unwrap_or_default()
}

/// Returns an error message if the entry is missing required fields; None if valid.
fn check_mcp_entry(s: &McpEntry) -> Option<String> {
    let is_http = s.url.is_some() && s.command.is_none()
        || s.server_type.as_deref() == Some("http")
        || s.server_type.as_deref() == Some("sse");
    if is_http {
        if s.url.as_deref().map_or(true, |u| u.trim().is_empty()) {
            return Some("HTTP/SSE server missing 'url' field".to_string());
        }
    } else {
        // stdio
        if s.command.as_deref().map_or(true, |c| c.trim().is_empty()) {
            return Some("stdio server missing 'command' field".to_string());
        }
    }
    None
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

    fn valid_profile_toml() -> &'static str {
        r#"[profile]
name = "test"
id = "test"
providers = ["claude"]
[skills]
refs = []
[mcp]
servers = []
[permissions]
preset = "ship-standard"
"#
    }

    fn setup() -> (TempDir, std::path::PathBuf) {
        let tmp = TempDir::new().unwrap();
        let agents_dir = tmp.path().join(".ship").join("agents");
        std::fs::create_dir_all(&agents_dir).unwrap();
        std::fs::create_dir_all(agents_dir.join("profiles")).unwrap();
        write(tmp.path(), ".ship/agents/permissions.toml",
            "[ship-standard]\ndefault_mode = \"acceptEdits\"\n");
        (tmp, agents_dir)
    }

    #[test]
    fn valid_profile_passes() {
        let (tmp, agents_dir) = setup();
        write(tmp.path(), ".ship/agents/profiles/test.toml", valid_profile_toml());
        let profile_path = agents_dir.join("profiles").join("test.toml");
        let report = validate_profile("test", &profile_path, &agents_dir);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
    }

    #[test]
    fn malformed_toml_reports_error() {
        let (tmp, agents_dir) = setup();
        write(tmp.path(), ".ship/agents/profiles/bad.toml", "this is not [[valid toml");
        let profile_path = agents_dir.join("profiles").join("bad.toml");
        let report = validate_profile("bad", &profile_path, &agents_dir);
        assert_eq!(report.errors.len(), 1);
        assert!(report.errors[0].error.contains("TOML parse error"), "{:?}", report.errors[0].error);
    }

    #[test]
    fn missing_skill_ref_reports_error() {
        let (tmp, agents_dir) = setup();
        let toml = r#"[profile]
name = "test"
id = "test"
providers = ["claude"]
[skills]
refs = ["nonexistent-skill"]
[mcp]
servers = []
"#;
        write(tmp.path(), ".ship/agents/profiles/test.toml", toml);
        let report = validate_profile("test", &agents_dir.join("profiles").join("test.toml"), &agents_dir);
        assert!(report.errors.iter().any(|e| e.error.contains("nonexistent-skill")), "{:?}", report.errors);
    }

    #[test]
    fn existing_skill_ref_passes() {
        let (tmp, agents_dir) = setup();
        write(tmp.path(), ".ship/agents/skills/my-skill.md", "---\nname: My Skill\n---\nContent.");
        let toml = r#"[profile]
name = "test"
id = "test"
providers = ["claude"]
[skills]
refs = ["my-skill"]
[mcp]
servers = []
"#;
        write(tmp.path(), ".ship/agents/profiles/test.toml", toml);
        let report = validate_profile("test", &agents_dir.join("profiles").join("test.toml"), &agents_dir);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
    }

    #[test]
    fn stdio_mcp_missing_command_reports_error() {
        let (tmp, agents_dir) = setup();
        write(tmp.path(), ".ship/agents/mcp.toml", r#"
[[servers]]
id = "bad-stdio"
server_type = "stdio"
"#);
        write(tmp.path(), ".ship/agents/profiles/test.toml", valid_profile_toml());
        let report = validate_profile("test", &agents_dir.join("profiles").join("test.toml"), &agents_dir);
        assert!(report.errors.iter().any(|e| e.error.contains("missing 'command'")), "{:?}", report.errors);
    }

    #[test]
    fn http_mcp_missing_url_reports_error() {
        let (tmp, agents_dir) = setup();
        write(tmp.path(), ".ship/agents/mcp.toml", r#"
[[servers]]
id = "bad-http"
server_type = "http"
"#);
        write(tmp.path(), ".ship/agents/profiles/test.toml", valid_profile_toml());
        let report = validate_profile("test", &agents_dir.join("profiles").join("test.toml"), &agents_dir);
        assert!(report.errors.iter().any(|e| e.error.contains("missing 'url'")), "{:?}", report.errors);
    }

    #[test]
    fn unknown_permissions_preset_reports_error() {
        let (tmp, agents_dir) = setup();
        let toml = r#"[profile]
name = "test"
id = "test"
providers = ["claude"]
[skills]
refs = []
[mcp]
servers = []
[permissions]
preset = "nonexistent-preset"
"#;
        write(tmp.path(), ".ship/agents/profiles/test.toml", toml);
        let report = validate_profile("test", &agents_dir.join("profiles").join("test.toml"), &agents_dir);
        assert!(report.errors.iter().any(|e| e.error.contains("nonexistent-preset")), "{:?}", report.errors);
    }

    #[test]
    fn run_validate_exits_nonzero_on_errors() {
        let (tmp, agents_dir) = setup();
        write(tmp.path(), ".ship/agents/profiles/bad.toml", "not valid toml [[");
        let result = run_validate(None, false, tmp.path());
        assert!(result.is_err());
    }

    #[test]
    fn run_validate_json_flag_emits_array() {
        let (tmp, agents_dir) = setup();
        write(tmp.path(), ".ship/agents/profiles/bad.toml", "not valid toml [[");
        // json mode returns Err (errors found) but prints JSON — we just check no panic
        let _ = run_validate(None, true, tmp.path());
    }

    #[test]
    fn run_validate_passes_on_valid_config() {
        let (tmp, agents_dir) = setup();
        write(tmp.path(), ".ship/agents/profiles/test.toml", valid_profile_toml());
        let result = run_validate(None, false, tmp.path());
        assert!(result.is_ok(), "{:?}", result);
    }
}
