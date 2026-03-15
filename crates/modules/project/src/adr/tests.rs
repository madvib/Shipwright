use super::crud::{create_adr, delete_adr, get_adr_by_id, list_adrs, move_adr, update_adr};
use super::migration::import_adrs_from_files;
use super::types::AdrStatus;
use crate::project::init_project;
use tempfile::tempdir;

#[test]
fn test_create_adr() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let project_dir = init_project(tmp.path().to_path_buf())?;
    let entry = create_adr(
        &project_dir,
        "Use PostgreSQL",
        "We need relational storage.",
        "Chosen for robustness",
        "accepted",
    )?;
    assert_eq!(entry.adr.metadata.title, "Use PostgreSQL");
    assert_eq!(entry.adr.context, "We need relational storage.");
    assert_eq!(entry.adr.decision, "Chosen for robustness");
    let expected_path = runtime::project::adrs_dir(&project_dir)
        .join("accepted")
        .join("use-postgresql.md");
    assert_eq!(entry.path, expected_path.to_string_lossy().to_string());
    assert!(
        !std::path::Path::new(&entry.path).exists(),
        "ADR path should be projected, not written by CRUD"
    );
    Ok(())
}

#[test]
fn test_create_adr_empty_title_rejected() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let project_dir = init_project(tmp.path().to_path_buf())?;
    let result = create_adr(&project_dir, "", "context", "decision", "accepted");
    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_create_adr_has_uuid() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let project_dir = init_project(tmp.path().to_path_buf())?;
    let entry = create_adr(
        &project_dir,
        "Use Redis",
        "",
        "Fast in-memory store",
        "accepted",
    )?;
    assert!(!entry.id.is_empty());
    assert_eq!(entry.id.len(), 8);
    Ok(())
}

#[test]
fn test_get_adr_by_id() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let project_dir = init_project(tmp.path().to_path_buf())?;
    let entry = create_adr(
        &project_dir,
        "Use SQLite",
        "Need embedded DB.",
        "Go with SQLite.",
        "proposed",
    )?;
    let fetched = get_adr_by_id(&project_dir, &entry.id)?;
    assert_eq!(fetched.adr.metadata.title, "Use SQLite");
    assert_eq!(fetched.adr.context, "Need embedded DB.");
    Ok(())
}

#[test]
fn test_update_adr() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let project_dir = init_project(tmp.path().to_path_buf())?;
    let entry = create_adr(
        &project_dir,
        "Update ADR",
        "original context",
        "original decision",
        "proposed",
    )?;
    let mut updated_adr = entry.adr.clone();
    updated_adr.decision = "updated decision".to_string();
    let refreshed = update_adr(&project_dir, &entry.id, updated_adr)?;
    assert_eq!(refreshed.adr.decision, "updated decision");
    Ok(())
}

#[test]
fn test_move_adr_to_accepted_updates_projected_path() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let project_dir = init_project(tmp.path().to_path_buf())?;
    let entry = create_adr(
        &project_dir,
        "Move ADR",
        "context",
        "the decision",
        "proposed",
    )?;
    let moved = move_adr(&project_dir, &entry.id, AdrStatus::Accepted)?;
    assert_eq!(moved.status, AdrStatus::Accepted);
    let accepted_file = runtime::project::adrs_dir(&project_dir)
        .join("accepted")
        .join(&moved.file_name);
    assert_eq!(moved.path, accepted_file.to_string_lossy().to_string());
    assert!(!accepted_file.exists());
    Ok(())
}

#[test]
fn test_list_adrs() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let project_dir = init_project(tmp.path().to_path_buf())?;
    create_adr(&project_dir, "ADR One", "", "decision one", "accepted")?;
    create_adr(&project_dir, "ADR Two", "", "decision two", "proposed")?;
    let adrs = list_adrs(&project_dir)?;
    assert_eq!(adrs.len(), 2);
    let titles: Vec<&str> = adrs.iter().map(|a| a.adr.metadata.title.as_str()).collect();
    assert!(titles.contains(&"ADR One"));
    assert!(titles.contains(&"ADR Two"));
    Ok(())
}

#[test]
fn test_delete_adr() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let project_dir = init_project(tmp.path().to_path_buf())?;
    let entry = create_adr(&project_dir, "Delete ADR", "", "decision", "accepted")?;
    delete_adr(&project_dir, &entry.id)?;
    assert!(get_adr_by_id(&project_dir, &entry.id).is_err());
    Ok(())
}

#[test]
fn test_import_adrs_from_files_idempotent() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let project_dir = init_project(tmp.path().to_path_buf())?;
    // Create 2 ADRs via the new API (they go into SQLite).
    create_adr(&project_dir, "IDR One", "ctx", "dec", "accepted")?;
    create_adr(&project_dir, "IDR Two", "ctx", "dec", "proposed")?;
    // Running import twice should not duplicate.
    let first = import_adrs_from_files(&project_dir)?;
    let second = import_adrs_from_files(&project_dir)?;
    assert_eq!(
        second, 0,
        "second import should import nothing (idempotent)"
    );
    let _ = first;
    Ok(())
}
