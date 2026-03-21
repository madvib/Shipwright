//! TUI mutation helpers — thin wrappers around runtime DB calls.
//!
//! All functions are infallible: they return a status message string
//! suitable for display in the TUI footer.

use runtime::db::{
    jobs::update_job_status,
    targets::{update_capability, CapabilityPatch},
};
use std::path::Path;

/// Cycle a job through: pending → running → complete → failed → pending.
pub fn cycle_job_status(ship_dir: &Path, job_id: &str, current: &str) -> String {
    let next = match current {
        "pending" => "running",
        "running" => "complete",
        "complete" => "failed",
        "failed" => "pending",
        _ => "pending",
    };
    match update_job_status(ship_dir, job_id, next) {
        Ok(()) => format!("job → {next}"),
        Err(e) => format!("error: {e}"),
    }
}

/// Cycle a capability through: aspirational → in_progress → actual → aspirational.
pub fn cycle_cap_status(ship_dir: &Path, cap_id: &str, current: &str) -> String {
    let next = match current {
        "aspirational" => "in_progress",
        "in_progress" => "actual",
        "actual" => "aspirational",
        _ => "aspirational",
    };
    let patch = CapabilityPatch {
        status: Some(next.to_string()),
        title: None,
        phase: None,
        acceptance_criteria: None,
        preset_hint: None,
        file_scope: None,
        assigned_to: None,
        priority: None,
    };
    match update_capability(ship_dir, cap_id, patch) {
        Ok(()) => format!("cap → {next}"),
        Err(e) => format!("error: {e}"),
    }
}

// ── CRUD actions for new tabs ─────────────────────────────────────────────────

pub fn activate_agent(agent_id: &str) -> String {
    let project_root = std::path::Path::new(".");
    match crate::profile::activate_agent(Some(agent_id), project_root) {
        Ok(()) => format!("activated '{agent_id}'"),
        Err(e) => format!("error: {e}"),
    }
}

pub fn delete_agent(agent_id: &str) -> String {
    let project_root = std::path::Path::new(".");
    match crate::profile::find_agent_file(agent_id, project_root) {
        Some(path) => match std::fs::remove_file(&path) {
            Ok(()) => format!("deleted '{agent_id}'"),
            Err(e) => format!("error: {e}"),
        },
        None => format!("agent '{agent_id}' not found"),
    }
}

pub fn create_agent(name: &str) -> String {
    let dir = crate::paths::agents_dir();
    let path = dir.join(format!("{name}.jsonc"));
    if path.exists() {
        return format!("agent '{name}' already exists");
    }
    let content = crate::agent_config::AgentConfig::scaffold_jsonc(name);
    if let Err(e) = std::fs::create_dir_all(&dir) {
        return format!("error: {e}");
    }
    match std::fs::write(&path, content) {
        Ok(()) => format!("created '{name}'"),
        Err(e) => format!("error: {e}"),
    }
}

pub fn delete_skill(skill_id: &str, scope: &str) -> String {
    let global = scope == "global";
    match crate::skill::remove(skill_id, global) {
        Ok(()) => format!("removed skill '{skill_id}'"),
        Err(e) => format!("error: {e}"),
    }
}

pub fn add_skill(source: &str) -> String {
    match crate::skill::add(source, None, false) {
        Ok(()) => format!("added skill from '{source}'"),
        Err(e) => format!("error: {e}"),
    }
}

pub fn delete_mcp(server_id: &str) -> String {
    match crate::mcp::remove(server_id) {
        Ok(()) => format!("removed MCP '{server_id}'"),
        Err(e) => format!("error: {e}"),
    }
}

