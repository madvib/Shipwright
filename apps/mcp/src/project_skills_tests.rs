use crate::requests::ListProjectSkillsRequest;
use crate::tools::skills::list_project_skills;
use compiler::PullSkill;
use runtime::project::init_project;
use tempfile::tempdir;

fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = tempdir().unwrap();
    let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
    (tmp, ship_dir)
}

fn write_skill(ship_dir: &std::path::Path, id: &str, frontmatter: &str, body: &str) {
    let skill_dir = ship_dir.join("skills").join(id);
    std::fs::create_dir_all(&skill_dir).unwrap();
    let content = format!("---\n{frontmatter}\n---\n{body}");
    std::fs::write(skill_dir.join("SKILL.md"), content).unwrap();
}

/// Write a skill to an arbitrary subdirectory relative to `.ship/`.
fn write_skill_at(
    ship_dir: &std::path::Path,
    rel_dir: &str,
    id: &str,
    frontmatter: &str,
    body: &str,
) {
    let skill_dir = ship_dir.join(rel_dir).join(id);
    std::fs::create_dir_all(&skill_dir).unwrap();
    let content = format!("---\n{frontmatter}\n---\n{body}");
    std::fs::write(skill_dir.join("SKILL.md"), content).unwrap();
}

/// Write a manifest with `project.skill_paths` to `.ship/ship.jsonc`.
fn write_manifest_with_skill_paths(ship_dir: &std::path::Path, paths: &[&str]) {
    let paths_json: Vec<String> = paths.iter().map(|p| format!("\"{}\"", p)).collect();
    let content = format!(
        r#"{{"id": "test123", "project": {{"skill_paths": [{}]}}}}"#,
        paths_json.join(", ")
    );
    std::fs::write(ship_dir.join(runtime::config::PRIMARY_CONFIG_FILE), content).unwrap();
}

#[test]
fn returns_all_skills() {
    let (_tmp, ship_dir) = setup();
    write_skill(&ship_dir, "tdd", "name: TDD", "Write tests first.");
    write_skill(&ship_dir, "browse", "name: Browse", "Browse the web.");
    write_skill(
        &ship_dir,
        "code-review",
        "name: Code Review",
        "Review code.",
    );

    let result = list_project_skills(&ship_dir, ListProjectSkillsRequest { query: None });
    let skills: Vec<PullSkill> = serde_json::from_str(&result).unwrap();
    let ids: Vec<&str> = skills.iter().map(|s| s.id.as_str()).collect();
    assert!(ids.contains(&"tdd"));
    assert!(ids.contains(&"browse"));
    assert!(ids.contains(&"code-review"));
    // init_project seeds a task-policy skill, so total is 4
    assert!(
        skills.len() >= 3,
        "expected at least 3 skills, got {}",
        skills.len()
    );
}

#[test]
fn filters_by_query() {
    let (_tmp, ship_dir) = setup();
    write_skill(
        &ship_dir,
        "tdd",
        "name: TDD\ndescription: Test-driven dev",
        "Body",
    );
    write_skill(
        &ship_dir,
        "browse",
        "name: Browse\ndescription: Web browsing",
        "Body",
    );

    let result = list_project_skills(
        &ship_dir,
        ListProjectSkillsRequest {
            query: Some("tdd".into()),
        },
    );
    let skills: Vec<PullSkill> = serde_json::from_str(&result).unwrap();
    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].id, "tdd");
}

#[test]
fn nonexistent_dir_returns_empty_array() {
    let tmp = tempdir().unwrap();
    let fake_ship_dir = tmp.path().join("no-such-ship");
    let result = list_project_skills(&fake_ship_dir, ListProjectSkillsRequest { query: None });
    let skills: Vec<PullSkill> = serde_json::from_str(&result).unwrap();
    assert!(skills.is_empty());
}

