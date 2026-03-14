use serde::{Deserialize, Serialize};

/// A rule from `agents/rules/*.md`. Always active — no mode/feature filtering.
/// The compiler receives pre-loaded `Rule` values — it does not read files.
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rule {
    pub file_name: String,
    pub content: String,
}
