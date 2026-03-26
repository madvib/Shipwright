//! Skill variable state management — vars.yaml parsing, state I/O, CLI commands.
//!
//! State files:
//! - Project scope: `.ship/state/skills/{id}.json`
//! - User scope:    `~/.ship/state/skills/{id}.json`
//!
//! Merge order (last wins): defaults → user state → project state.

pub mod commands;
pub mod schema;
pub mod state;

pub use commands::{
    run_vars_append, run_vars_edit, run_vars_get, run_vars_reset, run_vars_set,
};
pub use schema::{load_vars_json, parse_vars_json};
pub use state::read_skill_state;
