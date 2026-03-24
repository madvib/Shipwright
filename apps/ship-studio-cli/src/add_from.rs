//! `ship add --from <url>` — import agent config from a Studio share link via MCP.
//!
//! Connects to the Studio's MCP server endpoint over HTTP/SSE, calls the
//! `transfer/bundle` tool to receive the agent config and skill files, writes
//! them to `.ship/`, and runs `ship install` for any registry dependencies.
//!
//! Fallback: if the URL returns plain JSON (non-MCP), treats it as a static
//! config bundle and writes files directly.

use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

// ── Transfer payload types ──────────────────────────────────────────────────

/// The complete transfer bundle returned by the Studio MCP server.
#[derive(Debug, Deserialize, Serialize)]
pub struct TransferBundle {
    /// Agent profile to write as `.ship/agents/<id>.jsonc`.
    pub agent: AgentBundle,
    /// Registry dependencies to merge into `ship.toml`/`ship.jsonc`.
    #[serde(default)]
    pub dependencies: std::collections::HashMap<String, String>,
    /// Inline skill files (non-registry). Key = skill id.
    #[serde(default)]
    pub skills: std::collections::HashMap<String, SkillBundle>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AgentBundle {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub rules: Vec<String>,
    #[serde(default)]
    pub mcp_servers: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SkillBundle {
    /// Map of relative path → file content.
    pub files: std::collections::HashMap<String, String>,
}

// ── Public entry point ──────────────────────────────────────────────────────

/// Run `ship add --from <url>`.
pub fn run_add_from(url: &str) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let ship_dir = cwd.join(".ship");
    if !ship_dir.exists() {
        anyhow::bail!(".ship/ not found. Run `ship init` first.");
    }

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    let bundle = rt.block_on(fetch_bundle(url))?;
    write_bundle(&cwd, &bundle)?;

    let (agents, skills, deps) = (1, bundle.skills.len(), bundle.dependencies.len());
    println!(
        "imported agent '{}': {} skill(s) inline, {} dep(s)",
        bundle.agent.id, skills, deps
    );

    if deps > 0 {
        println!("running ship install...");
        crate::install::run_install(&cwd, false, false)?;
    }

    // Compile if an agent is active.
    let state = crate::profile::WorkspaceState::load(&ship_dir);
    crate::compile::run_compile(crate::compile::CompileOptions {
        project_root: &cwd,
        output_root: None,
        provider: None,
        dry_run: false,
        active_agent: state.active_agent.as_deref(),
    })
    .ok(); // Non-fatal — user can compile manually.

    println!(
        "\nadded {} agent, {} skill(s), {} dep(s)",
        agents, skills, deps
    );
    println!("activate with: ship use {}", bundle.agent.id);
    Ok(())
}

// ── Fetch ───────────────────────────────────────────────────────────────────

/// Try MCP transport first, fall back to plain JSON GET.
async fn fetch_bundle(url: &str) -> Result<TransferBundle> {
    // Try MCP Streamable HTTP first.
    match fetch_via_mcp(url).await {
        Ok(bundle) => return Ok(bundle),
        Err(e) => {
            tracing::debug!("MCP transport failed, trying plain JSON: {e}");
        }
    }

    // Fallback: plain JSON GET.
    fetch_via_json(url).await
}

/// Connect to Studio MCP server, call `transfer/bundle`, parse response.
async fn fetch_via_mcp(url: &str) -> Result<TransferBundle> {
    use rmcp::model::{CallToolRequestParams, ClientInfo, Implementation, RawContent};
    use rmcp::transport::StreamableHttpClientTransport;
    use rmcp::ServiceExt;

    let transport = StreamableHttpClientTransport::from_uri(url);

    let client_info = ClientInfo {
        client_info: Implementation {
            name: "ship-cli".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            ..Default::default()
        },
        ..Default::default()
    };

    let client = client_info
        .serve(transport)
        .await
        .map_err(|e| anyhow::anyhow!("MCP connection failed: {e:?}"))?;

    // Call the transfer/bundle tool.
    let result = client
        .call_tool(CallToolRequestParams {
            name: "transfer/bundle".into(),
            arguments: None,
            meta: None,
            task: None,
        })
        .await
        .map_err(|e| anyhow::anyhow!("transfer/bundle tool call failed: {e:?}"))?;

    // Extract text content from the tool result.
    let text = result
        .content
        .iter()
        .filter_map(|c| match &c.raw {
            RawContent::Text(t) => Some(t.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("");

    if text.is_empty() {
        anyhow::bail!("transfer/bundle returned empty response");
    }

    let bundle: TransferBundle =
        serde_json::from_str(&text).context("parsing transfer bundle from MCP response")?;

    // Security scan the bundle content before accepting.
    scan_bundle_security(&bundle)?;

    // Clean shutdown.
    let _ = client.cancel().await;

    Ok(bundle)
}

/// Plain JSON GET fallback.
async fn fetch_via_json(url: &str) -> Result<TransferBundle> {
    let resp = reqwest::get(url)
        .await
        .map_err(|e| anyhow::anyhow!("failed to fetch {url}: {e}"))?;

    if !resp.status().is_success() {
        anyhow::bail!("GET {url} returned HTTP {}", resp.status());
    }

    let text = resp
        .text()
        .await
        .context("reading response body")?;

    let bundle: TransferBundle =
        serde_json::from_str(&text).context("parsing transfer bundle JSON")?;

    scan_bundle_security(&bundle)?;

    Ok(bundle)
}

// ── Security ────────────────────────────────────────────────────────────────

/// Scan all inline content for hidden Unicode characters before writing.
fn scan_bundle_security(bundle: &TransferBundle) -> Result<()> {
    let mut all_findings = Vec::new();

    for (skill_id, skill) in &bundle.skills {
        for (path, content) in &skill.files {
            let filename = format!("skills/{skill_id}/{path}");
            let findings = runtime::security::scan_text(content, &filename);
            all_findings.extend(findings);
        }
    }

    // Also scan rules (they're inline prompt content).
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
            "security scan blocked import: {} critical finding(s):\n  {}",
            critical.len(),
            critical.join("\n  ")
        );
    }

    let (_, warnings, _) = runtime::security::summarize(&all_findings);
    if warnings > 0 {
        eprintln!(
            "warning: {} suspicious Unicode character(s) found in bundle (non-blocking)",
            warnings
        );
    }

    Ok(())
}

// ── Write ───────────────────────────────────────────────────────────────────

/// Write the transfer bundle to `.ship/`.
fn write_bundle(project_root: &Path, bundle: &TransferBundle) -> Result<()> {
    let ship_dir = project_root.join(".ship");

    // 1. Write agent profile as JSONC.
    write_agent(&ship_dir, &bundle.agent)?;

    // 2. Write inline skills.
    for (skill_id, skill) in &bundle.skills {
        write_skill(&ship_dir, skill_id, skill)?;
    }

    // 3. Merge dependencies into manifest.
    if !bundle.dependencies.is_empty() {
        merge_dependencies(&ship_dir, &bundle.dependencies)?;
    }

    Ok(())
}

/// Write an agent JSONC profile.
fn write_agent(ship_dir: &Path, agent: &AgentBundle) -> Result<()> {
    let agents_dir = ship_dir.join("agents");
    std::fs::create_dir_all(&agents_dir)?;

    let dest = agents_dir.join(format!("{}.jsonc", agent.id));
    if dest.exists() {
        eprintln!(
            "warning: overwriting existing agent '{}'",
            agent.id
        );
    }

    let profile = build_agent_jsonc(agent);
    std::fs::write(&dest, profile)
        .with_context(|| format!("writing agent {}", dest.display()))?;

    Ok(())
}

/// Build JSONC content for an agent profile.
fn build_agent_jsonc(agent: &AgentBundle) -> String {
    let mut obj = serde_json::Map::new();
    obj.insert("id".into(), serde_json::json!(agent.id));
    if let Some(ref name) = agent.name {
        obj.insert("name".into(), serde_json::json!(name));
    }
    if let Some(ref desc) = agent.description {
        obj.insert("description".into(), serde_json::json!(desc));
    }
    if let Some(ref model) = agent.model {
        obj.insert("model".into(), serde_json::json!(model));
    }
    if !agent.skills.is_empty() {
        obj.insert("skills".into(), serde_json::json!(agent.skills));
    }
    if !agent.rules.is_empty() {
        obj.insert("rules".into(), serde_json::json!(agent.rules));
    }
    if !agent.mcp_servers.is_empty() {
        obj.insert("mcp_servers".into(), serde_json::json!(agent.mcp_servers));
    }

    serde_json::to_string_pretty(&obj).unwrap_or_else(|_| "{}".into())
}

/// Write inline skill files to `.ship/skills/<id>/`.
fn write_skill(ship_dir: &Path, skill_id: &str, skill: &SkillBundle) -> Result<()> {
    let skill_dir = ship_dir.join("skills").join(skill_id);
    std::fs::create_dir_all(&skill_dir)?;

    for (rel_path, content) in &skill.files {
        let dest = skill_dir.join(rel_path);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&dest, content)
            .with_context(|| format!("writing skill file {}", dest.display()))?;
    }

    Ok(())
}

/// Merge dependencies into the existing manifest (ship.jsonc or ship.toml).
fn merge_dependencies(
    ship_dir: &Path,
    deps: &std::collections::HashMap<String, String>,
) -> Result<()> {
    let jsonc_path = ship_dir.join("ship.jsonc");
    let toml_path = ship_dir.join("ship.toml");

    let manifest_path = if jsonc_path.exists() {
        jsonc_path
    } else if toml_path.exists() {
        toml_path
    } else {
        anyhow::bail!("no ship.jsonc or ship.toml found to add dependencies to");
    };

    let raw = std::fs::read_to_string(&manifest_path)?;
    let is_jsonc = crate::paths::is_jsonc_ext(&manifest_path);

    let mut updated = raw;
    for (path, version) in deps {
        if updated.contains(path) {
            continue; // Already present.
        }
        updated = if is_jsonc {
            crate::add::append_dependency_jsonc(&updated, path, version)
        } else {
            crate::add::append_dependency(&updated, path, version)
        };
    }

    std::fs::write(&manifest_path, &updated)
        .with_context(|| format!("writing {}", manifest_path.display()))?;

    Ok(())
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_bundle() -> TransferBundle {
        TransferBundle {
            agent: AgentBundle {
                id: "test-agent".into(),
                name: Some("Test Agent".into()),
                description: Some("A test agent".into()),
                model: Some("sonnet".into()),
                skills: vec!["tdd".into(), "@ship/skills/backend-rust".into()],
                rules: vec!["always write tests".into()],
                mcp_servers: vec![],
            },
            dependencies: [("@ship/skills".into(), "^0.1.0".into())]
                .into_iter()
                .collect(),
            skills: [(
                "tdd".into(),
                SkillBundle {
                    files: [(
                        "SKILL.md".into(),
                        "---\nname: tdd\n---\nWrite tests first.".into(),
                    )]
                    .into_iter()
                    .collect(),
                },
            )]
            .into_iter()
            .collect(),
        }
    }

    #[test]
    fn build_agent_jsonc_output() {
        let bundle = make_bundle();
        let jsonc = build_agent_jsonc(&bundle.agent);
        let parsed: serde_json::Value = serde_json::from_str(&jsonc).unwrap();
        assert_eq!(parsed["id"], "test-agent");
        assert_eq!(parsed["skills"][0], "tdd");
        assert_eq!(parsed["rules"][0], "always write tests");
    }

    #[test]
    fn write_bundle_creates_files() {
        let tmp = TempDir::new().unwrap();
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(ship_dir.join("skills")).unwrap();
        // Create a minimal ship.jsonc for dep merging.
        std::fs::write(
            ship_dir.join("ship.jsonc"),
            "{\n  \"module\": { \"name\": \"test\", \"version\": \"0.1.0\" }\n}",
        )
        .unwrap();

        let bundle = make_bundle();
        write_bundle(tmp.path(), &bundle).unwrap();

        // Agent file written.
        let agent_path = ship_dir.join("agents/test-agent.jsonc");
        assert!(agent_path.exists(), "agent file must exist");
        let agent_content = std::fs::read_to_string(&agent_path).unwrap();
        assert!(agent_content.contains("test-agent"));

        // Skill files written.
        let skill_path = ship_dir.join("skills/tdd/SKILL.md");
        assert!(skill_path.exists(), "skill file must exist");
        let skill_content = std::fs::read_to_string(&skill_path).unwrap();
        assert!(skill_content.contains("Write tests first"));

        // Dependency merged.
        let manifest = std::fs::read_to_string(ship_dir.join("ship.jsonc")).unwrap();
        assert!(
            manifest.contains("@ship/skills"),
            "dependency must be merged"
        );
    }

    #[test]
    fn security_scan_blocks_critical() {
        let mut bundle = make_bundle();
        // Inject a bidi override into a skill file.
        bundle.skills.get_mut("tdd").unwrap().files.insert(
            "SKILL.md".into(),
            format!("normal \u{202E} hidden"),
        );
        let err = scan_bundle_security(&bundle).unwrap_err();
        assert!(
            err.to_string().contains("security scan blocked"),
            "got: {err}"
        );
    }

    #[test]
    fn security_scan_passes_clean() {
        let bundle = make_bundle();
        scan_bundle_security(&bundle).unwrap();
    }

    #[test]
    fn parse_transfer_bundle_json() {
        let json = r#"{
            "agent": {
                "id": "rust-expert",
                "name": "Rust Expert",
                "skills": ["tdd"],
                "rules": []
            },
            "dependencies": { "@ship/skills": "^0.1.0" },
            "skills": {
                "tdd": {
                    "files": { "SKILL.md": "test content" }
                }
            }
        }"#;
        let bundle: TransferBundle = serde_json::from_str(json).unwrap();
        assert_eq!(bundle.agent.id, "rust-expert");
        assert_eq!(bundle.skills.len(), 1);
        assert_eq!(bundle.dependencies.len(), 1);
    }
}
