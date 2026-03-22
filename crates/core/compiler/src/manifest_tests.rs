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
    assert!(
        err.to_string().contains("empty version constraint"),
        "{err}"
    );
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
fn name_rejects_uppercase() {
    let toml_str = r#"
[module]
name = "GitHub.com/owner/repo"
version = "1.0.0"
"#;
    let err = ShipManifest::from_toml_str(toml_str).unwrap_err();
    assert!(err.to_string().contains("invalid characters"), "{err}");
}

#[test]
fn name_rejects_spaces() {
    let toml_str = r#"
[module]
name = "my package"
version = "1.0.0"
"#;
    let err = ShipManifest::from_toml_str(toml_str).unwrap_err();
    assert!(err.to_string().contains("invalid characters"), "{err}");
}

#[test]
fn name_accepts_scoped() {
    let toml_str = r#"
[module]
name = "@myorg/my-agent"
version = "1.0.0"
"#;
    let m = ShipManifest::from_toml_str(toml_str).unwrap();
    assert_eq!(m.module.name, "@myorg/my-agent");
}

#[test]
fn name_accepts_dotted_host() {
    let toml_str = r#"
[module]
name = "gitlab.com/owner/repo"
version = "1.0.0"
"#;
    let m = ShipManifest::from_toml_str(toml_str).unwrap();
    assert_eq!(m.module.name, "gitlab.com/owner/repo");
}

#[test]
fn license_validates_spdx() {
    let toml_str = r#"
[module]
name = "github.com/owner/repo"
version = "1.0.0"
license = "MIT"
"#;
    let m = ShipManifest::from_toml_str(toml_str).unwrap();
    assert_eq!(m.module.license.as_deref(), Some("MIT"));
}

#[test]
fn license_accepts_compound_expression() {
    let toml_str = r#"
[module]
name = "github.com/owner/repo"
version = "1.0.0"
license = "MIT OR Apache-2.0"
"#;
    let m = ShipManifest::from_toml_str(toml_str).unwrap();
    assert_eq!(m.module.license.as_deref(), Some("MIT OR Apache-2.0"));
}

#[test]
fn license_rejects_unknown() {
    let toml_str = r#"
[module]
name = "github.com/owner/repo"
version = "1.0.0"
license = "MADE-UP-LICENSE"
"#;
    let err = ShipManifest::from_toml_str(toml_str).unwrap_err();
    assert!(err.to_string().contains("not a valid SPDX"), "{err}");
}

#[test]
fn authors_field_parses() {
    let toml_str = r#"
[module]
name = "github.com/owner/repo"
version = "1.0.0"
authors = ["Alice <alice@example.com>", "Bob"]
"#;
    let m = ShipManifest::from_toml_str(toml_str).unwrap();
    assert_eq!(m.module.authors.len(), 2);
    assert_eq!(m.module.authors[0], "Alice <alice@example.com>");
}

#[test]
fn authors_defaults_to_empty() {
    let m = ShipManifest::from_toml_str(minimal_toml()).unwrap();
    assert!(m.module.authors.is_empty());
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

#[test]
fn parse_jsonc_manifest() {
    let jsonc = r#"{
  // Ship manifest in JSONC format
  "module": {
    "name": "github.com/owner/repo",
    "version": "1.0.0",
    "description": "A library", // inline comment
  },
  "dependencies": {
    "github.com/a/b": "^1.0.0",
  },
}"#;
    let m = ShipManifest::from_jsonc_str(jsonc).unwrap();
    assert_eq!(m.module.name, "github.com/owner/repo");
    assert_eq!(m.module.version, "1.0.0");
    assert_eq!(m.module.description.as_deref(), Some("A library"));
    assert_eq!(m.dependencies.len(), 1);
}

#[test]
fn from_file_dispatches_by_extension() {
    let dir = tempfile::tempdir().unwrap();
    let jsonc_path = dir.path().join("ship.jsonc");
    std::fs::write(
        &jsonc_path,
        r#"{
  "module": { "name": "github.com/test/repo", "version": "0.1.0" }
}"#,
    )
    .unwrap();
    let m = ShipManifest::from_file(&jsonc_path).unwrap();
    assert_eq!(m.module.name, "github.com/test/repo");

    let toml_path = dir.path().join("ship.toml");
    std::fs::write(&toml_path, minimal_toml()).unwrap();
    let m2 = ShipManifest::from_file(&toml_path).unwrap();
    assert_eq!(m2.module.name, "github.com/owner/repo");
}
