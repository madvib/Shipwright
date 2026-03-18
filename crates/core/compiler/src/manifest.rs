//! ship.toml manifest parser.
//!
//! Parses and validates the three-section ship.toml format:
//! - [module]  — package identity (name, version, description, license)
//! - [dependencies] — direct dependency map (path → version or full table)
//! - [exports]  — skills and agent paths this module exports
//!
//! The compiler is pure (no I/O) except for `from_file`.
//! Validation is eager and returns actionable errors.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

// ── Module section ─────────────────────────────────────────────────────────────

/// Identity metadata for a ship module.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ManifestModule {
    /// Namespaced package path, e.g. `github.com/owner/repo`.
    pub name: String,
    /// Semver string, no `v` prefix required in manifest.
    pub version: String,
    /// Human-readable description.
    #[serde(default)]
    pub description: Option<String>,
    /// SPDX license identifier, e.g. `MIT`.
    #[serde(default)]
    pub license: Option<String>,
}

/// Raw (pre-validation) module section — fields are optional so we can emit
/// actionable errors rather than serde's "missing field" message.
#[derive(Debug, Deserialize)]
struct RawManifestModule {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    license: Option<String>,
}

// ── Dependencies section ───────────────────────────────────────────────────────

/// Shorthand `"^1.0.0"` or full `{ version = "^1.0.0", grant = ["Bash"] }`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ManifestDepValue {
    /// `"github.com/owner/repo" = "^1.0.0"` (shorthand, no grants)
    Version(String),
    /// `"github.com/owner/repo" = { version = "^1.0.0", grant = ["Bash"] }`
    Full(ManifestDependency),
}

/// Fully-expanded dependency record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ManifestDependency {
    pub version: String,
    #[serde(default)]
    pub grant: Vec<String>,
}

impl ManifestDepValue {
    /// Normalise to the full form, stripping the shorthand wrapper.
    pub fn into_dep(self) -> ManifestDependency {
        match self {
            ManifestDepValue::Version(v) => ManifestDependency {
                version: v,
                grant: vec![],
            },
            ManifestDepValue::Full(d) => d,
        }
    }
}

// ── Exports section ────────────────────────────────────────────────────────────

/// Paths exported by this module. Validated at publish time, not parse time.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ManifestExports {
    /// Skill directory paths relative to `.ship/` (must contain SKILL.md).
    #[serde(default)]
    pub skills: Vec<String>,
    /// Agent TOML paths relative to `.ship/` (must end in .toml).
    #[serde(default)]
    pub agents: Vec<String>,
}

// ── Raw manifest for deserialisation ──────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct RawShipManifest {
    module: Option<RawManifestModule>,
    #[serde(default)]
    dependencies: IndexMap<String, ManifestDepValue>,
    #[serde(default)]
    exports: ManifestExports,
}

// ── Root manifest ──────────────────────────────────────────────────────────────

/// Parsed and validated `ship.toml` manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShipManifest {
    pub module: ManifestModule,
    #[serde(default)]
    pub dependencies: IndexMap<String, ManifestDepValue>,
    #[serde(default)]
    pub exports: ManifestExports,
}

impl ShipManifest {
    /// Parse from a TOML string and validate required fields and semver.
    pub fn from_toml_str(s: &str) -> anyhow::Result<Self> {
        let raw: RawShipManifest = toml::from_str(s)
            .map_err(|e| anyhow::anyhow!("Failed to parse ship.toml: {e}"))?;

        // Validate [module] section presence.
        let raw_module = raw
            .module
            .ok_or_else(|| anyhow::anyhow!("[module] section is required in ship.toml"))?;

        let name = raw_module
            .name
            .filter(|n| !n.is_empty())
            .ok_or_else(|| anyhow::anyhow!("[module].name is required in ship.toml"))?;

        let version = raw_module
            .version
            .filter(|v| !v.is_empty())
            .ok_or_else(|| anyhow::anyhow!("[module].version is required in ship.toml"))?;

        // Validate semver (strip optional v-prefix).
        let ver_str = version.trim_start_matches('v');
        semver::Version::parse(ver_str).map_err(|e| {
            anyhow::anyhow!("[module].version '{}' is not valid semver: {e}", version)
        })?;

        // Validate dependency version strings.
        for (path, dep_val) in &raw.dependencies {
            let v = match dep_val {
                ManifestDepValue::Version(v) => v.as_str(),
                ManifestDepValue::Full(d) => d.version.as_str(),
            };
            if v.is_empty() {
                anyhow::bail!(
                    "Dependency '{}' has an empty version constraint in ship.toml",
                    path
                );
            }
        }

        Ok(ShipManifest {
            module: ManifestModule {
                name,
                version,
                description: raw_module.description,
                license: raw_module.license,
            },
            dependencies: raw.dependencies,
            exports: raw.exports,
        })
    }

