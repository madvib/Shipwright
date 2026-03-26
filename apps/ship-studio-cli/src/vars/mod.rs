//! Skill variable state management — vars.json parsing and CLI commands.
//!
//! Storage is delegated to `runtime::skill_vars`:
//! - User-scoped: platform.db KV
//! - Project-scoped: .ship/state.json

pub mod commands;
pub mod schema;
pub mod state;

pub use commands::{
    run_vars_append, run_vars_edit, run_vars_get, run_vars_reset, run_vars_set,
};
pub use schema::{load_vars_json, warn_invalid_enum_vars};
