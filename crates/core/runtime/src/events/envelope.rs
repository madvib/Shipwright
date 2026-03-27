use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub id: String,
    pub event_type: String,
    pub entity_id: String,
    pub actor: String,
    pub payload_json: String,
    pub version: u32,
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
    pub workspace_id: Option<String>,
    pub session_id: Option<String>,
    pub actor_id: Option<String>,
    pub parent_actor_id: Option<String>,
    pub elevated: bool,
    pub created_at: DateTime<Utc>,
}

impl EventEnvelope {
    pub fn new<P: Serialize>(event_type: &str, entity_id: &str, payload: &P) -> Result<Self> {
        let payload_json = serde_json::to_string(payload)?;
        Ok(Self {
            id: Ulid::new().to_string(),
            event_type: event_type.to_string(),
            entity_id: entity_id.to_string(),
            actor: "ship".to_string(),
            payload_json,
            version: 1,
            correlation_id: None,
            causation_id: None,
            workspace_id: None,
            session_id: None,
            actor_id: None,
            parent_actor_id: None,
            elevated: false,
            created_at: Utc::now(),
        })
    }

    pub fn with_correlation(mut self, correlation_id: &str) -> Self {
        self.correlation_id = Some(correlation_id.to_string());
        self
    }

    pub fn with_causation(mut self, causation_id: &str) -> Self {
        self.causation_id = Some(causation_id.to_string());
        self
    }

    pub fn with_context(mut self, workspace_id: Option<&str>, session_id: Option<&str>) -> Self {
        self.workspace_id = workspace_id.map(str::to_string);
        self.session_id = session_id.map(str::to_string);
        self
    }

    pub fn with_actor_id(mut self, actor_id: &str) -> Self {
        self.actor_id = Some(actor_id.to_string());
        self
    }

    pub fn with_parent_actor_id(mut self, parent_actor_id: &str) -> Self {
        self.parent_actor_id = Some(parent_actor_id.to_string());
        self
    }

    pub fn elevate(mut self) -> Self {
        self.elevated = true;
        self
    }
}
