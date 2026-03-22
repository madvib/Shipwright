use super::*;
use crate::project::init_project;
use tempfile::tempdir;

fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = tempdir().unwrap();
    let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
    (tmp, ship_dir)
}

// ─── Target tests ─────────────────────────────────────────────────────────────

#[test]
fn create_and_get_target() {
    let (_tmp, ship_dir) = setup();
    let t = create_target(
        &ship_dir,
        "milestone",
        "v0.1.0",
        Some("Funnel"),
        Some("ship in every project"),
        None,
    )
    .unwrap();
    assert_eq!(t.kind, "milestone");
    assert_eq!(t.status, "active");
    let fetched = get_target(&ship_dir, &t.id).unwrap().unwrap();
    assert_eq!(fetched.title, "v0.1.0");
}

#[test]
fn list_targets_by_kind() {
    let (_tmp, ship_dir) = setup();
    create_target(&ship_dir, "milestone", "v0.1.0", None, None, None).unwrap();
    create_target(&ship_dir, "surface", "compiler", None, None, None).unwrap();
    let milestones = list_targets(&ship_dir, Some("milestone")).unwrap();
    assert_eq!(milestones.len(), 1);
    assert_eq!(milestones[0].title, "v0.1.0");
}

#[test]
fn update_target_body_markdown_and_phase() {
    let (_tmp, ship_dir) = setup();
    let t = create_target(&ship_dir, "milestone", "v0.1.0", None, None, None).unwrap();
    assert!(t.body_markdown.is_none());
    assert!(t.phase.is_none());

    update_target(
        &ship_dir,
        &t.id,
        TargetPatch {
            body_markdown: Some("# v0.1.0\n\nShip in every project.".to_string()),
            phase: Some("alpha".to_string()),
            due_date: Some("2026-06-01".to_string()),
            ..Default::default()
        },
    )
    .unwrap();

    let fetched = get_target(&ship_dir, &t.id).unwrap().unwrap();
    assert_eq!(
        fetched.body_markdown.as_deref(),
        Some("# v0.1.0\n\nShip in every project.")
    );
    assert_eq!(fetched.phase.as_deref(), Some("alpha"));
    assert_eq!(fetched.due_date.as_deref(), Some("2026-06-01"));
}

#[test]
fn update_target_file_scope() {
    let (_tmp, ship_dir) = setup();
    let t = create_target(&ship_dir, "surface", "compiler", None, None, None).unwrap();
    assert!(t.file_scope.is_empty());

    update_target(
        &ship_dir,
        &t.id,
        TargetPatch {
            file_scope: Some(vec![
                "crates/core/compiler/".to_string(),
                "packages/compiler/".to_string(),
            ]),
            ..Default::default()
        },
    )
    .unwrap();

    let fetched = get_target(&ship_dir, &t.id).unwrap().unwrap();
    assert_eq!(
        fetched.file_scope,
        vec!["crates/core/compiler/", "packages/compiler/"]
    );
}

#[test]
fn update_target_preserves_unset_fields() {
    let (_tmp, ship_dir) = setup();
    let t = create_target(
        &ship_dir,
        "surface",
        "studio",
        Some("Web compiler"),
        Some("ship in browser"),
        None,
    )
    .unwrap();

    update_target(
        &ship_dir,
        &t.id,
        TargetPatch {
            phase: Some("beta".to_string()),
            ..Default::default()
        },
    )
    .unwrap();

    let fetched = get_target(&ship_dir, &t.id).unwrap().unwrap();
    assert_eq!(fetched.description.as_deref(), Some("Web compiler"));
    assert_eq!(fetched.goal.as_deref(), Some("ship in browser"));
    assert_eq!(fetched.phase.as_deref(), Some("beta"));
}

// ─── Capability tests ─────────────────────────────────────────────────────────

#[test]
fn capability_lifecycle() {
    let (_tmp, ship_dir) = setup();
    let t = create_target(&ship_dir, "surface", "compiler", None, None, None).unwrap();
    let c = create_capability(&ship_dir, &t.id, "Profile compilation", None).unwrap();
    assert_eq!(c.status, "aspirational");
    assert_eq!(c.priority, 0);
    assert!(c.acceptance_criteria.is_empty());

    mark_capability_actual(&ship_dir, &c.id, "test: profile_scaffold_parses").unwrap();
    let caps = list_capabilities(&ship_dir, Some(&t.id), Some("actual"), None).unwrap();
    assert_eq!(caps.len(), 1);
    assert_eq!(
        caps[0].evidence.as_deref(),
        Some("test: profile_scaffold_parses")
    );
}

