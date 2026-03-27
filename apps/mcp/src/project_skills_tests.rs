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

#[test]
fn returns_all_skills() {
    let (_tmp, ship_dir) = setup();
    write_skill(&ship_dir, "tdd", "name: TDD", "Write tests first.");
    write_skill(&ship_dir, "browse", "name: Browse", "Browse the web.");
    write_skill(&ship_dir, "code-review", "name: Code Review", "Review code.");

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
