use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum EventEntity {
    Project,
    Workspace,
    Session,
    Note,
    Adr,
    Config,
    Mode,
    Plugin,
    Ghost,
    Time,
    Agent,
    Mcp,
    Gate,
    Capability,
    Target,
    Job,
}

impl EventEntity {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventEntity::Project => "project",
            EventEntity::Workspace => "workspace",
            EventEntity::Session => "session",
            EventEntity::Note => "note",
            EventEntity::Adr => "adr",
            EventEntity::Config => "config",
            EventEntity::Mode => "mode",
            EventEntity::Plugin => "plugin",
            EventEntity::Ghost => "ghost",
            EventEntity::Time => "time",
            EventEntity::Agent => "agent",
            EventEntity::Mcp => "mcp",
            EventEntity::Gate => "gate",
            EventEntity::Capability => "capability",
            EventEntity::Target => "target",
            EventEntity::Job => "job",
        }
    }

    pub(crate) fn from_db(value: &str) -> Result<Self> {
        match value {
            "project" => Ok(EventEntity::Project),
            "workspace" => Ok(EventEntity::Workspace),
            "session" => Ok(EventEntity::Session),
            "note" => Ok(EventEntity::Note),
            "adr" => Ok(EventEntity::Adr),
            "config" => Ok(EventEntity::Config),
            "mode" => Ok(EventEntity::Mode),
            "plugin" => Ok(EventEntity::Plugin),
            "ghost" => Ok(EventEntity::Ghost),
            "time" => Ok(EventEntity::Time),
            "agent" => Ok(EventEntity::Agent),
            "mcp" => Ok(EventEntity::Mcp),
            "gate" => Ok(EventEntity::Gate),
            "capability" => Ok(EventEntity::Capability),
            "target" => Ok(EventEntity::Target),
            "job" => Ok(EventEntity::Job),
            other => Err(anyhow!("Unknown event entity '{}'", other)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum EventAction {
    Init,
    Create,
    Update,
    Delete,
    Move,
    Note,
    Link,
    Add,
    Remove,
    Set,
    Clear,
    Scan,
    Promote,
    Start,
    Stop,
    Log,
    Pass,
    Fail,
    Complete,
    Claim,
    Dispatch,
}

impl EventAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventAction::Init => "init",
            EventAction::Create => "create",
            EventAction::Update => "update",
            EventAction::Delete => "delete",
            EventAction::Move => "move",
            EventAction::Note => "note",
            EventAction::Link => "link",
            EventAction::Add => "add",
            EventAction::Remove => "remove",
            EventAction::Set => "set",
            EventAction::Clear => "clear",
            EventAction::Scan => "scan",
            EventAction::Promote => "promote",
            EventAction::Start => "start",
            EventAction::Stop => "stop",
            EventAction::Log => "log",
            EventAction::Pass => "pass",
            EventAction::Fail => "fail",
            EventAction::Complete => "complete",
            EventAction::Claim => "claim",
            EventAction::Dispatch => "dispatch",
        }
    }

    pub(crate) fn from_db(value: &str) -> Result<Self> {
        match value {
            "init" => Ok(EventAction::Init),
            "create" => Ok(EventAction::Create),
            "update" => Ok(EventAction::Update),
            "delete" => Ok(EventAction::Delete),
            "move" => Ok(EventAction::Move),
            "note" => Ok(EventAction::Note),
            "link" => Ok(EventAction::Link),
            "add" => Ok(EventAction::Add),
            "remove" => Ok(EventAction::Remove),
            "set" => Ok(EventAction::Set),
            "clear" => Ok(EventAction::Clear),
            "scan" => Ok(EventAction::Scan),
            "promote" => Ok(EventAction::Promote),
            "start" => Ok(EventAction::Start),
            "stop" => Ok(EventAction::Stop),
            "log" => Ok(EventAction::Log),
            "pass" => Ok(EventAction::Pass),
            "fail" => Ok(EventAction::Fail),
            "complete" => Ok(EventAction::Complete),
            "claim" => Ok(EventAction::Claim),
            "dispatch" => Ok(EventAction::Dispatch),
            other => Err(anyhow!("Unknown event action '{}'", other)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct EventRecord {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub actor: String,
    pub entity: EventEntity,
    pub action: EventAction,
    pub subject: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_id: Option<String>,
}

/// Optional context for event insertion.
#[derive(Debug, Default, Clone)]
pub struct EventContext<'a> {
    pub workspace_id: Option<&'a str>,
    pub session_id: Option<&'a str>,
    pub job_id: Option<&'a str>,
}

// ─── Public API ─────────────────────────────────────────────────────────────

pub fn append_event(
    ship_dir: &Path,
    actor: &str,
    entity: EventEntity,
    action: EventAction,
    subject: impl Into<String>,
    details: Option<String>,
) -> Result<EventRecord> {
    append_event_with_context(ship_dir, actor, entity, action, subject, details, &EventContext::default())
}

pub fn append_event_with_context(
    ship_dir: &Path,
    actor: &str,
    entity: EventEntity,
    action: EventAction,
    subject: impl Into<String>,
    details: Option<String>,
    ctx: &EventContext<'_>,
) -> Result<EventRecord> {
    let subject = subject.into();
    let entity_id = if subject.is_empty() { None } else { Some(subject.as_str()) };
    crate::db::events::insert_event(
        ship_dir,
        actor,
        &entity,
        entity_id,
        &action,
        details.as_deref(),
        ctx.workspace_id,
        ctx.session_id,
        ctx.job_id,
    )
}

pub fn read_events(ship_dir: &Path) -> Result<Vec<EventRecord>> {
    crate::db::events::list_all_events(ship_dir)
}

pub fn list_events_since(
    ship_dir: &Path,
    since: &DateTime<Utc>,
    limit: Option<usize>,
) -> Result<Vec<EventRecord>> {
    crate::db::events::list_events_since_time(ship_dir, since, limit)
}

pub fn read_recent_events(ship_dir: &Path, limit: usize) -> Result<Vec<EventRecord>> {
    crate::db::events::list_recent_events(ship_dir, limit)
}

/// Record a gate pass/fail outcome as a structured event.
///
/// - entity = Gate, entity_id = job_id, action = Pass or Fail
/// - detail = evidence string
/// - On pass, job status is set to "complete"
/// - On fail, job status stays "running" (retryable)
pub fn record_gate_outcome(
    ship_dir: &Path,
    job_id: &str,
    passed: bool,
    evidence: &str,
) -> Result<EventRecord> {
    crate::db::events::record_gate_outcome(ship_dir, job_id, passed, evidence)
}

/// List all gate outcomes (pass/fail events) for a given job.
pub fn list_gate_outcomes(ship_dir: &Path, job_id: &str) -> Result<Vec<EventRecord>> {
    crate::db::events::list_gate_outcomes(ship_dir, job_id)
}
