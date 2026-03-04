use super::types::{Spec, SpecMetadata};
use anyhow::{Context, Result, anyhow};

impl Spec {
    pub fn to_markdown(&self) -> Result<String> {
        let toml_str =
            toml::to_string(&self.metadata).context("Failed to serialise spec metadata as TOML")?;
        Ok(format!("+++\n{}+++\n\n{}", toml_str, self.body))
    }

    pub fn from_markdown(content: &str) -> Result<Self> {
        if content.starts_with("+++\n") {
            Self::from_toml_markdown(content)
        } else {
            // Support legacy markdown without frontmatter
            let title = content
                .lines()
                .find(|l| l.starts_with("# "))
                .map(|l| l.trim_start_matches("# ").trim().to_string())
                .unwrap_or_else(|| "Untitled Spec".to_string());

            let now = chrono::Utc::now().to_rfc3339();
            Ok(Spec {
                metadata: SpecMetadata {
                    id: String::new(),
                    title,
                    created: now.clone(),
                    updated: now,
                    ..Default::default()
                },
                body: content.to_string(),
            })
        }
    }

    fn from_toml_markdown(content: &str) -> Result<Self> {
        let rest = &content[4..]; // skip "+++\n"
        let end = rest
            .find("\n+++")
            .ok_or_else(|| anyhow!("Invalid spec format: missing closing +++"))?;
        let toml_str = &rest[..end];
        let body = rest[end + 4..].trim_start_matches('\n').to_string();
        let metadata: SpecMetadata =
            toml::from_str(toml_str).context("Failed to parse spec TOML frontmatter")?;
        Ok(Spec { metadata, body })
    }
}
