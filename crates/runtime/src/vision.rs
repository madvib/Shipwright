use crate::fs_util::write_atomic;
use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct VisionMetadata {
    pub title: String,
    pub updated: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct Vision {
    pub metadata: VisionMetadata,
    pub body: String,
}

impl Vision {
    pub fn from_markdown(content: &str) -> Result<Self> {
        if content.starts_with("+++\n") {
            let parts: Vec<&str> = content.splitn(3, "+++\n").collect();
            if parts.len() < 3 {
                return Err(anyhow!("Invalid Vision frontmatter format"));
            }
            let metadata: VisionMetadata =
                toml::from_str(parts[1]).with_context(|| "Failed to parse Vision frontmatter")?;
            Ok(Vision {
                metadata,
                body: parts[2].trim_start().to_string(),
            })
        } else {
            Ok(Vision {
                metadata: VisionMetadata {
                    title: "Vision".to_string(),
                    updated: Utc::now().to_rfc3339(),
                },
                body: content.to_string(),
            })
        }
    }

    pub fn to_markdown(&self) -> Result<String> {
        let toml_str =
            toml::to_string(&self.metadata).context("Failed to serialise Vision metadata")?;
        Ok(format!("+++\n{}+++\n\n{}", toml_str, self.body))
    }
}

// ─── CRUD ─────────────────────────────────────────────────────────────────────

fn vision_path(ship_dir: &std::path::Path) -> PathBuf {
    crate::project::project_ns(ship_dir).join("vision.md")
}

pub fn get_vision(ship_dir: PathBuf) -> Result<Vision> {
    let path = vision_path(&ship_dir);
    let content = if path.exists() {
        fs::read_to_string(&path)
            .with_context(|| format!("Failed to read vision.md: {}", path.display()))?
    } else {
        crate::project::read_template(&ship_dir, "vision")?
    };
    Vision::from_markdown(&content)
}

pub fn update_vision(ship_dir: PathBuf, vision: &Vision) -> Result<()> {
    let path = vision_path(&ship_dir);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = vision.to_markdown()?;
    write_atomic(&path, content).with_context(|| "Failed to write vision.md")
}
