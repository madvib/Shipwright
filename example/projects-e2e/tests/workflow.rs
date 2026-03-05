mod helpers;
use helpers::TestProject;
use runtime::project::*;
use ship_module_project::create_adr;

/// After init, all namespace directories exist in the right places.
#[test]
fn init_creates_namespace_structure() {
    let p = TestProject::new().unwrap();

    // workflow/
    p.assert_ship_file("workflow/issues/backlog");
    p.assert_ship_file("workflow/issues/in-progress");
    p.assert_ship_file("workflow/issues/blocked");
    p.assert_ship_file("workflow/issues/done");
    p.assert_ship_file("workflow/specs");
    p.assert_ship_file("project/features");

    // project/
    p.assert_ship_file("project/releases");
    p.assert_ship_file("project/adrs");
    p.assert_ship_file("project/notes");
    p.assert_ship_file("project/vision.md");
    p.assert_ship_file("project/TEMPLATE.md");
    p.assert_ship_file("project/releases/TEMPLATE.md");
    p.assert_ship_file("project/adrs/TEMPLATE.md");
    p.assert_ship_file("project/notes/TEMPLATE.md");

    // agents/
    p.assert_ship_file("agents/rules");

    // shared
    p.assert_ship_file("generated");
    p.assert_ship_file("ship.toml");
    p.assert_ship_file("README.md");
    p.assert_ship_file("workflow/README.md");
    p.assert_ship_file("workflow/issues/TEMPLATE.md");
    p.assert_ship_file("workflow/specs/TEMPLATE.md");
    p.assert_ship_file("project/features/TEMPLATE.md");
}

/// Vision document is seeded in project/ not workflow/specs/.
#[test]
fn vision_doc_lives_in_project_namespace() {
    let p = TestProject::new().unwrap();
    p.assert_ship_file("project/vision.md");
    p.assert_no_ship_file("specs/vision.md"); // old flat path must not exist
}

/// Core loop: release → feature → spec → issue, all resolve to correct paths.
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

    let spec = crate::helpers::create_spec(p.ship_dir.clone(), "Auth Spec", "", "draft").unwrap();
    assert!(spec.starts_with(specs_dir(&p.ship_dir)));

    let issue =
        crate::helpers::create_issue(p.ship_dir.clone(), "Implement login", "", "backlog").unwrap();
    assert!(issue.starts_with(issues_dir(&p.ship_dir).join("backlog")));
}

/// Issues move between statuses (folder rename), path reflects new status.
#[test]
fn issue_move_updates_path() {
    let p = TestProject::new().unwrap();

    let path = crate::helpers::create_issue(p.ship_dir.clone(), "My Issue", "", "backlog").unwrap();
    assert!(path.to_string_lossy().contains("backlog"));

    let new_path =
        crate::helpers::move_issue(p.ship_dir.clone(), path, "backlog", "in-progress").unwrap();
    assert!(new_path.to_string_lossy().contains("in-progress"));
    let reference = new_path.file_name().unwrap().to_string_lossy().to_string();
    let moved = ship_module_project::get_issue_by_id(&p.ship_dir, &reference).unwrap();
    assert_eq!(moved.status, ship_module_project::IssueStatus::InProgress);
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
    p.assert_ship_file_contains(
        adr_path
            .strip_prefix(&p.ship_dir)
            .unwrap()
            .to_str()
            .unwrap(),
        "Use TOML",
    );
}

/// .ship/.gitignore uses namespace paths, not flat names.
#[test]
fn gitignore_uses_namespace_paths() {
    let p = TestProject::new().unwrap();
    let gitignore = std::fs::read_to_string(p.ship_dir.join(".gitignore")).unwrap();

    // Issues local by default
    assert!(
        gitignore.contains("workflow/issues"),
        "issues should be gitignored"
    );
    assert!(gitignore.contains("generated/"));
    assert!(!gitignore.contains("events.ndjson"));
    // ship.db lives at ~/.ship/state/<slug>/ship.db — outside the project, not gitignored here

    // These are committed by default — must NOT appear in gitignore
    assert!(!gitignore.contains("project/adrs"));
    assert!(!gitignore.contains("project/notes"));
    assert!(!gitignore.contains("agents"));
    assert!(!gitignore.contains("project/features"));
    assert!(!gitignore.contains("project/releases"));
    assert!(!gitignore.contains("workflow/specs"));
}

/// Events track creates in both workflow/ and project/ namespaces.
#[test]
fn events_track_both_namespaces() {
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
