/// Markdown serialization for ADR committed files.
use super::types::{ADR, AdrMetadata, AdrStatus};
use anyhow::{Context, Result, anyhow};

impl ADR {
    pub fn to_markdown(&self) -> Result<String> {
        let toml_str =
            toml::to_string(&self.metadata).context("Failed to serialise ADR metadata as TOML")?;
        let mut out = format!(
            "<!-- \n  GENERATED FILE — DO NOT EDIT DIRECTLY\n  This file is exported from the Ship SQLite database.\n  Changes here will NOT be synchronized back to the database.\n-->\n\n+++\n{}+++\n",
            toml_str
        );
        if !self.context.trim().is_empty() {
            out.push_str(&format!("\n## Context\n\n{}\n", self.context.trim()));
        }
        if !self.decision.trim().is_empty() {
            out.push_str(&format!("\n## Decision\n\n{}\n", self.decision.trim()));
        }
        Ok(out)
    }

    pub fn from_markdown(content: &str) -> Result<Self> {
        // Strip HTML comment prefix that to_markdown writes (e.g. "<!-- GENERATED FILE ... -->")
        let content = if content.starts_with("<!--") {
            if let Some(end) = content.find("-->") {
                content[end + 3..].trim_start_matches('\n')
            } else {
                content
            }
        } else {
            content
        };
        if !content.starts_with("+++\n") {
            return Err(anyhow!("Invalid ADR format: missing TOML frontmatter"));
        }
        let rest = &content[4..];
        let end = rest
            .find("\n+++")
            .ok_or_else(|| anyhow!("Invalid ADR format: missing closing +++"))?;
        let toml_str = &rest[..end];
        let body = rest[end + 4..].trim_start_matches('\n').to_string();
        let metadata: AdrMetadata =
            toml::from_str(toml_str).context("Failed to parse ADR TOML frontmatter")?;
        let (context, decision) = split_body(&body);
        Ok(ADR {
            metadata,
            context,
            decision,
        })
    }
}

pub fn split_body(body: &str) -> (String, String) {
    let lower = body.to_lowercase();
    if let Some(pos) = find_h2_decision(&lower) {
        let context = body[..pos].trim().to_string();
        let after = &body[pos..];
        let decision_body = after
            .splitn(2, '\n')
            .nth(1)
            .unwrap_or("")
            .trim()
            .to_string();
        (context, decision_body)
    } else {
        (body.trim().to_string(), String::new())
    }
}

fn find_h2_decision(lower_body: &str) -> Option<usize> {
    for (i, line) in lower_body.lines().enumerate() {
        if line.starts_with("## decision") {
            let byte_offset: usize = lower_body.lines().take(i).map(|l| l.len() + 1).sum();
            return Some(byte_offset);
        }
    }
    None
}

pub fn status_dir_name(status: &AdrStatus) -> &'static str {
    match status {
        AdrStatus::Proposed => "proposed",
        AdrStatus::Accepted => "accepted",
        AdrStatus::Rejected => "rejected",
        AdrStatus::Superseded => "superseded",
        AdrStatus::Deprecated => "deprecated",
    }
}
