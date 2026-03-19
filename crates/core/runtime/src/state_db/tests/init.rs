use anyhow::Result;
use tempfile::tempdir;

use crate::state_db::init::{
    ensure_database, ensure_global_dir_outside_project,
};
use crate::state_db::migrations::PROJECT_MIGRATIONS;
use crate::state_db::project_db_path;

#[test]
fn ensure_project_database_is_idempotent() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir)?;
    std::fs::write(
        ship_dir.join(crate::config::PRIMARY_CONFIG_FILE),
        "id = 'TEST1234'\n",
    )?;
    let db_path = project_db_path(&ship_dir)?;
    let report_a = ensure_database(&db_path, PROJECT_MIGRATIONS)?;
    let report_b = ensure_database(&db_path, PROJECT_MIGRATIONS)?;

    assert!(report_a.created);
    assert!(report_a.applied_migrations >= 1);
    assert_eq!(report_a.db_path, report_b.db_path);
    assert!(!report_b.created);
    assert_eq!(report_b.applied_migrations, 0);
    assert!(!report_a.db_path.starts_with(tmp.path()));
    assert!(report_a.db_path.to_string_lossy().contains("ship.db"));

    std::fs::remove_file(&report_a.db_path).ok();
    Ok(())
}

#[test]
fn project_db_key_auto_populates_missing_id_in_ship_toml() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir)?;
    std::fs::write(
        ship_dir.join(crate::config::PRIMARY_CONFIG_FILE),
        "version = '1'\nname = 'legacy-project'\n",
    )?;

    let key = crate::project::project_slug_from_ship_dir(&ship_dir);
    assert!(!key.trim().is_empty());

    let raw = std::fs::read_to_string(ship_dir.join(crate::config::PRIMARY_CONFIG_FILE))?;
    let parsed: toml::Value = toml::from_str(&raw)?;
    let persisted_id = parsed
        .get("id")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string();

    assert!(key.contains("legacy-project-"));
    assert!(key.ends_with(&persisted_id.to_ascii_lowercase()));
    Ok(())
}

#[test]
fn rejects_global_state_dir_inside_project_tree() -> Result<()> {
    let tmp = tempdir()?;
    let project_root = tmp.path().join("repo");
    let ship_dir = project_root.join(".ship");
    std::fs::create_dir_all(&ship_dir)?;
    let local_global = ship_dir.join("state");
    std::fs::create_dir_all(&local_global)?;

    let err = ensure_global_dir_outside_project(&ship_dir, &local_global).unwrap_err();
    assert!(err.to_string().contains("inside project"));
    Ok(())
}

#[test]
fn allows_global_state_dir_outside_project_tree() -> Result<()> {
    let tmp = tempdir()?;
    let project_root = tmp.path().join("repo");
    let ship_dir = project_root.join(".ship");
    std::fs::create_dir_all(&ship_dir)?;
    let external_global = tmp.path().join("global-ship-dir");
    std::fs::create_dir_all(&external_global)?;

    ensure_global_dir_outside_project(&ship_dir, &external_global)?;
    Ok(())
}
