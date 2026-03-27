//! `ship add --from <url>` — import agent config from a Studio share link.
//!
//! Connects to the Studio's MCP server endpoint over Streamable HTTP, calls
//! `transfer_bundle` to receive agent config + skill files, writes to `.ship/`,
//! and runs `ship install` for registry dependencies.
//!
//! Fallback: if MCP transport fails, tries plain JSON GET.

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
    crate::add_from_write::write_bundle(&cwd, &bundle)?;

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
    match fetch_via_mcp(url).await {
        Ok(bundle) => return Ok(bundle),
        Err(e) => {
            tracing::debug!("MCP transport failed, trying plain JSON: {e}");
        }
    }
    fetch_via_json(url).await
}

/// Connect to Studio MCP server, call `transfer_bundle`, parse response.
async fn fetch_via_mcp(url: &str) -> Result<TransferBundle> {
    use rmcp::ServiceExt;
    use rmcp::model::{CallToolRequestParams, ClientInfo, Implementation, RawContent};
    use rmcp::transport::StreamableHttpClientTransport;

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

    let result = client
        .call_tool(CallToolRequestParams {
            name: "transfer_bundle".into(),
            arguments: None,
            meta: None,
            task: None,
        })
        .await
        .map_err(|e| anyhow::anyhow!("transfer_bundle call failed: {e:?}"))?;

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
        anyhow::bail!("transfer_bundle returned empty response");
    }

    let bundle: TransferBundle =
        serde_json::from_str(&text).context("parsing transfer bundle from MCP response")?;

    scan_bundle_security(&bundle)?;

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

    let text = resp.text().await.context("reading response body")?;
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

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
#[path = "add_from_tests.rs"]
mod tests;
