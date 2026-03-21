use std::path::Path;

use crate::requests::{
    CreateCapabilityRequest, CreateTargetRequest, DeleteCapabilityRequest, GetTargetRequest,
    ListCapabilitiesRequest, ListTargetsRequest, MarkCapabilityActualRequest,
    UpdateCapabilityRequest, UpdateTargetRequest,
};

pub fn create_target(project_dir: &Path, req: CreateTargetRequest) -> String {
    let ship_dir = project_dir.join(".ship");
    let t = match runtime::db::targets::create_target(
        &ship_dir,
        &req.kind,
        &req.title,
        req.description.as_deref(),
        req.goal.as_deref(),
        req.status.as_deref(),
    ) {
        Ok(t) => t,
        Err(e) => return format!("Error creating target: {}", e),
    };
    // Apply new fields if provided.
    let needs_update = req.phase.is_some()
        || req.due_date.is_some()
        || req.body_markdown.is_some()
        || req.file_scope.is_some();
    if needs_update {
        let patch = runtime::db::targets::TargetPatch {
            phase: req.phase,
            due_date: req.due_date,
            body_markdown: req.body_markdown,
            file_scope: req.file_scope,
            ..Default::default()
        };
        if let Err(e) = runtime::db::targets::update_target(&ship_dir, &t.id, patch) {
            return format!("Created target {} but failed to apply extra fields: {}", t.id, e);
        }
    }
    format!("Created target: {} (id: {}, kind: {})", t.title, t.id, t.kind)
}

pub fn update_target(project_dir: &Path, req: UpdateTargetRequest) -> String {
    let ship_dir = project_dir.join(".ship");
    let patch = runtime::db::targets::TargetPatch {
        title: req.title,
        description: req.description,
        goal: req.goal,
        status: req.status,
        phase: req.phase,
        due_date: req.due_date,
        body_markdown: req.body_markdown,
        file_scope: req.file_scope,
    };
    match runtime::db::targets::update_target(&ship_dir, &req.id, patch) {
        Ok(()) => format!("Updated target {}.", req.id),
        Err(e) => format!("Error updating target: {}", e),
    }
}

pub fn list_targets(project_dir: &Path, req: ListTargetsRequest) -> String {
    let ship_dir = project_dir.join(".ship");
    match runtime::db::targets::list_targets(&ship_dir, req.kind.as_deref()) {
        Ok(ts) if ts.is_empty() => "No targets found.".to_string(),
        Ok(ts) => {
            let mut out = String::from("Targets:\n");
            for t in &ts {
                out.push_str(&format!(
                    "- [{}] {} — {} ({})\n",
                    t.kind, t.id, t.title, t.status
                ));
                if let Some(ref p) = t.phase {
                    out.push_str(&format!("  phase: {}\n", p));
                }
                if let Some(ref g) = t.goal {
                    out.push_str(&format!("  goal: {}\n", g));
                }
            }
            out
        }
        Err(e) => format!("Error listing targets: {}", e),
    }
}

pub fn get_target(project_dir: &Path, req: GetTargetRequest) -> String {
    let ship_dir = project_dir.join(".ship");
    let target = match runtime::db::targets::get_target(&ship_dir, &req.id) {
        Ok(Some(t)) => t,
        Ok(None) => return format!("Target '{}' not found.", req.id),
        Err(e) => return format!("Error: {}", e),
    };
    let caps = if target.kind == "milestone" {
        match runtime::db::targets::list_capabilities_for_milestone(&ship_dir, &req.id, None) {
            Ok(c) => c,
            Err(e) => return format!("Error loading capabilities: {}", e),
        }
    } else {
        match runtime::db::targets::list_capabilities(&ship_dir, Some(&req.id), None, None) {
            Ok(c) => c,
            Err(e) => return format!("Error loading capabilities: {}", e),
        }
    };

    let actual: Vec<_> = caps.iter().filter(|c| c.status == "actual").collect();
    let in_progress: Vec<_> = caps.iter().filter(|c| c.status == "in_progress").collect();
    let aspirational: Vec<_> = caps.iter().filter(|c| c.status == "aspirational").collect();
    let total = caps.len();
    let done = actual.len();

    let mut out = format!(
        "# {} — {} ({})\n",
        target.kind.to_uppercase(),
        target.title,
        target.status
    );
    if let Some(ref p) = target.phase {
        out.push_str(&format!("Phase: {} ", p));
    }
    if let Some(ref d) = target.due_date {
        out.push_str(&format!("Due: {}", d));
    }
    if target.phase.is_some() || target.due_date.is_some() {
        out.push('\n');
    }
    if total > 0 {
        out.push_str(&format!("Progress: {}/{} done\n", done, total));
    }
    if let Some(ref g) = target.goal {
        out.push_str(&format!("Goal: {}\n", g));
    }
    if let Some(ref d) = target.description {
        out.push_str(&format!("{}\n", d));
    }
    if let Some(ref body) = target.body_markdown {
        out.push_str(&format!("\n{}\n", body));
    }
    if !actual.is_empty() {
        out.push_str("\n## Done\n");
        for c in &actual {
            out.push_str(&format!("- [x] {} (id: {})", c.title, c.id));
            if let Some(ref e) = c.evidence {
                out.push_str(&format!(" — {}", e));
            }
            out.push('\n');
        }
    }
    if !in_progress.is_empty() {
        out.push_str("\n## In Progress\n");
        for c in &in_progress {
            out.push_str(&format!("- [ ] {} (id: {})", c.title, c.id));
            if let Some(ref a) = c.assigned_to {
                out.push_str(&format!(" — assigned: {}", a));
            }
            out.push('\n');
        }
    }
    if !aspirational.is_empty() {
        out.push_str("\n## Planned\n");
        for c in &aspirational {
            let phase_tag = c.phase.as_deref().map(|p| format!(" [{}]", p)).unwrap_or_default();
            out.push_str(&format!("- [ ] {}{} (id: {})\n", c.title, phase_tag, c.id));
        }
    }
    if caps.is_empty() {
        out.push_str("\nNo capabilities yet.\n");
    }
    out
}

