// ─── Managed state ────────────────────────────────────────────────────────────

/// In-memory view of which server IDs Ship wrote into each provider's config.
/// Backed by the project SQLite DB (`managed_mcp_state` table).
#[derive(Debug, Default)]
struct ManagedState {
    providers: HashMap<String, ToolState>,
}

#[derive(Debug, Default, Clone)]
struct ToolState {
    managed_servers: Vec<String>,
    last_mode: Option<String>,
}

fn load_managed_state(project_dir: &Path) -> ManagedState {
    let mut state = ManagedState::default();
    for p in PROVIDERS {
        if let Ok((ids, last_mode)) = crate::state_db::get_managed_state_db(project_dir, p.id)
            && (!ids.is_empty() || last_mode.is_some())
        {
            state.providers.insert(
                p.id.to_string(),
                ToolState {
                    managed_servers: ids,
                    last_mode,
                },
            );
        }
    }
    state
}

fn save_managed_state(project_dir: &Path, state: &ManagedState) -> Result<()> {
    for (provider, tool_state) in &state.providers {
        // Non-fatal: DB writes fail gracefully when called from async context.
        let _ = crate::state_db::set_managed_state_db(
            project_dir,
            provider,
            &tool_state.managed_servers,
            tool_state.last_mode.as_deref(),
        );
    }
    Ok(())
}

// ─── Sync payload ─────────────────────────────────────────────────────────────