    /// Read and parse from a file path.
    pub fn from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Cannot read {}: {e}", path.display()))?;
        Self::from_toml_str(&content)
    }

    /// Iterate resolved (full-form) dependencies.
    pub fn resolved_deps(&self) -> impl Iterator<Item = (&str, ManifestDependency)> {
        self.dependencies
            .iter()
            .map(|(k, v)| (k.as_str(), v.clone().into_dep()))
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_toml() -> &'static str {
        r#"
[module]
name = "github.com/owner/repo"
version = "1.0.0"
"#
    }

    #[test]
    fn parse_minimal_manifest() {
        let m = ShipManifest::from_toml_str(minimal_toml()).unwrap();
        assert_eq!(m.module.name, "github.com/owner/repo");
        assert_eq!(m.module.version, "1.0.0");
        assert!(m.module.description.is_none());
        assert!(m.module.license.is_none());
        assert!(m.dependencies.is_empty());
        assert!(m.exports.skills.is_empty());
        assert!(m.exports.agents.is_empty());
    }

    #[test]
    fn parse_full_manifest() {
        let toml_str = r#"
[module]
name = "github.com/owner/mylib"
version = "2.3.1"
description = "A great library"
license = "MIT"

[dependencies]
"github.com/a/b" = "^1.0.0"
"github.com/c/d" = { version = "~2.1.0", grant = ["Bash", "Read"] }

[exports]
skills = ["agents/skills/my-skill"]
agents = ["agents/profiles/my-agent.toml"]
"#;
        let m = ShipManifest::from_toml_str(toml_str).unwrap();
        assert_eq!(m.module.description.as_deref(), Some("A great library"));
        assert_eq!(m.module.license.as_deref(), Some("MIT"));
        assert_eq!(m.dependencies.len(), 2);

        let (_, dep_b) = m.resolved_deps().next().unwrap();
        assert_eq!(dep_b.version, "^1.0.0");
        assert!(dep_b.grant.is_empty());

        let dep_d = m
            .dependencies
            .get("github.com/c/d")
            .unwrap()
            .clone()
            .into_dep();
        assert_eq!(dep_d.version, "~2.1.0");
        assert_eq!(dep_d.grant, vec!["Bash", "Read"]);

        assert_eq!(m.exports.skills, vec!["agents/skills/my-skill"]);
        assert_eq!(m.exports.agents, vec!["agents/profiles/my-agent.toml"]);
    }

    #[test]
    fn missing_name_is_error() {
        let toml_str = r#"
[module]
version = "1.0.0"
"#;
        let err = ShipManifest::from_toml_str(toml_str).unwrap_err();
        assert!(err.to_string().contains("[module].name"), "{err}");
    }

    #[test]
    fn missing_version_is_error() {
        let toml_str = r#"
[module]
name = "github.com/owner/repo"
"#;
        let err = ShipManifest::from_toml_str(toml_str).unwrap_err();
        assert!(err.to_string().contains("[module].version"), "{err}");
    }

    #[test]
    fn missing_module_section_is_error() {
        let toml_str = r#"
[dependencies]
"github.com/a/b" = "^1.0.0"
"#;
        let err = ShipManifest::from_toml_str(toml_str).unwrap_err();
        assert!(err.to_string().contains("[module]"), "{err}");
    }

    #[test]
    fn invalid_semver_is_error() {
        let toml_str = r#"
[module]
name = "github.com/owner/repo"
version = "not-semver"
"#;
        let err = ShipManifest::from_toml_str(toml_str).unwrap_err();
        assert!(err.to_string().contains("not valid semver"), "{err}");
    }

    #[test]
    fn version_with_v_prefix_is_valid() {
        let toml_str = r#"
[module]
name = "github.com/owner/repo"
version = "v1.2.3"
"#;
        // v-prefix is stripped before semver parse
        let m = ShipManifest::from_toml_str(toml_str).unwrap();
        assert_eq!(m.module.version, "v1.2.3");
    }

    #[test]
    fn empty_dependencies_is_valid() {
        let toml_str = r#"
[module]
name = "github.com/owner/repo"
version = "0.1.0"

[dependencies]
"#;
        let m = ShipManifest::from_toml_str(toml_str).unwrap();
        assert!(m.dependencies.is_empty());
    }

    #[test]
    fn empty_version_constraint_is_error() {
        let toml_str = r#"
[module]
name = "github.com/owner/repo"
version = "0.1.0"

[dependencies]
"github.com/a/b" = ""
"#;
        let err = ShipManifest::from_toml_str(toml_str).unwrap_err();
        assert!(err.to_string().contains("empty version constraint"), "{err}");
    }

    #[test]
    fn resolved_deps_normalises_shorthand() {
        let toml_str = r#"
[module]
name = "github.com/owner/repo"
version = "0.1.0"

[dependencies]
"github.com/a/b" = "main"
"#;
        let m = ShipManifest::from_toml_str(toml_str).unwrap();
        let deps: Vec<_> = m.resolved_deps().collect();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].0, "github.com/a/b");
        assert_eq!(deps[0].1.version, "main");
        assert!(deps[0].1.grant.is_empty());
    }

    #[test]
    fn omitting_exports_is_valid() {
        let m = ShipManifest::from_toml_str(minimal_toml()).unwrap();
        assert!(m.exports.skills.is_empty());
        assert!(m.exports.agents.is_empty());
    }

    #[test]
    fn sha_constraint_is_valid() {
        let sha = "abc1234567890abcdef1234567890abcdef1234";
        let toml_str = format!(
            r#"
[module]
name = "github.com/owner/repo"
version = "0.1.0"

[dependencies]
"github.com/a/b" = "{sha}"
"#
        );
        let m = ShipManifest::from_toml_str(&toml_str).unwrap();
        assert_eq!(
            m.dependencies
                .get("github.com/a/b")
                .unwrap()
                .clone()
                .into_dep()
                .version,
            sha
        );
    }
}