pub fn create_capability(project_dir: &Path, req: CreateCapabilityRequest) -> String {
    let ship_dir = project_dir.join(".ship");
    let c = match runtime::db::targets::create_capability(
        &ship_dir,
        &req.target_id,
        &req.title,
        req.milestone_id.as_deref(),
    ) {
        Ok(c) => c,
        Err(e) => return format!("Error creating capability: {}", e),
    };
    let needs_update = req.phase.is_some()
        || req.acceptance_criteria.is_some()
        || req.preset_hint.is_some()
        || req.file_scope.is_some()
        || req.assigned_to.is_some()
        || req.priority.is_some();
    if needs_update {
        let patch = runtime::db::targets::CapabilityPatch {
            phase: req.phase,
            acceptance_criteria: req.acceptance_criteria,
            preset_hint: req.preset_hint,
            file_scope: req.file_scope,
            assigned_to: req.assigned_to,
            priority: req.priority,
            ..Default::default()
        };
        if let Err(e) = runtime::db::targets::update_capability(&ship_dir, &c.id, patch) {
            return format!("Created capability {} but failed to apply extra fields: {}", c.id, e);
        }
    }
    format!("Created capability: {} (id: {})", c.title, c.id)
}

pub fn update_capability(project_dir: &Path, req: UpdateCapabilityRequest) -> String {
    let ship_dir = project_dir.join(".ship");
    let patch = runtime::db::targets::CapabilityPatch {
        title: req.title,
        status: req.status,
        phase: req.phase,
        acceptance_criteria: req.acceptance_criteria,
        preset_hint: req.preset_hint,
        file_scope: req.file_scope,
        assigned_to: req.assigned_to,
        priority: req.priority,
    };
    match runtime::db::targets::update_capability(&ship_dir, &req.id, patch) {
        Ok(()) => format!("Updated capability {}.", req.id),
        Err(e) => format!("Error updating capability: {}", e),
    }
}

pub fn delete_capability(project_dir: &Path, req: DeleteCapabilityRequest) -> String {
    let ship_dir = project_dir.join(".ship");
    match runtime::db::targets::delete_capability(&ship_dir, &req.id) {
        Ok(true) => format!("Deleted capability {}.", req.id),
        Ok(false) => format!("Capability '{}' not found.", req.id),
        Err(e) => format!("Error deleting capability: {}", e),
    }
}

pub fn mark_capability_actual(project_dir: &Path, req: MarkCapabilityActualRequest) -> String {
    let ship_dir = project_dir.join(".ship");
    match runtime::db::targets::mark_capability_actual(&ship_dir, &req.id, &req.evidence) {
        Ok(()) => format!("Capability {} marked actual.", req.id),
        Err(e) => format!("Error: {}", e),
    }
}

pub fn list_capabilities(project_dir: &Path, req: ListCapabilitiesRequest) -> String {
    let ship_dir = project_dir.join(".ship");
    let result = if let Some(ref mid) = req.milestone_id {
        runtime::db::targets::list_capabilities_for_milestone(&ship_dir, mid, req.status.as_deref())
    } else {
        runtime::db::targets::list_capabilities(
            &ship_dir,
            req.target_id.as_deref(),
            req.status.as_deref(),
            req.phase.as_deref(),
        )
    };
    match result {
        Ok(cs) if cs.is_empty() => "No capabilities found.".to_string(),
        Ok(cs) => {
            let mut out = String::from("Capabilities:\n");
            for c in &cs {
                let check = match c.status.as_str() {
                    "actual" => "x",
                    "in_progress" => "~",
                    _ => " ",
                };
                out.push_str(&format!(
                    "- [{}] {} (id: {}, target: {}, status: {})\n",
                    check, c.title, c.id, c.target_id, c.status
                ));
                if let Some(ref p) = c.phase {
                    out.push_str(&format!("  phase: {}\n", p));
                }
                if let Some(ref a) = c.assigned_to {
                    out.push_str(&format!("  assigned: {}\n", a));
                }
                if !c.acceptance_criteria.is_empty() {
                    out.push_str(&format!("  criteria: {}\n", c.acceptance_criteria.join(" | ")));
                }
                if let Some(ref e) = c.evidence {
                    out.push_str(&format!("  evidence: {}\n", e));
                }
            }
            out
        }
        Err(e) => format!("Error: {}", e),
    }
}
