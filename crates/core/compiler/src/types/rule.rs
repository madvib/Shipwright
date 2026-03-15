use serde::{Deserialize, Serialize};

/// A rule from `agents/rules/*.md`. Always active — no mode/feature filtering.
/// The compiler receives pre-loaded `Rule` values — it does not read files.
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rule {
    pub file_name: String,
    pub content: String,
    /// If true, inject into every agent context (default). False = conditional.
    #[serde(default = "default_always_apply")]
    pub always_apply: bool,
    /// File glob patterns — Cursor "Apply to Specific Files" mode.
    #[serde(default)]
    pub globs: Vec<String>,
    /// Description for "Apply Intelligently" mode (Cursor).
    #[serde(default)]
    pub description: Option<String>,
}

fn default_always_apply() -> bool {
    true
}