#[test]
fn update_capability_new_fields_round_trip() {
    let (_tmp, ship_dir) = setup();
    let t = create_target(&ship_dir, "milestone", "v0.1.0", None, None, None).unwrap();
    let c = create_capability(&ship_dir, &t.id, "CLI workspace create", None).unwrap();

    update_capability(
        &ship_dir,
        &c.id,
        CapabilityPatch {
            phase: Some("bootstrap".to_string()),
            acceptance_criteria: Some(vec![
                "ship workspace create works".to_string(),
                "worktree created".to_string(),
            ]),
            preset_hint: Some("rust-runtime".to_string()),
            file_scope: Some(vec!["apps/ship-studio-cli/".to_string()]),
            assigned_to: Some("rust-lane".to_string()),
            priority: Some(1),
            ..Default::default()
        },
    )
    .unwrap();

    let fetched = get_capability(&ship_dir, &c.id).unwrap().unwrap();
    assert_eq!(fetched.phase.as_deref(), Some("bootstrap"));
    assert_eq!(
        fetched.acceptance_criteria,
        vec!["ship workspace create works", "worktree created"]
    );
    assert_eq!(fetched.preset_hint.as_deref(), Some("rust-runtime"));
    assert_eq!(fetched.file_scope, vec!["apps/ship-studio-cli/"]);
    assert_eq!(fetched.assigned_to.as_deref(), Some("rust-lane"));
    assert_eq!(fetched.priority, 1);
}

#[test]
fn update_capability_status_in_progress() {
    let (_tmp, ship_dir) = setup();
    let t = create_target(&ship_dir, "surface", "cli", None, None, None).unwrap();
    let c = create_capability(&ship_dir, &t.id, "ship init", None).unwrap();
    assert_eq!(c.status, "aspirational");

    update_capability(
        &ship_dir,
        &c.id,
        CapabilityPatch {
            status: Some("in_progress".to_string()),
            assigned_to: Some("rust-lane".to_string()),
            ..Default::default()
        },
    )
    .unwrap();

    let fetched = get_capability(&ship_dir, &c.id).unwrap().unwrap();
    assert_eq!(fetched.status, "in_progress");
    assert_eq!(fetched.assigned_to.as_deref(), Some("rust-lane"));
}

#[test]
fn list_capabilities_phase_filter() {
    let (_tmp, ship_dir) = setup();
    let t = create_target(&ship_dir, "milestone", "v0.1.0", None, None, None).unwrap();
    let c1 = create_capability(&ship_dir, &t.id, "init", None).unwrap();
    let c2 = create_capability(&ship_dir, &t.id, "auth", None).unwrap();

    update_capability(
        &ship_dir,
        &c1.id,
        CapabilityPatch {
            phase: Some("bootstrap".to_string()),
            ..Default::default()
        },
    )
    .unwrap();
    update_capability(
        &ship_dir,
        &c2.id,
        CapabilityPatch {
            phase: Some("polish".to_string()),
            ..Default::default()
        },
    )
    .unwrap();

    let bootstrap = list_capabilities(&ship_dir, Some(&t.id), None, Some("bootstrap")).unwrap();
    assert_eq!(bootstrap.len(), 1);
    assert_eq!(bootstrap[0].id, c1.id);

    let all = list_capabilities(&ship_dir, Some(&t.id), None, None).unwrap();
    assert_eq!(all.len(), 2);
}

#[test]
fn list_capabilities_all_filters() {
    let (_tmp, ship_dir) = setup();
    let t = create_target(&ship_dir, "milestone", "v0.1.0", None, None, None).unwrap();
    create_capability(&ship_dir, &t.id, "accounts", None).unwrap();
    let c2 = create_capability(&ship_dir, &t.id, "cli auth", None).unwrap();
    mark_capability_actual(&ship_dir, &c2.id, "ship login works").unwrap();

    let all = list_capabilities(&ship_dir, Some(&t.id), None, None).unwrap();
    assert_eq!(all.len(), 2);
    let actual = list_capabilities(&ship_dir, None, Some("actual"), None).unwrap();
    assert_eq!(actual.len(), 1);
}

#[test]
fn delete_capability_removes_row() {
    let (_tmp, ship_dir) = setup();
    let t = create_target(&ship_dir, "surface", "cli", None, None, None).unwrap();
    let c = create_capability(&ship_dir, &t.id, "ephemeral cap", None).unwrap();

    assert!(get_capability(&ship_dir, &c.id).unwrap().is_some());
    let deleted = delete_capability(&ship_dir, &c.id).unwrap();
    assert!(deleted);
    assert!(get_capability(&ship_dir, &c.id).unwrap().is_none());

    // Second delete returns false (not found)
    let deleted_again = delete_capability(&ship_dir, &c.id).unwrap();
    assert!(!deleted_again);
}

#[test]
fn delete_capability_nonexistent_returns_false() {
    let (_tmp, ship_dir) = setup();
    let deleted = delete_capability(&ship_dir, "nonexistent_id").unwrap();
    assert!(!deleted);
}

#[test]
fn capability_priority_ordering() {
    let (_tmp, ship_dir) = setup();
    let t = create_target(&ship_dir, "surface", "cli", None, None, None).unwrap();
    let c1 = create_capability(&ship_dir, &t.id, "low priority", None).unwrap();
    let c2 = create_capability(&ship_dir, &t.id, "high priority", None).unwrap();

    update_capability(
        &ship_dir,
        &c1.id,
        CapabilityPatch {
            priority: Some(10),
            ..Default::default()
        },
    )
    .unwrap();
    update_capability(
        &ship_dir,
        &c2.id,
        CapabilityPatch {
            priority: Some(1),
            ..Default::default()
        },
    )
    .unwrap();

    let caps = list_capabilities(&ship_dir, Some(&t.id), None, None).unwrap();
    assert_eq!(caps[0].id, c2.id); // priority 1 first
    assert_eq!(caps[1].id, c1.id); // priority 10 second
}
