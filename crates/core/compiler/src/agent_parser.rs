//! Agent TOML definition parser.
//!
//! Parses `.ship/agents/<name>.toml` (new format: `[agent]` section)
//! and also accepts the legacy `.ship/agents/profiles/<name>.toml` format
//! (`[profile]` section) without error.
//!
//! New format sections: [agent], [permissions], [[mcp]], [providers]
//! Legacy format sections: [profile], [skills], [mcp], [permissions], [rules]

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

// ── Sub-types ──────────────────────────────────────────────────────────────────

/// Tool permission grants for an agent.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AgentPermissions {
    /// List of allowed tool permission strings, e.g. `["Bash", "Read", "*"]`.
    #[serde(default)]
    pub allow: Vec<String>,
}

/// A single MCP server declaration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpDecl {
    /// Unique MCP server identifier.
    pub id: String,
    /// Command to run the MCP server.
    pub command: String,
    /// Optional arguments.
    #[serde(default)]
    pub args: Vec<String>,
    /// Optional environment variables. Values may contain `${VAR}` references.
    #[serde(default)]
    pub env: IndexMap<String, String>,
}

/// Provider targets and per-provider override tables.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AgentProviders {
    /// Target provider ids, e.g. `["claude", "cursor"]`.
    #[serde(default)]
    pub targets: Vec<String>,
    /// Per-provider override sub-tables, keyed by provider id.
    #[serde(default, flatten)]
    pub overrides: IndexMap<String, toml::Value>,
}

// ── AgentDef ───────────────────────────────────────────────────────────────────

/// A fully-parsed agent definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentDef {
    /// Agent name (should match filename without `.toml` extension).
    pub name: String,
    /// Optional human-readable description.
    #[serde(default)]
    pub description: Option<String>,
    /// Rule paths/references (local or dep-scoped).
    #[serde(default)]
    pub rules: Vec<String>,
    /// Skill paths/references (local or dep-scoped).
    #[serde(default)]
    pub skills: Vec<String>,
    /// Permission grants.
    #[serde(default)]
    pub permissions: AgentPermissions,
    /// MCP server declarations.
    #[serde(default)]
    pub mcp: Vec<McpDecl>,
    /// Provider targets and overrides.
    #[serde(default)]
    pub providers: AgentProviders,
}

// ── Raw TOML intermediary ──────────────────────────────────────────────────────

/// Raw TOML representation — handles both `[agent]` (new) and `[profile]` (legacy).
///
/// The `mcp` field accepts either:
/// - A TOML array of tables `[[mcp]]` (new format: `Vec<McpDecl>`)
/// - A TOML table `[mcp]` (legacy format, ignored — has `servers = [...]`)
///
/// We deserialise to `toml::Value` and convert manually.
#[derive(Debug, Deserialize)]
struct RawAgentFile {
    // New format
    agent: Option<RawAgentSection>,
    // Legacy format
    profile: Option<RawProfileSection>,
    #[serde(default)]
    permissions: RawPermissionsSection,
    /// Can be either `[[mcp]]` array or `[mcp]` table (legacy).
    #[serde(default)]
    mcp: Option<toml::Value>,
    #[serde(default)]
    providers: Option<toml::Value>,
    // Legacy fields
    #[serde(default)]
    rules: Option<RawRulesSection>,
    #[serde(default)]
    skills: Option<RawSkillsSection>,
}

