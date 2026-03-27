use crate::requests::{DeleteSkillFileRequest, WriteSkillFileRequest};
use crate::tools::skills::{delete_skill_file, write_skill_file};
use runtime::project::init_project;
use tempfile::tempdir;

fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = tempdir().unwrap();
    let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
    (tmp, ship_dir)
}

// ── write_skill_file ─────────────────────────────────────────────

#[test]
fn write_skill_file_creates_file() {
    let (_tmp, ship_dir) = setup();
    let result = write_skill_file(
        &ship_dir,
        WriteSkillFileRequest {
            skill_id: "my-skill".into(),
            file_path: "SKILL.md".into(),
            content: "# My Skill\nHello".into(),
        },
    );
    assert!(result.starts_with("Wrote "), "unexpected: {result}");
    let written = ship_dir.join("skills/my-skill/SKILL.md");
    assert!(written.exists());
    assert_eq!(std::fs::read_to_string(&written).unwrap(), "# My Skill\nHello");
}

#[test]
fn write_skill_file_creates_nested_dirs() {
    let (_tmp, ship_dir) = setup();
    let result = write_skill_file(
        &ship_dir,
        WriteSkillFileRequest {
            skill_id: "browse".into(),
            file_path: "references/docs/index.md".into(),
            content: "nested content".into(),
        },
    );
    assert!(result.starts_with("Wrote "), "unexpected: {result}");
    let written = ship_dir.join("skills/browse/references/docs/index.md");
    assert!(written.exists());
    assert_eq!(std::fs::read_to_string(&written).unwrap(), "nested content");
}

#[test]
fn write_skill_file_overwrites_existing() {
    let (_tmp, ship_dir) = setup();
    let skill_dir = ship_dir.join("skills/tdd");
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(skill_dir.join("SKILL.md"), "old content").unwrap();

    let result = write_skill_file(
        &ship_dir,
        WriteSkillFileRequest {
            skill_id: "tdd".into(),
            file_path: "SKILL.md".into(),
            content: "new content".into(),
        },
    );
    assert!(result.starts_with("Wrote "), "unexpected: {result}");
    assert_eq!(
        std::fs::read_to_string(skill_dir.join("SKILL.md")).unwrap(),
        "new content"
    );
}

#[test]
fn write_skill_file_rejects_invalid_skill_id() {
    let (_tmp, ship_dir) = setup();
    let result = write_skill_file(
        &ship_dir,
        WriteSkillFileRequest {
            skill_id: "Invalid-Name".into(),
            file_path: "SKILL.md".into(),
            content: "x".into(),
        },
    );
    assert!(result.contains("Error"), "expected error: {result}");
    assert!(result.contains("Invalid skill_id"), "unexpected message: {result}");
}

#[test]
fn write_skill_file_rejects_path_traversal() {
    let (_tmp, ship_dir) = setup();
    let result = write_skill_file(
        &ship_dir,
        WriteSkillFileRequest {
            skill_id: "my-skill".into(),
            file_path: "../escape.md".into(),
            content: "x".into(),
        },
    );
    assert!(result.contains("Error"), "expected error: {result}");
    assert!(result.contains(".."), "should mention traversal: {result}");
}

#[test]
fn write_skill_file_rejects_absolute_path() {
    let (_tmp, ship_dir) = setup();
    let result = write_skill_file(
        &ship_dir,
        WriteSkillFileRequest {
            skill_id: "my-skill".into(),
            file_path: "/etc/passwd".into(),
            content: "x".into(),
        },
    );
    assert!(result.contains("Error"), "expected error: {result}");
    assert!(result.contains("relative"), "should mention relative: {result}");
}

#[test]
fn write_skill_file_rejects_empty_path() {
    let (_tmp, ship_dir) = setup();
    let result = write_skill_file(
        &ship_dir,
        WriteSkillFileRequest {
            skill_id: "my-skill".into(),
            file_path: "".into(),
            content: "x".into(),
        },
    );
    assert!(result.contains("Error"), "expected error: {result}");
    assert!(result.contains("empty"), "should mention empty: {result}");
}

// ── delete_skill_file ────────────────────────────────────────────

#[test]
fn delete_skill_file_removes_file() {
    let (_tmp, ship_dir) = setup();
    let skill_dir = ship_dir.join("skills/my-skill");
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(skill_dir.join("extra.md"), "will be deleted").unwrap();

    let result = delete_skill_file(
        &ship_dir,
        DeleteSkillFileRequest {
            skill_id: "my-skill".into(),
            file_path: "extra.md".into(),
        },
    );
    assert!(result.starts_with("Deleted "), "unexpected: {result}");
    assert!(!skill_dir.join("extra.md").exists());
}

#[test]
fn delete_skill_file_refuses_skill_md() {
    let (_tmp, ship_dir) = setup();
    let result = delete_skill_file(
        &ship_dir,
        DeleteSkillFileRequest {
            skill_id: "my-skill".into(),
            file_path: "SKILL.md".into(),
        },
    );
    assert!(result.contains("Error"), "expected error: {result}");
    assert!(result.contains("SKILL.md"), "should mention SKILL.md: {result}");
}

#[test]
fn delete_skill_file_returns_error_for_missing_file() {
    let (_tmp, ship_dir) = setup();
    let skill_dir = ship_dir.join("skills/my-skill");
    std::fs::create_dir_all(&skill_dir).unwrap();

    let result = delete_skill_file(
        &ship_dir,
        DeleteSkillFileRequest {
            skill_id: "my-skill".into(),
            file_path: "nonexistent.md".into(),
        },
    );
    assert!(result.contains("Error"), "expected error: {result}");
    assert!(result.contains("does not exist"), "unexpected message: {result}");
}

#[test]
fn delete_skill_file_rejects_path_traversal() {
    let (_tmp, ship_dir) = setup();
    let result = delete_skill_file(
        &ship_dir,
        DeleteSkillFileRequest {
            skill_id: "my-skill".into(),
            file_path: "../other-skill/SKILL.md".into(),
        },
    );
    assert!(result.contains("Error"), "expected error: {result}");
    assert!(result.contains(".."), "should mention traversal: {result}");
}

#[test]
fn delete_skill_file_rejects_invalid_skill_id() {
    let (_tmp, ship_dir) = setup();
    let result = delete_skill_file(
        &ship_dir,
        DeleteSkillFileRequest {
            skill_id: "BAD NAME".into(),
            file_path: "file.md".into(),
        },
    );
    assert!(result.contains("Error"), "expected error: {result}");
    assert!(result.contains("Invalid skill_id"), "unexpected message: {result}");
}
