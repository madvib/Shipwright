use super::*;
use std::collections::BTreeMap;
use tempfile::tempdir;

#[test]
fn dry_run_reads_manifest_and_hashes() -> Result<()> {
    let dir = tempdir()?;
    let ship_dir = dir.path().join(".ship");
    std::fs::create_dir_all(&ship_dir)?;
    std::fs::write(
        ship_dir.join("ship.jsonc"),
        r#"{
  "module": {
    "name": "github.com/test/pkg",
    "version": "0.1.0",
    "description": "Test package",
    "license": "MIT",
    "authors": ["Test Author"]
  },
  "exports": {
    "skills": ["agents/skills/my-skill"]
  }
}"#,
    )?;
    let skill_dir = ship_dir.join("agents").join("skills").join("my-skill");
    std::fs::create_dir_all(&skill_dir)?;
    std::fs::write(skill_dir.join("SKILL.md"), "# My Skill")?;

    run_publish(dir.path(), None, true, None)?;
    Ok(())
}

#[test]
fn publish_without_token_errors() -> Result<()> {
    let dir = tempdir()?;
    let ship_dir = dir.path().join(".ship");
    std::fs::create_dir_all(&ship_dir)?;
    std::fs::write(
        ship_dir.join("ship.jsonc"),
        r#"{
  "module": {
    "name": "github.com/test/pkg",
    "version": "0.1.0"
  }
}"#,
    )?;

    let err = run_publish(dir.path(), None, false, None).unwrap_err();
    assert!(
        err.to_string().contains("ship login"),
        "expected auth error, got: {err}"
    );
    Ok(())
}

fn sample_hashes() -> ExportHashes {
    let mut per_export = BTreeMap::new();
    per_export.insert("agents/skills/foo".into(), "sha256:aaa".into());
    per_export.insert("agents/profiles/bar.toml".into(), "sha256:bbb".into());
    ExportHashes {
        combined: "sha256:abc123".into(),
        per_export,
    }
}

fn multi_export_manifest() -> ShipManifest {
    ShipManifest::from_toml_str(
        r#"
[module]
name = "github.com/test/pkg"
version = "1.0.0"
description = "A test"
license = "MIT"
authors = ["Alice"]

[exports]
skills = ["agents/skills/foo"]
agents = ["agents/profiles/bar.toml"]
"#,
    )
    .unwrap()
}

#[test]
fn build_payload_includes_all_fields() {
    let manifest = multi_export_manifest();
    let hashes = sample_hashes();
    let payload = build_payload(&manifest, &hashes, Some("beta"), None);
    assert_eq!(payload["name"], "github.com/test/pkg");
    assert_eq!(payload["version"], "1.0.0");
    assert_eq!(payload["hash"], "sha256:abc123");
    assert_eq!(payload["description"], "A test");
    assert_eq!(payload["license"], "MIT");
    assert_eq!(payload["tag"], "beta");
    assert_eq!(payload["authors"][0], "Alice");
    assert_eq!(payload["exports_skills"][0], "agents/skills/foo");
    assert_eq!(payload["export_hashes"]["agents/skills/foo"], "sha256:aaa");
    assert_eq!(
        payload["export_hashes"]["agents/profiles/bar.toml"],
        "sha256:bbb"
    );
}

#[test]
fn dry_run_reads_jsonc_manifest() -> Result<()> {
    let dir = tempdir()?;
    let ship_dir = dir.path().join(".ship");
    std::fs::create_dir_all(&ship_dir)?;
    std::fs::write(
        ship_dir.join("ship.jsonc"),
        r#"{
  "module": {
    "name": "github.com/test/jsonc-pkg",
    "version": "0.2.0"
  },
  "exports": {
    "skills": ["agents/skills/my-skill"]
  }
}"#,
    )?;
    let skill_dir = ship_dir.join("agents").join("skills").join("my-skill");
    std::fs::create_dir_all(&skill_dir)?;
    std::fs::write(skill_dir.join("SKILL.md"), "# My Skill")?;

    run_publish(dir.path(), None, true, None)?;
    Ok(())
}

