use runtime::{DefaultRuntimeHooks, RuntimeHooks};
use std::path::Path;

pub mod adr;
pub mod feature;
pub mod note;
pub mod release;

pub trait ShipModule: Send + Sync + 'static {
    fn module_type_id() -> &'static str
    where
        Self: Sized;
}

impl ShipModule for crate::ADR {
    fn module_type_id() -> &'static str {
        "adr"
    }
}

impl ShipModule for crate::Feature {
    fn module_type_id() -> &'static str {
        "feature"
    }
}

impl ShipModule for crate::Note {
    fn module_type_id() -> &'static str {
        "note"
    }
}

impl ShipModule for crate::Release {
    fn module_type_id() -> &'static str {
        "release"
    }
}

#[derive(Debug)]
pub enum OpsError {
    NotFound(String),
    InvalidTransition(String, String),
    Validation(String),
    Internal(anyhow::Error),
}

impl std::fmt::Display for OpsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpsError::NotFound(subject) => write!(f, "Not found: {subject}"),
            OpsError::InvalidTransition(from, to) => {
                write!(f, "Invalid status transition: {from} -> {to}")
            }
            OpsError::Validation(message) => write!(f, "Validation failed: {message}"),
            OpsError::Internal(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for OpsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            OpsError::Internal(err) => Some(err.root_cause()),
            _ => None,
        }
    }
}

impl From<anyhow::Error> for OpsError {
    fn from(err: anyhow::Error) -> Self {
        let message = err.to_string();
        if message.to_ascii_lowercase().contains("not found") {
            return OpsError::NotFound(message);
        }
        OpsError::Internal(err)
    }
}

pub type OpsResult<T> = std::result::Result<T, OpsError>;

pub(crate) fn default_hooks() -> DefaultRuntimeHooks {
    DefaultRuntimeHooks
}

pub(crate) fn append_project_log(ship_dir: &Path, action: &str, details: &str) -> OpsResult<()> {
    default_hooks()
        .append_log(ship_dir, action, details)
        .map_err(OpsError::from)
}
