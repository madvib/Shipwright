use super::types::{Issue, IssueMetadata};
use anyhow::{Context, Result, anyhow};
use chrono::Utc;

impl Issue {
    pub fn to_markdown(&self) -> Result<String> {
        let toml_str = toml::to_string(&self.metadata)
            .context("Failed to serialise issue metadata as TOML")?;
        Ok(format!("+++\n{}+++\n\n{}", toml_str, self.description))
    }

    /// Parse both new TOML (`+++`) and legacy YAML (`---`) frontmatter.
    pub fn from_markdown(content: &str) -> Result<Self> {
        if content.starts_with("+++\n") {
            Self::from_toml_markdown(content)
        } else if content.starts_with("---\n") {
            Self::from_yaml_markdown_legacy(content)
        } else {
            Err(anyhow!("Invalid issue format: missing frontmatter start"))
        }
    }

    fn from_toml_markdown(content: &str) -> Result<Self> {
        let rest = &content[4..]; // skip leading "+++\n"
        let end = rest
            .find("\n+++")
            .ok_or_else(|| anyhow!("Invalid issue format: missing closing +++"))?;
        let toml_str = &rest[..end];
        let description = rest[end + 4..].trim_start_matches('\n').to_string();
        let metadata: IssueMetadata =
            toml::from_str(toml_str).context("Failed to parse issue TOML frontmatter")?;
        Ok(Issue {
            metadata,
            description,
        })
    }

    /// Minimal YAML reader for the old `---` format — avoids keeping serde_yaml.
    fn from_yaml_markdown_legacy(content: &str) -> Result<Self> {
        let parts: Vec<&str> = content.splitn(3, "---\n").collect();
        if parts.len() < 3 {
            return Err(anyhow!(
                "Invalid legacy issue format: incomplete frontmatter"
            ));
        }
        let yaml = parts[1];
        let description = parts[2].trim_start_matches('\n').to_string();

        let mut title = String::new();
        let mut created = Utc::now().to_rfc3339();
        let mut updated = Utc::now().to_rfc3339();

        for line in yaml.lines() {
            if let Some(v) = line.strip_prefix("title: ") {
                title = v.trim().to_string();
            } else if let Some(v) = line.strip_prefix("created_at: ") {
                created = v.trim().to_string();
            } else if let Some(v) = line.strip_prefix("updated_at: ") {
                updated = v.trim().to_string();
            }
        }

        Ok(Issue {
            metadata: IssueMetadata {
                title,
                created,
                updated,
                ..Default::default()
            },
            description,
        })
    }
}