pub fn update_setting(key: &str, value: &str) -> String {
    let mut cfg = crate::config::ShipConfig::load();
    if let Err(e) = cfg.set(key, value) {
        return format!("error: {e}");
    }
    match cfg.save() {
        Ok(()) => format!("set {key} = {value}"),
        Err(e) => format!("error saving: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use runtime::db::{ensure_db, jobs::{create_job, get_job}};
    use runtime::db::targets::{create_target, create_capability, get_capability};
    use runtime::project::init_project;
    use tempfile::TempDir;

    fn setup() -> (TempDir, std::path::PathBuf) {
        let tmp = TempDir::new().unwrap();
        let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
        ensure_db(&ship_dir).unwrap();
        (tmp, ship_dir)
    }

    #[test]
    fn status_cycling_job_and_capability() {
        let (_tmp, ship_dir) = setup();

        // ── Job: full cycle pending → running → complete → failed → pending ──
        let job = create_job(
            &ship_dir, "test", None, None, None, None, 0, None,
            vec![], vec![],
        ).unwrap();
        assert_eq!(job.status, "pending");

        let transitions = [
            ("pending", "running"),
            ("running", "complete"),
            ("complete", "failed"),
            ("failed", "pending"),
        ];
        let mut current = job.status.clone();
        for (from, to) in &transitions {
            assert_eq!(&current, from);
            let msg = cycle_job_status(&ship_dir, &job.id, &current);
            assert_eq!(msg, format!("job → {to}"));
            current = get_job(&ship_dir, &job.id).unwrap().unwrap().status;
            assert_eq!(&current, to);
        }

        // ── Capability: aspirational → in_progress → actual → aspirational ──
        let target = create_target(&ship_dir, "milestone", "Test Target", None, None, None)
            .unwrap();
        let cap = create_capability(&ship_dir, &target.id, "Test Cap", None)
            .unwrap();
        assert_eq!(cap.status, "aspirational");

        let cap_transitions = [
            ("aspirational", "in_progress"),
            ("in_progress", "actual"),
            ("actual", "aspirational"),
        ];
        let mut cap_status = cap.status.clone();
        for (from, to) in &cap_transitions {
            assert_eq!(&cap_status, from);
            let msg = cycle_cap_status(&ship_dir, &cap.id, &cap_status);
            assert_eq!(msg, format!("cap → {to}"));
            cap_status = get_capability(&ship_dir, &cap.id).unwrap().unwrap().status;
            assert_eq!(&cap_status, to);
        }
    }

    // ── CRUD action tests ────────────────────────────────────────────────────

    /// Helper: run a closure with CWD set to a temp directory, then restore.
    fn with_tmp_cwd<F: FnOnce(&std::path::Path)>(f: F) {
        let tmp = TempDir::new().unwrap();
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();
        f(tmp.path());
        std::env::set_current_dir(original).unwrap();
    }

    #[test]
    fn create_agent_writes_jsonc_scaffold() {
        with_tmp_cwd(|tmp| {
            let msg = create_agent("my-bot");
            assert_eq!(msg, "created 'my-bot'");

            let path = tmp.join(".ship/agents/my-bot.jsonc");
            assert!(path.exists(), "agent file should be created");

            let content = std::fs::read_to_string(&path).unwrap();
            assert!(content.contains("\"name\": \"my-bot\""));
            assert!(content.contains("\"id\": \"my-bot\""));
            assert!(content.contains("\"providers\""));
            assert!(!content.contains("[tools]"), "must not contain invalid TOML field");
        });
    }

    #[test]
    fn create_agent_rejects_duplicate() {
        with_tmp_cwd(|tmp| {
            let dir = tmp.join(".ship/agents");
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("dup.jsonc"), "{}").unwrap();

            let msg = create_agent("dup");
            assert_eq!(msg, "agent 'dup' already exists");
        });
    }

    #[test]
    fn delete_agent_removes_file() {
        with_tmp_cwd(|tmp| {
            let dir = tmp.join(".ship/agents");
            std::fs::create_dir_all(&dir).unwrap();
            let path = dir.join("doomed.jsonc");
            std::fs::write(&path, "{}").unwrap();
            assert!(path.exists());

            let msg = delete_agent("doomed");
            assert_eq!(msg, "deleted 'doomed'");
            assert!(!path.exists(), "file should be removed");
        });
    }

    #[test]
    fn delete_agent_not_found() {
        with_tmp_cwd(|_tmp| {
            let msg = delete_agent("ghost");
            assert_eq!(msg, "agent 'ghost' not found");
        });
    }

    #[test]
    fn activate_agent_fails_without_file() {
        with_tmp_cwd(|tmp| {
            // init .ship so activate_agent can find the dir
            std::fs::create_dir_all(tmp.join(".ship/agents")).unwrap();
            let msg = activate_agent("nonexistent");
            assert!(msg.starts_with("error:"), "expected error, got: {msg}");
        });
    }

    #[test]
    fn delete_skill_error_on_missing() {
        with_tmp_cwd(|_tmp| {
            let msg = delete_skill("no-such-skill", "local");
            assert!(msg.starts_with("error:"), "expected error, got: {msg}");
        });
    }

    #[test]
    fn delete_mcp_error_on_missing() {
        with_tmp_cwd(|_tmp| {
            let msg = delete_mcp("no-such-server");
            assert!(msg.starts_with("error:"), "expected error, got: {msg}");
        });
    }

    #[test]
    fn update_setting_rejects_unknown_key() {
        let msg = update_setting("bogus.key", "value");
        assert!(msg.starts_with("error:"), "expected error, got: {msg}");
        assert!(msg.contains("Unknown key"));
    }
}
