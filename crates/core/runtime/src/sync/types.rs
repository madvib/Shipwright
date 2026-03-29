//! Request/response types for the sync protocol.

use serde::{Deserialize, Serialize};

use crate::events::EventEnvelope;

/// Push request body — sent to POST /api/sync/push or /api/sync/workspace/{id}/push.
#[derive(Debug, Serialize)]
pub struct PushRequest {
    pub project_id: String,
    pub events: Vec<EventEnvelope>,
}

/// Push response — returned by the cloud after accepting events.
#[derive(Debug, Deserialize)]
pub struct PushResponse {
    pub accepted: u64,
    pub cursor: String,
}

/// Pull response — returned by the cloud with events since a cursor.
#[derive(Debug, Deserialize)]
pub struct PullResponse {
    pub events: Vec<EventEnvelope>,
    pub cursor: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn sample_envelope() -> EventEnvelope {
        EventEnvelope {
            id: "01J0000000000000000000000A".to_string(),
            event_type: "workspace.created".to_string(),
            entity_id: "ws-1".to_string(),
            actor: "ship".to_string(),
            payload_json: "{}".to_string(),
            version: 1,
            correlation_id: None,
            causation_id: None,
            workspace_id: Some("ws-1".to_string()),
            session_id: None,
            actor_id: None,
            parent_actor_id: None,
            elevated: true,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn push_request_serializes() {
        let req = PushRequest {
            project_id: "proj-1".to_string(),
            events: vec![sample_envelope()],
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("proj-1"));
        assert!(json.contains("workspace.created"));
    }

    #[test]
    fn push_response_deserializes() {
        let json = r#"{"accepted": 3, "cursor": "01J0000000000000000000000C"}"#;
        let resp: PushResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.accepted, 3);
        assert_eq!(resp.cursor, "01J0000000000000000000000C");
    }

    #[test]
    fn pull_response_deserializes() {
        let json = r#"{
            "events": [{
                "id": "01J0000000000000000000000A",
                "event_type": "workspace.created",
                "entity_id": "ws-1",
                "actor": "ship",
                "payload_json": "{}",
                "version": 1,
                "correlation_id": null,
                "causation_id": null,
                "workspace_id": "ws-1",
                "session_id": null,
                "actor_id": null,
                "parent_actor_id": null,
                "elevated": true,
                "created_at": "2026-03-28T00:00:00Z"
            }],
            "cursor": "01J0000000000000000000000A"
        }"#;
        let resp: PullResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.events.len(), 1);
        assert_eq!(resp.cursor, "01J0000000000000000000000A");
    }

    #[test]
    fn pull_response_empty_events() {
        let json = r#"{"events": [], "cursor": ""}"#;
        let resp: PullResponse = serde_json::from_str(json).unwrap();
        assert!(resp.events.is_empty());
    }
}
