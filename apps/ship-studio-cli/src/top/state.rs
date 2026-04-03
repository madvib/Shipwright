//! Data fetching for `ship top` — queries runtime state on each tick.

use std::collections::VecDeque;
use std::path::Path;

pub struct TopState {
    pub workspaces: Vec<WorkspaceRow>,
    pub sessions: Vec<SessionRow>,
    pub events: VecDeque<EventRow>,
    pub agents: Vec<AgentRow>,
    pub network_port: Option<String>,
    pub actor_count: usize,
    pub event_cursor: Option<String>,
    pub error: Option<String>,
}

pub struct WorkspaceRow {
    pub branch: String,
    pub status: String,
    pub agent: String,
    pub providers: String,
    pub session_status: String,
}

pub struct SessionRow {
    pub id_short: String,
    pub branch: String,
    pub status: String,
    pub provider: String,
    pub agent: String,
    pub tool_calls: i64,
    pub started: String,
    pub goal: String,
}

pub struct EventRow {
    pub time: String,
    pub event_type: String,
    pub actor: String,
    pub entity: String,
}

pub struct AgentRow {
    pub agent_id: String,
    pub capabilities: String,
    pub status: String,
    pub registered: String,
}

const MAX_EVENTS: usize = 200;

impl TopState {
    pub fn empty() -> Self {
        Self {
            workspaces: vec![],
            sessions: vec![],
            events: VecDeque::with_capacity(MAX_EVENTS),
            agents: vec![],
            network_port: None,
            actor_count: 0,
            event_cursor: None,
            error: None,
        }
    }
}

/// Full refresh of workspaces, sessions, agents, network. Events are incremental.
pub fn refresh(state: &mut TopState, ship_dir: &Path) {
    state.error = None;
    refresh_workspaces(state, ship_dir);
    refresh_sessions(state);
    refresh_events(state, ship_dir);
    refresh_agents(state);
    refresh_network(state);
}

fn refresh_workspaces(state: &mut TopState, ship_dir: &Path) {
    state.workspaces.clear();
    match runtime::list_workspaces(ship_dir) {
        Ok(ws_list) => {
            for ws in ws_list {
                let session_status =
                    match runtime::get_active_workspace_session(ship_dir, &ws.branch) {
                        Ok(Some(s)) => format!("{}", s.status),
                        Ok(None) => "—".into(),
                        Err(_) => "err".into(),
                    };
                state.workspaces.push(WorkspaceRow {
                    branch: ws.branch,
                    status: format!("{}", ws.status),
                    agent: ws.active_agent.unwrap_or_else(|| "—".into()),
                    providers: "—".into(),
                    session_status,
                });
            }
        }
        Err(e) => state.error = Some(format!("workspaces: {e}")),
    }
}

fn refresh_sessions(state: &mut TopState) {
    state.sessions.clear();
    match runtime::db::session::list_workspace_sessions_db(None, 50) {
        Ok(sessions) => {
            for s in sessions {
                state.sessions.push(SessionRow {
                    id_short: s.id.chars().take(8).collect(),
                    branch: s.workspace_branch,
                    status: s.status,
                    provider: s.primary_provider.unwrap_or_else(|| "—".into()),
                    agent: s.agent_id.unwrap_or_else(|| "—".into()),
                    tool_calls: s.tool_call_count,
                    started: format_time(&s.started_at),
                    goal: s.goal.as_deref().unwrap_or("—").chars().take(40).collect(),
                });
            }
        }
        Err(e) => set_error(state, format!("sessions: {e}")),
    }
}

fn refresh_events(state: &mut TopState, ship_dir: &Path) {
    // First load: get recent events. Subsequent: incremental via cursor.
    if state.event_cursor.is_none() {
        match runtime::read_recent_events(ship_dir, MAX_EVENTS) {
            Ok(events) => {
                for e in events {
                    let id = e.id.clone();
                    state.events.push_back(to_event_row(e));
                    state.event_cursor = Some(id);
                }
            }
            Err(e) => set_error(state, format!("events: {e}")),
        }
        return;
    }
    match runtime::query_events_since(state.event_cursor.as_deref(), false) {
        Ok(new_events) => {
            for e in new_events {
                let id = e.id.clone();
                state.events.push_back(to_event_row(e));
                state.event_cursor = Some(id);
                if state.events.len() > MAX_EVENTS {
                    state.events.pop_front();
                }
            }
        }
        Err(e) => set_error(state, format!("events: {e}")),
    }
}

fn refresh_agents(state: &mut TopState) {
    state.agents.clear();

    // Reconstruct mesh agents from event log.
    match runtime::query_events_since(None, false) {
        Ok(events) => {
            let mut agents: std::collections::HashMap<String, AgentRow> =
                std::collections::HashMap::new();
            for e in &events {
                if e.event_type == "mesh.register" {
                    if let Ok(p) = serde_json::from_str::<serde_json::Value>(&e.payload_json) {
                        let agent_id = p["agent_id"].as_str().unwrap_or("unknown");
                        let caps: Vec<&str> = p["capabilities"]
                            .as_array()
                            .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
                            .unwrap_or_default();
                        agents.insert(
                            agent_id.to_string(),
                            AgentRow {
                                agent_id: agent_id.to_string(),
                                capabilities: if caps.is_empty() {
                                    "—".into()
                                } else {
                                    caps.join(", ")
                                },
                                status: "active".into(),
                                registered: format_time(&e.created_at.to_rfc3339()),
                            },
                        );
                    }
                } else if e.event_type == "mesh.deregister" {
                    if let Ok(p) = serde_json::from_str::<serde_json::Value>(&e.payload_json) {
                        if let Some(id) = p["agent_id"].as_str() {
                            agents.remove(id);
                        }
                    }
                } else if e.event_type == "mesh.status" {
                    if let Ok(p) = serde_json::from_str::<serde_json::Value>(&e.payload_json) {
                        if let (Some(id), Some(status)) =
                            (p["agent_id"].as_str(), p["status"].as_str())
                        {
                            if let Some(entry) = agents.get_mut(id) {
                                entry.status = status.to_string();
                            }
                        }
                    }
                }
            }
            state.agents = agents.into_values().collect();
            state.agents.sort_by(|a, b| a.agent_id.cmp(&b.agent_id));
        }
        Err(_) => {}
    }

    // Actor count from KernelRouter (if initialized).
    if let Some(kr) = runtime::events::kernel_router() {
        if let Ok(kr_guard) = kr.try_lock() {
            state.actor_count = kr_guard.actor_count();
        }
    }
}

fn refresh_network(state: &mut TopState) {
    if let Ok(global_dir) = runtime::project::get_global_dir() {
        state.network_port = std::fs::read_to_string(global_dir.join("network.port"))
            .ok()
            .map(|p| p.trim().to_string());
    }
}

fn to_event_row(e: runtime::EventEnvelope) -> EventRow {
    EventRow {
        time: format_time(&e.created_at.to_rfc3339()),
        event_type: e.event_type,
        actor: e
            .actor_id
            .unwrap_or_else(|| e.actor.chars().take(20).collect()),
        entity: e.entity_id.chars().take(30).collect(),
    }
}

fn format_time(rfc3339: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(rfc3339)
        .map(|dt| dt.format("%H:%M:%S").to_string())
        .unwrap_or_else(|_| rfc3339.chars().take(8).collect())
}

fn set_error(state: &mut TopState, msg: String) {
    if state.error.is_none() {
        state.error = Some(msg);
    }
}
