//! Event validation — ingress checks before persistence.

use std::fmt;

use crate::events::EventEnvelope;

/// Who is emitting the event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallerKind {
    Cli,
    Mcp,
    Sdk,
    CloudSync,
    Runtime,
}

/// Context provided alongside every emit call.
pub struct EmitContext {
    pub caller_kind: CallerKind,
    pub skill_id: Option<String>,
    pub workspace_id: Option<String>,
    pub session_id: Option<String>,
}

/// Reasons an event can be rejected before persistence.
#[derive(Debug)]
pub enum ValidationError {
    NamespaceViolation { expected: String, got: String },
    ReservedNamespace { namespace: String },
    DirectionViolation { event_type: String, direction: String },
    SchemaViolation { event_type: String, error: String },
    RateLimited { producer: String },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NamespaceViolation { expected, got } => {
                write!(f, "namespace violation: expected '{expected}.*', got '{got}'")
            }
            Self::ReservedNamespace { namespace } => {
                write!(f, "reserved namespace: '{namespace}'")
            }
            Self::DirectionViolation {
                event_type,
                direction,
            } => {
                write!(f, "direction violation: '{event_type}' cannot be emitted {direction}")
            }
            Self::SchemaViolation { event_type, error } => {
                write!(f, "schema violation for '{event_type}': {error}")
            }
            Self::RateLimited { producer } => {
                write!(f, "rate limited: producer '{producer}'")
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Validates an event before it is persisted.
pub trait EventValidator: Send + Sync {
    fn validate(&self, event: &EventEnvelope, ctx: &EmitContext)
        -> Result<(), ValidationError>;
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Namespaces reserved by the platform. Agents cannot emit events with these
/// prefixes. This is the single source of truth — MCP tools and validators
/// both reference this list.
pub const RESERVED_NAMESPACES: &[&str] = &[
    "actor.",
    "config.",
    "gate.",
    "job.",
    "mesh.",
    "project.",
    "runtime.",
    "session.",
    "skill.",
    "studio.",
    "sync.",
    "workspace.",
];

fn is_system_event(event_type: &str) -> bool {
    RESERVED_NAMESPACES
        .iter()
        .any(|ns| event_type.starts_with(ns))
}

fn is_trusted_caller(kind: &CallerKind) -> bool {
    matches!(kind, CallerKind::Runtime | CallerKind::Cli)
}

// ── Built-in validators ──────────────────────────────────────────────────────

/// Skills can only emit events in their own namespace (`{skill_id}.*`).
/// System namespaces are always reserved.
pub struct NamespaceValidator;

impl EventValidator for NamespaceValidator {
    fn validate(
        &self,
        event: &EventEnvelope,
        ctx: &EmitContext,
    ) -> Result<(), ValidationError> {
        if is_trusted_caller(&ctx.caller_kind) {
            return Ok(());
        }
        if let Some(ref skill_id) = ctx.skill_id {
            let prefix = format!("{skill_id}.");
            if !event.event_type.starts_with(&prefix) {
                return Err(ValidationError::NamespaceViolation {
                    expected: skill_id.clone(),
                    got: event.event_type.clone(),
                });
            }
            if is_system_event(&event.event_type) {
                return Err(ValidationError::ReservedNamespace {
                    namespace: event.event_type.clone(),
                });
            }
        }
        Ok(())
    }
}

/// Blocks non-trusted callers from emitting system-namespace events.
pub struct ReservedNamespaceValidator;

impl EventValidator for ReservedNamespaceValidator {
    fn validate(
        &self,
        event: &EventEnvelope,
        ctx: &EmitContext,
    ) -> Result<(), ValidationError> {
        if is_trusted_caller(&ctx.caller_kind) {
            return Ok(());
        }
        if is_system_event(&event.event_type) {
            return Err(ValidationError::ReservedNamespace {
                namespace: event.event_type.clone(),
            });
        }
        Ok(())
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EventEnvelope;

    fn skill_ctx(skill_id: &str) -> EmitContext {
        EmitContext {
            caller_kind: CallerKind::Mcp,
            skill_id: Some(skill_id.to_string()),
            workspace_id: None,
            session_id: None,
        }
    }

    fn caller_ctx(kind: CallerKind) -> EmitContext {
        EmitContext {
            caller_kind: kind,
            skill_id: None,
            workspace_id: None,
            session_id: None,
        }
    }

    fn make_event(event_type: &str) -> EventEnvelope {
        EventEnvelope::new(event_type, "entity-1", &serde_json::json!({})).unwrap()
    }

    #[test]
    fn namespace_validator_allows_matching_skill() {
        let v = NamespaceValidator;
        let event = make_event("mcp-setup.server_ready");
        let ctx = skill_ctx("mcp-setup");
        assert!(v.validate(&event, &ctx).is_ok());
    }

    #[test]
    fn namespace_validator_rejects_wrong_skill() {
        let v = NamespaceValidator;
        let event = make_event("visual-brainstorm.annotation");
        let ctx = skill_ctx("mcp-setup");
        let err = v.validate(&event, &ctx).unwrap_err();
        assert!(matches!(err, ValidationError::NamespaceViolation { .. }));
    }

    #[test]
    fn namespace_validator_blocks_system_namespace() {
        let v = NamespaceValidator;
        let event = make_event("workspace.created");
        let ctx = skill_ctx("workspace");
        let err = v.validate(&event, &ctx).unwrap_err();
        assert!(matches!(err, ValidationError::ReservedNamespace { .. }));
    }

    #[test]
    fn runtime_caller_can_emit_system_events() {
        let v = NamespaceValidator;
        let event = make_event("workspace.created");
        let ctx = caller_ctx(CallerKind::Runtime);
        assert!(v.validate(&event, &ctx).is_ok());
    }

    #[test]
    fn cli_caller_can_emit_system_events() {
        let v = NamespaceValidator;
        let event = make_event("session.started");
        let ctx = caller_ctx(CallerKind::Cli);
        assert!(v.validate(&event, &ctx).is_ok());
    }
}