#[test]
fn build_payload_omits_empty_export_hashes() {
    let manifest = ShipManifest::from_toml_str(
        r#"
[module]
name = "github.com/test/pkg"
version = "1.0.0"
"#,
    )
    .unwrap();
    let hashes = ExportHashes {
        combined: "sha256:empty".into(),
        per_export: BTreeMap::new(),
    };
    let payload = build_payload(&manifest, &hashes, None, None);
    assert!(payload.get("export_hashes").is_none());
}

// ── Single-export filtering tests ────────────────────────────────────────────

#[test]
fn filter_exports_finds_skill() {
    let manifest = multi_export_manifest();
    let (skills, agents) = filter_exports(&manifest, "agents/skills/foo").unwrap();
    assert_eq!(skills, vec!["agents/skills/foo"]);
    assert!(agents.is_empty());
}

#[test]
fn filter_exports_finds_agent() {
    let manifest = multi_export_manifest();
    let (skills, agents) = filter_exports(&manifest, "agents/profiles/bar.toml").unwrap();
    assert!(skills.is_empty());
    assert_eq!(agents, vec!["agents/profiles/bar.toml"]);
}

#[test]
fn filter_exports_rejects_unknown_path() {
    let manifest = multi_export_manifest();
    let err = filter_exports(&manifest, "agents/skills/nonexistent").unwrap_err();
    assert!(
        err.to_string().contains("not found in manifest"),
        "expected not-found error, got: {err}"
    );
}

#[test]
fn build_payload_filters_to_single_skill() {
    let manifest = multi_export_manifest();
    let hashes = sample_hashes();
    let payload = build_payload(&manifest, &hashes, None, Some("agents/skills/foo"));
    // Only the skill export should be present.
    assert_eq!(payload["exports_skills"][0], "agents/skills/foo");
    assert!(
        payload.get("exports_agents").is_none(),
        "agents should be excluded when filtering to a skill"
    );
}

#[test]
fn build_payload_filters_to_single_agent() {
    let manifest = multi_export_manifest();
    let hashes = sample_hashes();
    let payload = build_payload(&manifest, &hashes, None, Some("agents/profiles/bar.toml"));
    assert_eq!(payload["exports_agents"][0], "agents/profiles/bar.toml");
    assert!(
        payload.get("exports_skills").is_none(),
        "skills should be excluded when filtering to an agent"
    );
}

#[test]
fn dry_run_single_skill_export() -> Result<()> {
    let dir = tempdir()?;
    let ship_dir = dir.path().join(".ship");
    std::fs::create_dir_all(&ship_dir)?;
    std::fs::write(
        ship_dir.join("ship.jsonc"),
        r#"{
  "module": {
    "name": "github.com/test/pkg",
    "version": "0.1.0"
  },
  "exports": {
    "skills": ["agents/skills/alpha", "agents/skills/beta"]
  }
}"#,
    )?;
    for name in &["alpha", "beta"] {
        let skill_dir = ship_dir.join("agents").join("skills").join(name);
        std::fs::create_dir_all(&skill_dir)?;
        std::fs::write(skill_dir.join("SKILL.md"), format!("# {}", name))?;
    }

    // Publish only alpha — should succeed and only hash that one.
    run_publish(dir.path(), Some("agents/skills/alpha"), true, None)?;
    Ok(())
}

#[test]
fn dry_run_unknown_export_errors() {
    let dir = tempdir().unwrap();
    let ship_dir = dir.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    std::fs::write(
        ship_dir.join("ship.jsonc"),
        r#"{
  "module": {
    "name": "github.com/test/pkg",
    "version": "0.1.0"
  },
  "exports": {
    "skills": ["agents/skills/alpha"]
  }
}"#,
    )
    .unwrap();

    let err = run_publish(dir.path(), Some("agents/skills/nope"), true, None).unwrap_err();
    assert!(
        err.to_string().contains("not found in manifest"),
        "expected not-found error, got: {err}"
    );
}
