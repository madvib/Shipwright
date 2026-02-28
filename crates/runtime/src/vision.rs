use crate::fs_util::write_atomic;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::fs;
use std::path::PathBuf;

// ─── Data types ───────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct Vision {
    pub content: String,
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
        String::new()
    };
    Ok(Vision { content })
}

pub fn update_vision(ship_dir: PathBuf, content: &str) -> Result<()> {
    let path = vision_path(&ship_dir);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    write_atomic(&path, content).with_context(|| "Failed to write vision.md")
}