#[test]
fn skips_dirs_without_skill_md() {
    let (_tmp, ship_dir) = setup();
    write_skill(&ship_dir, "real-skill", "name: Real", "Has SKILL.md");
    let orphan = ship_dir.join("skills/orphan");
    std::fs::create_dir_all(&orphan).unwrap();
    std::fs::write(orphan.join("random.txt"), "not a skill").unwrap();

    let result = list_project_skills(&ship_dir, ListProjectSkillsRequest { query: None });
    let skills: Vec<PullSkill> = serde_json::from_str(&result).unwrap();
    let ids: Vec<&str> = skills.iter().map(|s| s.id.as_str()).collect();
    assert!(ids.contains(&"real-skill"));
    assert!(!ids.contains(&"orphan"));
}

#[test]
fn includes_vars_and_files() {
    let (_tmp, ship_dir) = setup();
    write_skill(&ship_dir, "my-skill", "name: My Skill", "Content here");
    let skill_dir = ship_dir.join("skills/my-skill");
    std::fs::create_dir_all(skill_dir.join("assets")).unwrap();
    std::fs::write(
        skill_dir.join("assets/vars.json"),
        r#"{"style":{"type":"string","default":"standard"}}"#,
    )
    .unwrap();

    let result = list_project_skills(
        &ship_dir,
        ListProjectSkillsRequest {
            query: Some("my-skill".into()),
        },
    );
    let skills: Vec<PullSkill> = serde_json::from_str(&result).unwrap();
    assert_eq!(skills.len(), 1);
    assert!(skills[0].vars_schema.is_some());
    assert!(skills[0].files.contains(&"SKILL.md".to_string()));
    assert!(skills[0].files.contains(&"assets/vars.json".to_string()));
    assert_eq!(skills[0].source, "project");
}

// ── skill_paths multi-directory tests ──────────────────────────────────────

#[test]
fn discovers_skills_from_multiple_paths() {
    let tmp = tempdir().unwrap();
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    write_manifest_with_skill_paths(&ship_dir, &["skills/", "docs/"]);

    write_skill_at(&ship_dir, "skills", "alpha", "name: Alpha", "Alpha skill");
    write_skill_at(&ship_dir, "docs", "beta", "name: Beta", "Beta skill");

    let result = list_project_skills(&ship_dir, ListProjectSkillsRequest { query: None });
    let skills: Vec<PullSkill> = serde_json::from_str(&result).unwrap();
    let ids: Vec<&str> = skills.iter().map(|s| s.id.as_str()).collect();
    assert!(ids.contains(&"alpha"), "should find skill in skills/");
    assert!(ids.contains(&"beta"), "should find skill in docs/");
    assert_eq!(skills.len(), 2);
}

#[test]
fn default_no_skill_paths_scans_skills_only() {
    let (_tmp, ship_dir) = setup();
    // init_project creates a manifest without skill_paths — only skills/ is scanned.
    // Write a skill in a non-default directory to prove it is NOT found.
    write_skill(&ship_dir, "found-skill", "name: Found", "This is found.");
    write_skill_at(&ship_dir, "extra", "hidden", "name: Hidden", "Not found.");

    let result = list_project_skills(&ship_dir, ListProjectSkillsRequest { query: None });
    let skills: Vec<PullSkill> = serde_json::from_str(&result).unwrap();
    let ids: Vec<&str> = skills.iter().map(|s| s.id.as_str()).collect();
    assert!(ids.contains(&"found-skill"));
    assert!(
        !ids.contains(&"hidden"),
        "skills in non-configured dirs should be invisible"
    );
}

#[test]
fn duplicate_skill_id_first_path_wins() {
    let tmp = tempdir().unwrap();
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    write_manifest_with_skill_paths(&ship_dir, &["first/", "second/"]);

    write_skill_at(
        &ship_dir,
        "first",
        "dup",
        "name: First Version",
        "First body",
    );
    write_skill_at(
        &ship_dir,
        "second",
        "dup",
        "name: Second Version",
        "Second body",
    );

    let result = list_project_skills(&ship_dir, ListProjectSkillsRequest { query: None });
    let skills: Vec<PullSkill> = serde_json::from_str(&result).unwrap();
    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].name, "First Version", "first path should win");
    assert!(skills[0].content.contains("First body"));
}
