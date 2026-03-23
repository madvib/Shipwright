use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct CreateTargetRequest {
    /// Target kind: "milestone" (e.g. v0.1.0) or "surface" (e.g. compiler, studio)
    pub kind: String,
    /// Short title
    pub title: String,
    /// Optional longer description
    pub description: Option<String>,
    /// One-line north star goal
    pub goal: Option<String>,
    /// Status: "active" | "planned" | "complete" | "frozen". Defaults to "active".
    pub status: Option<String>,
    /// Current phase: "alpha" | "beta" | "stable" | "frozen" (or any label)
    pub phase: Option<String>,
    /// Target due date (ISO 8601 date, e.g. "2026-06-01")
    pub due_date: Option<String>,
    /// Long-form markdown: strategy, constraints, decisions, open questions
    pub body_markdown: Option<String>,
    /// File/directory paths owned by this target
    pub file_scope: Option<Vec<String>>,
}

#[derive(Deserialize, JsonSchema)]
pub struct UpdateTargetRequest {
    /// Target id to update
    pub id: String,
    /// New title
    pub title: Option<String>,
    /// New description
    pub description: Option<String>,
    /// New goal
    pub goal: Option<String>,
    /// New status
    pub status: Option<String>,
    /// New phase
    pub phase: Option<String>,
    /// New due date
    pub due_date: Option<String>,
    /// Replace the long-form markdown body
    pub body_markdown: Option<String>,
    /// Replace the file scope list
    pub file_scope: Option<Vec<String>>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ListTargetsRequest {
    /// Filter by kind: "milestone" | "surface"
    pub kind: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GetTargetRequest {
    /// Target id
    pub id: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct CreateCapabilityRequest {
    /// Target this capability belongs to
    pub target_id: String,
    /// Capability title
    pub title: String,
    /// Optional milestone target id this capability is required for
    pub milestone_id: Option<String>,
    /// Phase grouping within the target (e.g. "bootstrap", "core", "polish")
    pub phase: Option<String>,
    /// Acceptance criteria — checklist items that define "done"
    pub acceptance_criteria: Option<Vec<String>>,
    /// File/directory paths this capability is scoped to
    pub file_scope: Option<Vec<String>>,
    /// Agent or workspace id currently assigned to this capability
    pub assigned_to: Option<String>,
    /// Scheduling priority — lower numbers run first (default 0)
    pub priority: Option<i32>,
}

#[derive(Deserialize, JsonSchema)]
pub struct UpdateCapabilityRequest {
    /// Capability id to update
    pub id: String,
    /// New title
    pub title: Option<String>,
    /// New status: "aspirational" | "in_progress" | "actual"
    pub status: Option<String>,
    /// New phase
    pub phase: Option<String>,
    /// Replace acceptance criteria checklist
    pub acceptance_criteria: Option<Vec<String>>,
    /// Replace file scope
    pub file_scope: Option<Vec<String>>,
    /// Assign to agent or workspace id
    pub assigned_to: Option<String>,
    /// New priority
    pub priority: Option<i32>,
}

#[derive(Deserialize, JsonSchema)]
pub struct DeleteCapabilityRequest {
    /// Capability id to delete
    pub id: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct MarkCapabilityActualRequest {
    /// Capability id
    pub id: String,
    /// Evidence that proves this capability is actual (test name, commit, URL)
    pub evidence: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct ListCapabilitiesRequest {
    /// Filter by surface target id
    pub target_id: Option<String>,
    /// Filter by milestone id — returns capabilities across surfaces linked to this milestone
    pub milestone_id: Option<String>,
    /// Filter by status: "aspirational" | "in_progress" | "actual"
    pub status: Option<String>,
    /// Filter by phase (e.g. "bootstrap", "core", "polish")
    pub phase: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ListEventsRequest {
    /// Show events since this timestamp (ISO 8601) or relative ("1h", "24h", "7d")
    pub since: Option<String>,
    /// Filter by actor (substring match)
    pub actor: Option<String>,
    /// Filter by entity type: workspace, session, note, adr, config, etc.
    pub entity: Option<String>,
    /// Filter by action: create, update, delete, start, stop, etc.
    pub action: Option<String>,
    /// Maximum number of events to return (default: 50, max: 200)
    pub limit: Option<u32>,
}
