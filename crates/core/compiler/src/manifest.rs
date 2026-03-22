//! Ship manifest parser (JSONC primary, TOML legacy).
//!
//! Parses and validates the three-section manifest format:
//! - [module]  — package identity (name, version, description, license)
//! - [dependencies] — direct dependency map (path → version or full table)
//! - [exports]  — skills and agent paths this module exports
//!
//! The compiler is pure (no I/O) except for `from_file`.
//! Validation is eager and returns actionable errors.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Check that a package name matches `^[a-z0-9._/@-]+$`.
fn is_valid_package_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.' | b'_' | b'/' | b'@' | b'-'))
}

// ── Module section ─────────────────────────────────────────────────────────────

/// Identity metadata for a ship module.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ManifestModule {
    /// Namespaced package path, e.g. `github.com/owner/repo` or `@scope/name`.
    pub name: String,
    /// Semver string, no `v` prefix required in manifest.
    pub version: String,
    /// Human-readable description.
    #[serde(default)]
    pub description: Option<String>,
    /// SPDX license identifier, e.g. `MIT`.
    #[serde(default)]
    pub license: Option<String>,
    /// Package authors, e.g. `["Alice <alice@example.com>"]`.
    #[serde(default)]
    pub authors: Vec<String>,
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
    #[serde(default)]
    authors: Vec<String>,
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

/// Parsed and validated Ship manifest.
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
        Self::from_raw(raw)
    }

    /// Validate a raw manifest and produce a [`ShipManifest`].
    fn from_raw(raw: RawShipManifest) -> anyhow::Result<Self> {
        let raw_module = raw
            .module
            .ok_or_else(|| anyhow::anyhow!("[module] section is required in ship manifest"))?;

        let name = raw_module
            .name
            .filter(|n| !n.is_empty())
            .ok_or_else(|| anyhow::anyhow!("[module].name is required in ship manifest"))?;

        // Validate name: lowercase alphanumeric, dots, hyphens, underscores, slashes, @.
        if !is_valid_package_name(&name) {
            anyhow::bail!(
                "[module].name '{}' contains invalid characters — \
                 must match [a-z0-9._/@-]+",
                name
            );
        }

        let version = raw_module
            .version
            .filter(|v| !v.is_empty())
            .ok_or_else(|| anyhow::anyhow!("[module].version is required in ship manifest"))?;

        // Validate semver (strip optional v-prefix).
        let ver_str = version.trim_start_matches('v');
        semver::Version::parse(ver_str).map_err(|e| {
            anyhow::anyhow!("[module].version '{}' is not valid semver: {e}", version)
        })?;

        // Validate SPDX license if provided.
        if let Some(ref lic) = raw_module.license {
            if !lic.is_empty() {
                validate_spdx(lic).map_err(|e| {
                    anyhow::anyhow!("[module].license '{}' is not a valid SPDX expression: {e}", lic)
                })?;
            }
        }

        // Validate dependency version strings.
        for (path, dep_val) in &raw.dependencies {
            let v = match dep_val {
                ManifestDepValue::Version(v) => v.as_str(),
                ManifestDepValue::Full(d) => d.version.as_str(),
            };
            if v.is_empty() {
                anyhow::bail!(
                    "Dependency '{}' has an empty version constraint in ship manifest",
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
                authors: raw_module.authors,
            },
            dependencies: raw.dependencies,
            exports: raw.exports,
        })
    }

    /// Parse from a JSONC string and validate required fields and semver.
    pub fn from_jsonc_str(s: &str) -> anyhow::Result<Self> {
        let raw: RawShipManifest = crate::jsonc::from_jsonc_str(s)
            .map_err(|e| anyhow::anyhow!("Failed to parse ship.jsonc: {e}"))?;
        Self::from_raw(raw)
    }

    /// Read and parse from a file path. Dispatches to JSONC or TOML based on extension.
    pub fn from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Cannot read {}: {e}", path.display()))?;
        if crate::jsonc::is_jsonc_path(path) {
            Self::from_jsonc_str(&content)
        } else {
            Self::from_toml_str(&content)
        }
    }

    /// Iterate resolved (full-form) dependencies.
    pub fn resolved_deps(&self) -> impl Iterator<Item = (&str, ManifestDependency)> {
        self.dependencies
            .iter()
            .map(|(k, v)| (k.as_str(), v.clone().into_dep()))
    }
}

/// Validate a name as suitable for publishing.
pub fn validate_package_name(name: &str) -> anyhow::Result<()> {
    if !is_valid_package_name(name) {
        anyhow::bail!(
            "package name '{}' contains invalid characters — must match [a-z0-9._/@-]+",
            name
        );
    }
    Ok(())
}

/// Lightweight SPDX validation — accepts common license ids and simple
/// expressions (`MIT`, `Apache-2.0`, `MIT OR Apache-2.0`).
/// Full SPDX expression parsing is deferred to a future release.
fn validate_spdx(expr: &str) -> Result<(), String> {
    // Accept known single identifiers or simple OR/AND expressions.
    let tokens: Vec<&str> = expr.split_whitespace().collect();
    if tokens.is_empty() {
        return Err("empty license expression".into());
    }
    for token in &tokens {
        if *token == "OR" || *token == "AND" || *token == "WITH" {
            continue;
        }
        if !is_known_spdx_id(token) {
            return Err(format!("unknown license identifier '{}'", token));
        }
    }
    Ok(())
}

fn is_known_spdx_id(id: &str) -> bool {
    matches!(
        id,
        "0BSD" | "AAL" | "AFL-3.0" | "AGPL-3.0-only" | "AGPL-3.0-or-later"
            | "Apache-2.0" | "Artistic-2.0" | "BlueOak-1.0.0"
            | "BSD-2-Clause" | "BSD-3-Clause" | "BSL-1.0" | "CAL-1.0"
            | "CAL-1.0-Combined-Work-Exception" | "CC-BY-4.0" | "CC-BY-SA-4.0"
            | "CC0-1.0" | "CPAL-1.0" | "ECL-2.0" | "EFL-2.0" | "EUPL-1.2"
            | "GPL-2.0-only" | "GPL-2.0-or-later" | "GPL-3.0-only" | "GPL-3.0-or-later"
            | "ISC" | "LGPL-2.1-only" | "LGPL-2.1-or-later" | "LGPL-3.0-only"
            | "LGPL-3.0-or-later" | "LiLiQ-P-1.1" | "LiLiQ-R-1.1" | "LiLiQ-Rplus-1.1"
            | "MIT" | "MIT-0" | "MPL-2.0" | "MulanPSL-2.0" | "NCSA" | "OFL-1.1"
            | "OSL-3.0" | "PostgreSQL" | "RPL-1.5" | "SimPL-2.0" | "UPL-1.0"
            | "Unicode-DFS-2016" | "Unlicense" | "Vim" | "WTFPL" | "Zlib" | "ZPL-2.0"
    )
}

#[cfg(test)]
#[path = "manifest_tests.rs"]
mod tests;