#[derive(Debug, Deserialize)]
struct RawAgentSection {
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    rules: Vec<String>,
    #[serde(default)]
    skills: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RawProfileSection {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    providers: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
struct RawPermissionsSection {
    #[serde(default)]
    allow: Vec<String>,
    // Legacy fields — parsed to avoid "unknown key" errors but not used
    #[serde(default)]
    #[allow(dead_code)]
    preset: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    default_mode: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    tools_deny: Vec<String>,
    #[serde(default)]
    #[allow(dead_code)]
    tools_ask: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
struct RawRulesSection {
    #[serde(default)]
    inline: Option<String>,
    #[serde(default)]
    refs: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
struct RawSkillsSection {
    #[serde(default)]
    refs: Vec<String>,
}

// ── Public API ─────────────────────────────────────────────────────────────────

impl AgentDef {
    /// Parse an agent definition from a TOML string.
    ///
    /// Accepts both `[agent]` (new format) and `[profile]` (legacy format).
    /// Returns an error if neither section is present or if `name` is missing.
    pub fn from_toml_str(s: &str) -> anyhow::Result<Self> {
        let raw: RawAgentFile =
            toml::from_str(s).map_err(|e| anyhow::anyhow!("Failed to parse agent TOML: {e}"))?;
        Self::from_raw(raw, None)
    }

    /// Parse from a TOML string, using the file stem as a fallback name.
    pub fn from_toml_str_with_stem(s: &str, file_stem: &str) -> anyhow::Result<Self> {
        let raw: RawAgentFile =
            toml::from_str(s).map_err(|e| anyhow::anyhow!("Failed to parse agent TOML: {e}"))?;
        Self::from_raw(raw, Some(file_stem))
    }

    /// Read and parse from a TOML file path.
    ///
    /// Uses the file stem as a fallback when `[agent].name` is absent.
    pub fn from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Cannot read {}: {e}", path.display()))?;
        Self::from_toml_str_with_stem(&content, &stem)
    }

    // ── Internal ────────────────────────────────────────────────────────────────

    fn from_raw(raw: RawAgentFile, file_stem: Option<&str>) -> anyhow::Result<Self> {
        // Convert mcp field: [[mcp]] array → Vec<McpDecl>, [mcp] table → empty (legacy)
        let mcp_decls = Self::parse_mcp_field(raw.mcp)?;

        // Prefer [agent] section; fall back to [profile].
        if let Some(agent_sec) = raw.agent {
            // New format
            let name = agent_sec
                .name
                .or_else(|| file_stem.map(str::to_string))
                .filter(|n| !n.is_empty())
                .ok_or_else(|| anyhow::anyhow!("[agent].name is required in agent TOML"))?;

            let providers = Self::parse_providers(raw.providers);

            Ok(AgentDef {
                name,
                description: agent_sec.description,
                rules: agent_sec.rules,
                skills: agent_sec.skills,
                permissions: AgentPermissions {
                    allow: raw.permissions.allow,
                },
                mcp: mcp_decls,
                providers,
            })
        } else if let Some(profile) = raw.profile {
            // Legacy format — map to AgentDef
            let name = profile
                .name
                .or_else(|| profile.id.clone())
                .or_else(|| file_stem.map(str::to_string))
                .filter(|n| !n.is_empty())
                .ok_or_else(|| {
                    anyhow::anyhow!("[profile].name or [profile].id is required in agent TOML")
                })?;

            let rules: Vec<String> = raw
                .rules
                .as_ref()
                .map(|r| {
                    let mut v = r.refs.clone();
                    if let Some(inline) = &r.inline
                        && !inline.trim().is_empty()
                    {
                        v.push(format!("inline:{}", inline.trim()));
                    }
                    v
                })
                .unwrap_or_default();

            let skills: Vec<String> = raw
                .skills
                .as_ref()
                .map(|s| s.refs.clone())
                .unwrap_or_default();

            let mut providers = AgentProviders {
                targets: profile.providers.clone(),
                overrides: IndexMap::new(),
            };
            // Merge any parsed providers overrides
            let extra = Self::parse_providers(raw.providers);
            providers.overrides = extra.overrides;

            Ok(AgentDef {
                name,
                description: profile.description,
                rules,
                skills,
                permissions: AgentPermissions {
                    allow: raw.permissions.allow,
                },
                mcp: mcp_decls,
                providers,
            })
        } else {
            anyhow::bail!("Agent TOML must contain either an [agent] or [profile] section");
        }
    }

    /// Convert the `mcp` TOML value to a list of `McpDecl`.
    ///
    /// - TOML array of tables (`[[mcp]]`) → parse each entry as `McpDecl`
    /// - TOML table (`[mcp]`) → legacy format, return empty (no MCP decls)
    /// - Absent → empty
    fn parse_mcp_field(val: Option<toml::Value>) -> anyhow::Result<Vec<McpDecl>> {
        match val {
            None => Ok(vec![]),
            // Legacy [mcp] table with `servers = [...]` — not MCP declarations
            Some(toml::Value::Table(_)) => Ok(vec![]),
            // New [[mcp]] array of tables
            Some(toml::Value::Array(arr)) => {
                let mut decls = Vec::new();
                for item in arr {
                    let decl: McpDecl = item.try_into().map_err(|e| {
                        anyhow::anyhow!("Failed to parse [[mcp]] entry: {e}")
                    })?;
                    decls.push(decl);
                }
                Ok(decls)
            }
            Some(other) => anyhow::bail!(
                "Unexpected value type for `mcp` field: {}",
                other.type_str()
            ),
        }
    }

    fn parse_providers(val: Option<toml::Value>) -> AgentProviders {
        match val {
            None => AgentProviders::default(),
            Some(toml::Value::Table(t)) => {
                let mut targets: Vec<String> = vec![];
                let mut overrides: IndexMap<String, toml::Value> = IndexMap::new();
                for (k, v) in t {
                    if k == "targets" {
                        if let toml::Value::Array(arr) = v {
                            targets = arr
                                .into_iter()
                                .filter_map(|x| {
                                    if let toml::Value::String(s) = x {
                                        Some(s)
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                        }
                    } else {
                        overrides.insert(k, v);
                    }
                }
                AgentProviders { targets, overrides }
            }
            _ => AgentProviders::default(),
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const NEW_FORMAT: &str = r#"
[agent]
name = "my-agent"
description = "A test agent"
rules = ["rules/style.md", "github.com/org/pkg/rules/safety.md"]
skills = ["agents/skills/backend-rust"]

[permissions]
allow = ["Bash", "Read", "Write"]

[[mcp]]
id = "ship"
command = "ship"
args = ["mcp"]
env = { SHIP_TOKEN = "${SHIP_TOKEN}" }

[[mcp]]
id = "extra"
command = "node"
args = ["server.js"]

[providers]
targets = ["claude", "cursor"]

[providers.claude]
model = "claude-opus-4-5"
context_window = 200000
"#;

    #[test]
    fn parse_new_format() {
        let a = AgentDef::from_toml_str(NEW_FORMAT).unwrap();
        assert_eq!(a.name, "my-agent");
        assert_eq!(a.description.as_deref(), Some("A test agent"));
        assert_eq!(a.rules.len(), 2);
        assert_eq!(a.skills.len(), 1);
        assert_eq!(a.permissions.allow, vec!["Bash", "Read", "Write"]);
        assert_eq!(a.mcp.len(), 2);
        assert_eq!(a.mcp[0].id, "ship");
        assert_eq!(a.mcp[0].args, vec!["mcp"]);
        assert_eq!(
            a.mcp[0].env.get("SHIP_TOKEN").map(|s| s.as_str()),
            Some("${SHIP_TOKEN}")
        );
        assert_eq!(a.providers.targets, vec!["claude", "cursor"]);
        assert!(a.providers.overrides.contains_key("claude"));
    }

    #[test]
    fn parse_legacy_profile_format() {
        let toml_str = r#"
[profile]
id = "rust-compiler"
name = "Rust Compiler"
version = "0.1.0"
description = "Ship compiler"
providers = ["claude"]

[skills]
refs = []

[permissions]
preset = "ship-standard"
default_mode = "bypassPermissions"
tools_deny = ["Bash(git push --force*)"]

[rules]
inline = """
Your domain is the Ship compiler.
"""
"#;
        let a = AgentDef::from_toml_str(toml_str).unwrap();
        assert_eq!(a.name, "Rust Compiler");
        assert_eq!(a.description.as_deref(), Some("Ship compiler"));
        assert!(a.rules.iter().any(|r| r.contains("inline:")));
        assert!(a.permissions.allow.is_empty());
        assert_eq!(a.providers.targets, vec!["claude"]);
    }

    #[test]
    fn parse_legacy_profiles_dir() {
        // Verify real profile files parse without error
        let profiles_dir =
            std::path::Path::new("/workspaces/ship/.ship/agents/profiles");
        if profiles_dir.exists() {
            for entry in std::fs::read_dir(profiles_dir).unwrap() {
                let path = entry.unwrap().path();
                if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                    let result = AgentDef::from_file(&path);
                    assert!(
                        result.is_ok(),
                        "Failed to parse {}: {:?}",
                        path.display(),
                        result.err()
                    );
                }
            }
        }
    }

    #[test]
    fn missing_name_is_error() {
        let toml_str = r#"
[agent]
description = "no name"
"#;
        let err = AgentDef::from_toml_str(toml_str).unwrap_err();
        assert!(err.to_string().contains("[agent].name"), "{err}");
    }

    #[test]
    fn file_stem_used_as_fallback_name() {
        let toml_str = r#"
[agent]
description = "no explicit name"
"#;
        let a = AgentDef::from_toml_str_with_stem(toml_str, "my-agent").unwrap();
        assert_eq!(a.name, "my-agent");
    }

    #[test]
    fn no_agent_or_profile_section_is_error() {
        let err = AgentDef::from_toml_str("[other]\nfoo = 1\n").unwrap_err();
        assert!(
            err.to_string().contains("[agent] or [profile]"),
            "{err}"
        );
    }

    #[test]
    fn agent_with_only_rules_no_skills_is_valid() {
        let toml_str = r#"
[agent]
name = "rules-only"
rules = ["rules/style.md"]
"#;
        let a = AgentDef::from_toml_str(toml_str).unwrap();
        assert_eq!(a.rules, vec!["rules/style.md"]);
        assert!(a.skills.is_empty());
    }

    #[test]
    fn agent_with_only_skills_no_rules_is_valid() {
        let toml_str = r#"
[agent]
name = "skills-only"
skills = ["agents/skills/backend"]
"#;
        let a = AgentDef::from_toml_str(toml_str).unwrap();
        assert!(a.rules.is_empty());
        assert_eq!(a.skills, vec!["agents/skills/backend"]);
    }

    #[test]
    fn empty_permissions_is_valid() {
        let toml_str = r#"
[agent]
name = "no-perms"
"#;
        let a = AgentDef::from_toml_str(toml_str).unwrap();
        assert!(a.permissions.allow.is_empty());
    }

    #[test]
    fn multiple_mcp_entries() {
        let toml_str = r#"
[agent]
name = "multi-mcp"

[[mcp]]
id = "a"
command = "cmd-a"

[[mcp]]
id = "b"
command = "cmd-b"
args = ["--flag"]
"#;
        let a = AgentDef::from_toml_str(toml_str).unwrap();
        assert_eq!(a.mcp.len(), 2);
        assert_eq!(a.mcp[0].id, "a");
        assert_eq!(a.mcp[1].id, "b");
        assert_eq!(a.mcp[1].args, vec!["--flag"]);
    }

    #[test]
    fn providers_targets_and_overrides() {
        let toml_str = r#"
[agent]
name = "provider-test"

[providers]
targets = ["claude"]

[providers.claude]
model = "claude-opus-4-5"
"#;
        let a = AgentDef::from_toml_str(toml_str).unwrap();
        assert_eq!(a.providers.targets, vec!["claude"]);
        assert!(a.providers.overrides.contains_key("claude"));
    }
}
