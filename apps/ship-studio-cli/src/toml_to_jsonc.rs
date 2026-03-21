//! TOML → JSONC converter for Ship config files.
//!
//! Converts `.ship/ship.toml`, agent TOML files, `mcp.toml`, and
//! `permissions.toml` to their JSONC equivalents.

use anyhow::Result;

/// Convert a TOML config string to pretty-printed JSONC.
///
/// Parses the TOML, converts to `serde_json::Value`, and emits
/// pretty-printed JSON (which is valid JSONC).
pub fn toml_to_jsonc(toml_str: &str) -> Result<String> {
    let val: toml::Value = toml::from_str(toml_str)
        .map_err(|e| anyhow::anyhow!("invalid TOML: {e}"))?;
    let json = toml_value_to_json(&val);
    Ok(serde_json::to_string_pretty(&json)?)
}

/// Convert a `toml::Value` to `serde_json::Value`.
fn toml_value_to_json(v: &toml::Value) -> serde_json::Value {
    match v {
        toml::Value::String(s) => serde_json::Value::String(s.clone()),
        toml::Value::Integer(i) => serde_json::json!(*i),
        toml::Value::Float(f) => serde_json::json!(*f),
        toml::Value::Boolean(b) => serde_json::Value::Bool(*b),
        toml::Value::Datetime(d) => serde_json::Value::String(d.to_string()),
        toml::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(toml_value_to_json).collect())
        }
        toml::Value::Table(t) => {
            let mut map = serde_json::Map::new();
            for (k, val) in t {
                map.insert(k.clone(), toml_value_to_json(val));
            }
            serde_json::Value::Object(map)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_ship_toml() {
        let toml = r#"
id = "test-id"

[module]
name = "github.com/owner/repo"
version = "0.1.0"
description = "A library"

[dependencies]
"github.com/a/b" = "main"

[exports]
skills = ["agents/skills/my-skill"]
agents = ["agents/profiles/my-agent.toml"]
"#;
        let jsonc = toml_to_jsonc(toml).unwrap();
        let v: serde_json::Value = serde_json::from_str(&jsonc).unwrap();
        assert_eq!(v["module"]["name"], "github.com/owner/repo");
        assert_eq!(v["module"]["version"], "0.1.0");
        assert_eq!(v["dependencies"]["github.com/a/b"], "main");
        assert_eq!(v["exports"]["skills"][0], "agents/skills/my-skill");
    }

    #[test]
    fn convert_agent_toml() {
        let toml = r#"
[agent]
id = "default"
name = "Ship Dev"
version = "0.1.0"
description = "Default dev preset"
providers = ["claude"]

[skills]
refs = []

[mcp]
servers = []

[plugins]
install = ["superpowers@claude-plugins-official"]
scope = "project"

[permissions]
preset = "ship-standard"
tools_deny = ["Bash(rm -rf *)"]
"#;
        let jsonc = toml_to_jsonc(toml).unwrap();
        let v: serde_json::Value = serde_json::from_str(&jsonc).unwrap();
        assert_eq!(v["agent"]["name"], "Ship Dev");
        assert_eq!(v["permissions"]["preset"], "ship-standard");
        assert_eq!(v["plugins"]["install"][0], "superpowers@claude-plugins-official");
    }

    #[test]
    fn convert_mcp_toml() {
        let toml = r#"
[mcp.servers.ship]
id = ""
name = "Ship"
command = "ship"
args = ["mcp"]
scope = "project"
server_type = "stdio"
disabled = false
timeout_secs = 30

[mcp.servers.ship.env]

[mcp.servers.github]
id = ""
name = "GitHub MCP"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-github"]
scope = "project"
server_type = "stdio"
disabled = true

[mcp.servers.github.env]
GITHUB_TOKEN = "${GITHUB_TOKEN}"
"#;
        let jsonc = toml_to_jsonc(toml).unwrap();
        let v: serde_json::Value = serde_json::from_str(&jsonc).unwrap();
        assert_eq!(v["mcp"]["servers"]["ship"]["name"], "Ship");
        assert_eq!(v["mcp"]["servers"]["github"]["env"]["GITHUB_TOKEN"], "${GITHUB_TOKEN}");
    }

    #[test]
    fn convert_permissions_toml() {
        let toml = r#"
[ship-readonly]
default_mode = "plan"
tools_allow = ["Read", "Glob", "Grep"]
tools_deny = ["Write(*)", "Edit(*)"]

[ship-standard]
default_mode = "default"
tools_allow = ["Read", "Glob", "Grep"]

[ship-autonomous]
default_mode = "dontAsk"
tools_allow = ["Read", "Glob", "Grep"]
"#;
        let jsonc = toml_to_jsonc(toml).unwrap();
        let v: serde_json::Value = serde_json::from_str(&jsonc).unwrap();
        assert_eq!(v["ship-readonly"]["default_mode"], "plan");
        assert_eq!(v["ship-standard"]["default_mode"], "default");
        assert_eq!(v["ship-autonomous"]["default_mode"], "dontAsk");
        assert_eq!(v["ship-readonly"]["tools_allow"][0], "Read");
    }

    #[test]
    fn round_trip_ship_toml() {
        let toml = r#"
[module]
name = "github.com/test/repo"
version = "1.0.0"

[dependencies]
"github.com/a/b" = "^1.0.0"
"#;
        let jsonc = toml_to_jsonc(toml).unwrap();
        // Parse JSONC back and verify structure
        let manifest = compiler::manifest::ShipManifest::from_jsonc_str(&jsonc).unwrap();
        assert_eq!(manifest.module.name, "github.com/test/repo");
        assert_eq!(manifest.module.version, "1.0.0");
        assert_eq!(manifest.dependencies.len(), 1);
    }

    #[test]
    fn round_trip_agent_toml() {
        let toml = r#"
[agent]
name = "test-agent"
rules = ["rules/style.md"]
skills = ["agents/skills/backend"]

[permissions]
allow = ["Bash", "Read"]
"#;
        let jsonc = toml_to_jsonc(toml).unwrap();
        let agent = compiler::agent_parser::AgentDef::from_jsonc_str(&jsonc).unwrap();
        assert_eq!(agent.name, "test-agent");
        assert_eq!(agent.rules, vec!["rules/style.md"]);
        assert_eq!(agent.permissions.allow, vec!["Bash", "Read"]);
    }
}
