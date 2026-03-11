use super::types::{Feature, FeatureCriterion, FeatureMetadata, FeatureTodo};
use anyhow::{Context, Result, anyhow};
use chrono::Utc;

impl Feature {
    pub fn to_markdown(&self) -> Result<String> {
        let body = self.body.trim_start_matches('\n');
        // Include title as H1 so the file can be re-imported (e.g. after git clone)
        let body_section = if !self.metadata.title.is_empty() && !body.starts_with("# ") {
            if body.is_empty() {
                format!("# {}", self.metadata.title)
            } else {
                format!("# {}\n\n{}", self.metadata.title, body)
            }
        } else {
            body.to_string()
        };
        Ok(format!(
            "<!-- ship:feature id={} -->\n\n{}",
            self.metadata.id, body_section
        ))
    }

    pub fn from_markdown(content: &str) -> Result<Self> {
        if content.starts_with("+++\n") {
            Self::from_toml_markdown(content)
        } else {
            let (header_id, body_content) = parse_generated_feature_header(content);
            let title = body_content
                .lines()
                .find(|l| l.starts_with("# "))
                .map(|l| l.trim_start_matches("# ").trim().to_string())
                .unwrap_or_default();
            let now = Utc::now().to_rfc3339();
            Ok(Feature {
                metadata: FeatureMetadata {
                    id: header_id.unwrap_or_default(),
                    title,
                    description: None,
                    created: now.clone(),
                    updated: now,
                    release_id: None,
                    active_target_id: None,
                    branch: None,
                    agent: None,
                    tags: Vec::new(),
                },
                body: body_content,
                todos: Vec::new(),
                criteria: Vec::new(),
            })
        }
    }

    fn from_toml_markdown(content: &str) -> Result<Self> {
        let rest = &content[4..]; // skip "+++\n"
        let end = rest
            .find("\n+++")
            .ok_or_else(|| anyhow!("Invalid feature format: missing closing +++"))?;
        let toml_str = &rest[..end];
        let body = rest[end + 4..].trim_start_matches('\n').to_string();
        let metadata: FeatureMetadata =
            toml::from_str(toml_str).context("Failed to parse feature TOML frontmatter")?;

        let mut feature = Feature {
            metadata,
            body: body.clone(),
            todos: Vec::new(),
            criteria: Vec::new(),
        };

        // Extract todos and criteria from body
        feature.extract_structured_data();

        Ok(feature)
    }

    pub fn extract_structured_data(&mut self) {
        self.todos = parse_checklist(&self.body, "Delivery Todos");
        self.criteria = parse_checklist(&self.body, "Acceptance Criteria");
    }
}

fn parse_generated_feature_header(content: &str) -> (Option<String>, String) {
    let mut lines = content.lines();
    if let Some(first) = lines.next() {
        let trimmed = first.trim();
        if trimmed.starts_with("<!-- ship:feature ") && trimmed.ends_with("-->") {
            let id = trimmed
                .split_whitespace()
                .find_map(|part| part.strip_prefix("id="))
                .map(|value| value.trim_end_matches("-->").trim().to_string());
            let mut body = lines.collect::<Vec<_>>().join("\n");
            body = body.trim_start_matches('\n').to_string();
            return (id, body);
        }
    }
    (None, content.to_string())
}

fn parse_checklist<T: ChecklistItem>(body: &str, section_name: &str) -> Vec<T> {
    let mut items = Vec::new();
    let mut in_section = false;

    for line in body.lines() {
        if line.starts_with("## ") && line.contains(section_name) {
            in_section = true;
            continue;
        } else if line.starts_with("## ") && in_section {
            break;
        }

        if in_section {
            let trimmed = line.trim();
            if trimmed.starts_with("- [ ]") || trimmed.starts_with("- [x]") {
                let completed = trimmed.starts_with("- [x]");
                let text = trimmed[5..].trim().to_string();
                if !text.is_empty() {
                    items.push(T::new(text, completed));
                }
            }
        }
    }
    items
}

trait ChecklistItem {
    fn new(text: String, completed: bool) -> Self;
}

impl ChecklistItem for FeatureTodo {
    fn new(text: String, completed: bool) -> Self {
        FeatureTodo {
            id: runtime::gen_nanoid(),
            text,
            completed,
        }
    }
}

impl ChecklistItem for FeatureCriterion {
    fn new(text: String, completed: bool) -> Self {
        FeatureCriterion {
            id: runtime::gen_nanoid(),
            text,
            met: completed,
        }
    }
}
