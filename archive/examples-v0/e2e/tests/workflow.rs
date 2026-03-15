mod helpers;
use helpers::TestProject;
use runtime::project::*;
use ship_module_project::create_adr;

/// After init, core agent/runtime assets and top-level vision exist.
#[test]
fn init_creates_namespace_structure() {
    let p = TestProject::new().unwrap();

    // top-level vision
    p.assert_ship_file("vision.md");
    p.assert_no_ship_file("project");
    p.assert_no_ship_file("project/vision.md");
    p.assert_no_ship_file("TEMPLATE.md");
    p.assert_no_ship_file("README.md");

    // agents/
    p.assert_ship_file("agents/rules");
    p.assert_ship_file("agents/mcp.toml");
    p.assert_ship_file("agents/permissions.toml");
    p.assert_ship_file("agents/skills/task-policy/SKILL.md");
    p.assert_ship_file_contains("agents/mcp.toml", "[mcp.servers.ship]");
    p.assert_ship_file_contains("agents/mcp.toml", "command = \"ship\"");
    p.assert_ship_file_contains("agents/mcp.toml", "\"serve\"");

    // shared
    p.assert_ship_file("generated");
    p.assert_ship_file("ship.toml");
}

/// Vision document is seeded at .ship/vision.md.
#[test]
fn vision_doc_lives_at_top_level() {
    let p = TestProject::new().unwrap();
    p.assert_ship_file("vision.md");
    p.assert_no_ship_file("specs/vision.md"); // old flat path must not exist
    p.assert_no_ship_file("project/vision.md");
}

/// Core loop: release → feature, all resolve to correct paths.
#[test]
fn core_loop_paths_resolve_correctly() {
    let p = TestProject::new().unwrap();

    let release = crate::helpers::create_release(p.ship_dir.clone(), "v0.1.0-alpha", "").unwrap();
    assert!(release.starts_with(releases_dir(&p.ship_dir)));

    let feature = crate::helpers::create_feature(
        p.ship_dir.clone(),
        "Auth Redesign",
        "",
        Some(release.file_name().unwrap().to_str().unwrap()),
        None,
        None,
    )
    .unwrap();
    assert!(feature.1.starts_with(features_dir(&p.ship_dir)));
}

/// ADRs land in project/adrs/.
#[test]
fn adrs_land_in_project_namespace() {
    let p = TestProject::new().unwrap();

    let adr_entry = create_adr(
        &p.ship_dir,
        "Use TOML",
        "Context",
        "Simpler for AI agents",
        "accepted",
    )
    .unwrap();
    let adr_path = std::path::PathBuf::from(adr_entry.path);
    assert!(adr_path.starts_with(adrs_dir(&p.ship_dir)));
    assert_eq!(adr_entry.adr.metadata.title, "Use TOML");
    assert!(
        !adr_path.exists(),
        "ADR markdown path should be projected, not written"
    );
}

/// .ship/.gitignore uses namespace paths, not flat names.
#[test]
fn gitignore_uses_namespace_paths() {
    let p = TestProject::new().unwrap();
    let gitignore = std::fs::read_to_string(p.ship_dir.join(".gitignore")).unwrap();

    assert!(gitignore.contains("generated/"));
    assert!(!gitignore.contains("events.ndjson"));
    // ship.db lives at ~/.ship/state/<slug>/ship.db — outside the project, not gitignored here

    // Committed by default — must NOT appear in gitignore
    assert!(!gitignore.contains("agents/rules"));
    assert!(!gitignore.contains("agents/mcp.toml"));
    assert!(!gitignore.contains("agents/permissions.toml"));
    assert!(!gitignore.contains("ship.toml"));

    // Optional (local by default) — must appear in gitignore
    assert!(gitignore.contains("project/adrs"));
    assert!(gitignore.contains("project/notes"));
    assert!(gitignore.contains("project/features"));
    assert!(gitignore.contains("project/releases"));
    assert!(gitignore.contains("vision.md"));
    assert!(gitignore.contains("agents/skills"));
}

/// Events track creates in the project namespace.
#[test]
fn events_track_project_namespace() {
    use crate::helpers::{create_feature, create_release};
    use runtime::{latest_event_seq, list_events_since};

    let p = TestProject::new().unwrap();
    let seq0 = latest_event_seq(&p.ship_dir).unwrap();

    create_release(p.ship_dir.clone(), "v0.1.0", "").unwrap();
    create_feature(p.ship_dir.clone(), "Feature A", "", None, None, None).unwrap();

    let events = list_events_since(&p.ship_dir, seq0, None).unwrap();
    assert!(events.len() >= 2);
    let entities: Vec<_> = events.iter().map(|e| format!("{:?}", e.entity)).collect();
    assert!(entities.iter().any(|e| e.contains("Release")));
    assert!(entities.iter().any(|e| e.contains("Feature")));
}
