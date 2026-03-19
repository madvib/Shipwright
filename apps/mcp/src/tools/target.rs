use std::path::Path;

use crate::requests::{
    CreateCapabilityRequest, CreateTargetRequest, GetTargetRequest, ListCapabilitiesRequest,
    ListTargetsRequest, MarkCapabilityActualRequest,
};

pub fn create_target(project_dir: &Path, req: CreateTargetRequest) -> String {
    let ship_dir = project_dir.join(".ship");
    match runtime::db::targets::create_target(
        &ship_dir,
        &req.kind,
        &req.title,
        req.description.as_deref(),
        req.goal.as_deref(),
        req.status.as_deref(),
    ) {
        Ok(t) => format!("Created target: {} (id: {}, kind: {})", t.title, t.id, t.kind),
        Err(e) => format!("Error creating target: {}", e),
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
        match runtime::db::targets::list_capabilities(&ship_dir, Some(&req.id), None) {
            Ok(c) => c,
            Err(e) => return format!("Error loading capabilities: {}", e),
        }
    };
    let mut out = format!(
        "# {} — {} ({})\n",
        target.kind.to_uppercase(),
        target.title,
        target.status
    );
    if let Some(ref g) = target.goal {
        out.push_str(&format!("Goal: {}\n", g));
    }
    if let Some(ref d) = target.description {
        out.push_str(&format!("{}\n", d));
    }
    let actual: Vec<_> = caps.iter().filter(|c| c.status == "actual").collect();
    let aspirational: Vec<_> = caps.iter().filter(|c| c.status == "aspirational").collect();
    if !actual.is_empty() {
        out.push_str("\n## Actual\n");
        for c in &actual {
            out.push_str(&format!("- [x] {} (id: {})\n", c.title, c.id));
        }
    }
    if !aspirational.is_empty() {
        out.push_str("\n## Aspirational\n");
        for c in &aspirational {
            out.push_str(&format!("- [ ] {} (id: {})\n", c.title, c.id));
        }
    }
    if caps.is_empty() {
        out.push_str("\nNo capabilities yet.\n");
    }
    out
}

pub fn create_capability(project_dir: &Path, req: CreateCapabilityRequest) -> String {
    let ship_dir = project_dir.join(".ship");
    match runtime::db::targets::create_capability(
        &ship_dir,
        &req.target_id,
        &req.title,
        req.milestone_id.as_deref(),
    ) {
        Ok(c) => format!("Created capability: {} (id: {})", c.title, c.id),
        Err(e) => format!("Error creating capability: {}", e),
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
        )
    };
    match result {
        Ok(cs) if cs.is_empty() => "No capabilities found.".to_string(),
        Ok(cs) => {
            let mut out = String::from("Capabilities:\n");
            for c in &cs {
                let check = if c.status == "actual" { "x" } else { " " };
                out.push_str(&format!(
                    "- [{}] {} (id: {}, target: {})\n",
                    check, c.title, c.id, c.target_id
                ));
                if let Some(ref e) = c.evidence {
                    out.push_str(&format!("  evidence: {}\n", e));
                }
            }
            out
        }
        Err(e) => format!("Error: {}", e),
    }
}
