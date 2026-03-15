use super::types::{Release, ReleaseBreakingChange, ReleaseMetadata, ReleaseStatus};
use anyhow::{Context, Result, anyhow};
use chrono::Utc;

impl Release {
    pub fn to_markdown(&self) -> Result<String> {
        let body = self.body.trim_start_matches('\n');
        Ok(format!(
            "<!-- ship:release id={} version={} -->\n\n{}",
            self.metadata.id, self.metadata.version, body
        ))
    }

    pub fn from_markdown(content: &str) -> Result<Self> {
        if content.starts_with("+++\n") {
            Self::from_toml_markdown(content)
        } else {
            let (header_id, header_version, body_content) = parse_generated_release_header(content);
            let version = header_version.unwrap_or_else(|| {
                body_content
                    .lines()
                    .find(|l| l.starts_with("# "))
                    .map(|l| l.trim_start_matches("# ").trim().to_string())
                    .unwrap_or_default()
            });
            let id = header_id.unwrap_or_else(|| version.clone());
            let now = Utc::now().to_rfc3339();
            Ok(Release {
                metadata: ReleaseMetadata {
                    id,
                    version,
                    status: ReleaseStatus::default(),
                    created: now.clone(),
                    updated: now,
                    supported: None,
                    target_date: None,
                    tags: Vec::new(),
                },
                body: body_content,
                breaking_changes: Vec::new(),
            })
        }
    }

    fn from_toml_markdown(content: &str) -> Result<Self> {
        let rest = &content[4..]; // skip "+++\n"
        let end = rest
            .find("\n+++")
            .ok_or_else(|| anyhow!("Invalid release format: missing closing +++"))?;
        let toml_str = &rest[..end];
        let body = rest[end + 4..].trim_start_matches('\n').to_string();
        let metadata: ReleaseMetadata =
            toml::from_str(toml_str).context("Failed to parse release TOML frontmatter")?;

        let mut release = Release {
            metadata,
            body: body.clone(),
            breaking_changes: Vec::new(),
        };

        release.extract_breaking_changes();

        Ok(release)
    }

    pub fn extract_breaking_changes(&mut self) {
        let mut items = Vec::new();
        let mut in_section = false;

        for line in self.body.lines() {
            if line.starts_with("## ") && line.contains("Breaking Changes") {
                in_section = true;
                continue;
            } else if line.starts_with("## ") && in_section {
                break;
            }

            if in_section {
                let trimmed = line.trim();
                if trimmed.starts_with("- ") {
                    let text = trimmed[2..].trim().to_string();
                    if !text.is_empty() {
                        items.push(ReleaseBreakingChange {
                            id: runtime::gen_nanoid(),
                            text,
                        });
                    }
                }
            }
        }
        self.breaking_changes = items;
    }
}

fn parse_generated_release_header(content: &str) -> (Option<String>, Option<String>, String) {
    let mut lines = content.lines();
    if let Some(first) = lines.next() {
        let trimmed = first.trim();
        if trimmed.starts_with("<!-- ship:release ") && trimmed.ends_with("-->") {
            let id = trimmed
                .split_whitespace()
                .find_map(|part| part.strip_prefix("id="))
                .map(|value| value.trim().to_string());
            let version = trimmed
                .split_whitespace()
                .find_map(|part| part.strip_prefix("version="))
                .map(|value| value.trim_end_matches("-->").trim().to_string());
            let mut body = lines.collect::<Vec<_>>().join("\n");
            body = body.trim_start_matches('\n').to_string();
            return (id, version, body);
        }
    }
    (None, None, content.to_string())
}
