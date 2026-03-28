// MCP tool: ship_event — allows agents to emit domain events into the Ship event store.
//
// actor_id and workspace_id are injected from MCP connection context.
// Agents cannot supply or override these values.
//
// Platform-reserved prefixes ("actor.", "session.", etc.) are blocked.
// Only domain events are accepted.
//
// Implementation pending. Signatures below define the contract exercised by tests.

// pub const RESERVED_PREFIXES: &[&str] = &[
//     "actor.", "session.", "skill.", "workspace.", "gate.", "job.", "config.", "project.",
// ];

// pub fn handle_ship_event(
//     actor_id: &str,         // from MCP connection context — not agent-controlled
//     workspace_id: &str,     // from MCP connection context — not agent-controlled
//     event_type: &str,       // from agent request — validated against reserved list
//     payload: serde_json::Value, // from agent request
//     elevated: bool,         // from agent request
// ) -> anyhow::Result<runtime::events::EventEnvelope> {
//     todo!()
// }

#[cfg(test)]
#[path = "event_tests.rs"]
mod event_tests;
