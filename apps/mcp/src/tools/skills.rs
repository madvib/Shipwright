use std::path::Path;

use runtime::list_effective_skills;

use crate::requests::ListSkillsRequest;

pub fn list_skills(project_dir: &Path, req: ListSkillsRequest) -> String {
    let skills = match list_effective_skills(project_dir) {
        Ok(s) => s,
        Err(e) => return format!("Error listing skills: {}", e),
    };
    let filtered: Vec<_> = if let Some(ref query) = req.query {
        let q = query.to_ascii_lowercase();
        skills
            .into_iter()
            .filter(|s| {
                s.id.to_ascii_lowercase().contains(&q)
                    || s.name.to_ascii_lowercase().contains(&q)
                    || s.description
                        .as_deref()
                        .unwrap_or("")
                        .to_ascii_lowercase()
                        .contains(&q)
            })
            .collect()
    } else {
        skills
    };
    if filtered.is_empty() {
        return "No skills found.".to_string();
    }
    let mut out = String::from("Skills:\n");
    for s in &filtered {
        let desc = s.description.as_deref().unwrap_or("(no description)");
        out.push_str(&format!("- {} — {} — {}\n", s.id, s.name, desc));
    }
    out
}
