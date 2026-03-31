//! Agent mesh service — registration, directed messaging, broadcast, discovery.
//!
//! The mesh is a thin validator and address resolver. It does not route events
//! itself — it creates `EventEnvelope`s with `target_actor_id` set and pushes
//! them to an outbox channel. The caller drains the outbox through the kernel.

use std::collections::HashMap;

use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::events::{ActorStore, EventEnvelope};
use crate::services::ServiceHandler;

/// Status of a registered agent in the mesh.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Active,
    Busy,
    Idle,
}

/// A registered agent in the mesh registry.
#[derive(Clone, Debug)]
pub struct MeshEntry {
    pub agent_id: String,
    pub label: String,
    pub capabilities: Vec<String>,
    pub registered_at: DateTime<Utc>,
    pub status: AgentStatus,
}

/// Agent mesh service — validates and resolves agent-to-agent messaging.
pub struct MeshService {
    agents: HashMap<String, MeshEntry>,
    outbox: mpsc::UnboundedSender<EventEnvelope>,
}

impl MeshService {
    pub fn new(outbox: mpsc::UnboundedSender<EventEnvelope>) -> Self {
        Self {
            agents: HashMap::new(),
            outbox,
        }
    }

    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    pub fn get_agent(&self, agent_id: &str) -> Option<&MeshEntry> {
        self.agents.get(agent_id)
    }

    fn handle_register(&mut self, event: &EventEnvelope) -> Result<()> {
        let payload: serde_json::Value = serde_json::from_str(&event.payload_json)?;
        let agent_id = payload["agent_id"]
            .as_str()
            .ok_or_else(|| anyhow!("mesh.register: missing agent_id"))?;
        let capabilities: Vec<String> = payload["capabilities"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        self.agents.insert(
            agent_id.to_string(),
            MeshEntry {
                agent_id: agent_id.to_string(),
                label: agent_id.to_string(),
                capabilities,
                registered_at: Utc::now(),
                status: AgentStatus::Active,
            },
        );
        Ok(())
    }

    fn handle_deregister(&mut self, event: &EventEnvelope) -> Result<()> {
        let payload: serde_json::Value = serde_json::from_str(&event.payload_json)?;
        let agent_id = payload["agent_id"]
            .as_str()
            .ok_or_else(|| anyhow!("mesh.deregister: missing agent_id"))?;
        self.agents.remove(agent_id);
        Ok(())
    }

    fn handle_send(&self, event: &EventEnvelope) -> Result<()> {
        let payload: serde_json::Value = serde_json::from_str(&event.payload_json)?;
        let to = payload["to"]
            .as_str()
            .ok_or_else(|| anyhow!("mesh.send: missing 'to' field"))?;
        let sender = sender_from(event, &payload);

        if !self.agents.contains_key(to) {
            let fail = EventEnvelope::new(
                "mesh.send.failed",
                to,
                &serde_json::json!({ "to": to, "reason": "agent not found" }),
            )?
            .with_causation(&event.id)
            .with_target(sender);
            let _ = self.outbox.send(fail);
            return Ok(());
        }

        let msg = EventEnvelope::new(
            "mesh.message",
            to,
            &serde_json::json!({ "from": sender, "body": payload["body"] }),
        )?
        .with_causation(&event.id)
        .with_target(to);
        let _ = self.outbox.send(msg);
        Ok(())
    }

    fn handle_broadcast(&self, event: &EventEnvelope) -> Result<()> {
        let payload: serde_json::Value = serde_json::from_str(&event.payload_json)?;
        let sender = sender_from(event, &payload);
        let cap_filter = payload["capability_filter"].as_str();
        let msg_type = payload["message_type"].as_str().unwrap_or("broadcast");

        for (agent_id, entry) in &self.agents {
            if agent_id == sender {
                continue;
            }
            if let Some(cap) = cap_filter {
                if !entry.capabilities.iter().any(|c| c == cap) {
                    continue;
                }
            }
            let msg = EventEnvelope::new(
                "mesh.message",
                agent_id,
                &serde_json::json!({
                    "from": sender,
                    "message_type": msg_type,
                    "body": payload["body"],
                }),
            )?
            .with_causation(&event.id)
            .with_target(agent_id);
            let _ = self.outbox.send(msg);
        }
        Ok(())
    }

    fn handle_discover(&self, event: &EventEnvelope) -> Result<()> {
        let payload: serde_json::Value = serde_json::from_str(&event.payload_json)?;
        let sender = sender_from(event, &payload);
        let cap_filter = payload["capability"].as_str();
        let status_filter = payload["status"].as_str();

        let matches: Vec<serde_json::Value> = self
            .agents
            .values()
            .filter(|e| {
                if let Some(cap) = cap_filter {
                    if !e.capabilities.iter().any(|c| c == cap) {
                        return false;
                    }
                }
                if let Some(st) = status_filter {
                    if status_str(&e.status) != st {
                        return false;
                    }
                }
                true
            })
            .map(|e| {
                serde_json::json!({
                    "agent_id": e.agent_id,
                    "label": e.label,
                    "capabilities": e.capabilities,
                    "status": e.status,
                })
            })
            .collect();

        let resp = EventEnvelope::new(
            "mesh.discover.response",
            sender,
            &serde_json::json!({ "agents": matches }),
        )?
        .with_causation(&event.id)
        .with_target(sender);
        let _ = self.outbox.send(resp);
        Ok(())
    }

    fn handle_status(&mut self, event: &EventEnvelope) -> Result<()> {
        let payload: serde_json::Value = serde_json::from_str(&event.payload_json)?;
        let agent_id = payload["agent_id"]
            .as_str()
            .ok_or_else(|| anyhow!("mesh.status: missing agent_id"))?;
        let status = parse_status(payload["status"].as_str())?;
        if let Some(entry) = self.agents.get_mut(agent_id) {
            entry.status = status;
        }
        Ok(())
    }
}

impl ServiceHandler for MeshService {
    fn name(&self) -> &str {
        "mesh"
    }

    fn handle(&mut self, event: &EventEnvelope, _store: &ActorStore) -> Result<()> {
        match event.event_type.as_str() {
            "mesh.register" => self.handle_register(event),
            "mesh.deregister" => self.handle_deregister(event),
            "mesh.send" => self.handle_send(event),
            "mesh.broadcast" => self.handle_broadcast(event),
            "mesh.discover.request" => self.handle_discover(event),
            "mesh.status" => self.handle_status(event),
            _ => Ok(()),
        }
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn sender_from<'a>(event: &'a EventEnvelope, payload: &'a serde_json::Value) -> &'a str {
    event
        .actor_id
        .as_deref()
        .or_else(|| payload["from"].as_str())
        .unwrap_or("unknown")
}

fn status_str(s: &AgentStatus) -> &'static str {
    match s {
        AgentStatus::Active => "active",
        AgentStatus::Busy => "busy",
        AgentStatus::Idle => "idle",
    }
}

fn parse_status(s: Option<&str>) -> Result<AgentStatus> {
    match s {
        Some("active") => Ok(AgentStatus::Active),
        Some("busy") => Ok(AgentStatus::Busy),
        Some("idle") => Ok(AgentStatus::Idle),
        Some(other) => Err(anyhow!("mesh.status: unknown status '{other}'")),
        None => Err(anyhow!("mesh.status: missing status")),
    }
}
