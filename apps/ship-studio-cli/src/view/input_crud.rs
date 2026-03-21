//! CRUD-specific key handlers for the TUI — text input, agent/skill/MCP/settings mutations.

use crossterm::event::KeyCode;

use super::{App, InputAction, InputMode, Screen, Tab, actions};

/// Handles all keystrokes while in text-input mode (create, delete confirm, edit).
pub fn handle_text_input(app: &mut App, code: KeyCode, action: InputAction) {
    match code {
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
            app.input_buf.clear();
            app.input_prompt.clear();
            app.status = "cancelled".into();
        }
        KeyCode::Enter => {
            let input = app.input_buf.clone();
            app.input_mode = InputMode::Normal;
            app.input_buf.clear();
            app.input_prompt.clear();
            if input.is_empty() {
                app.status = "cancelled (empty)".into();
                return;
            }
            match action {
                InputAction::CreateAgent => {
                    app.status = actions::create_agent(&input);
                    app.refresh();
                }
                InputAction::AddSkill => {
                    app.status = actions::add_skill(&input);
                    app.refresh();
                }
                InputAction::EditSetting => {
                    if let Some((key, _)) = app.settings.get(app.sel_setting) {
                        let key = key.clone();
                        app.status = actions::update_setting(&key, &input);
                        app.refresh();
                    }
                }
                InputAction::ConfirmDeleteAgent => {
                    if input == "y" || input == "Y" {
                        if let Some((id, _)) = app.agents.get(app.sel_agent) {
                            let id = id.clone();
                            app.status = actions::delete_agent(&id);
                            app.refresh();
                        }
                    } else {
                        app.status = "cancelled".into();
                    }
                }
                InputAction::ConfirmDeleteSkill => {
                    if input == "y" || input == "Y" {
                        if let Some((id, scope)) = app.skills.get(app.sel_skill) {
                            let (id, scope) = (id.clone(), scope.clone());
                            app.status = actions::delete_skill(&id, &scope);
                            app.refresh();
                        }
                    } else {
                        app.status = "cancelled".into();
                    }
                }
                InputAction::ConfirmDeleteMcp => {
                    if input == "y" || input == "Y" {
                        if let Some(s) = app.mcp_servers.get(app.sel_mcp) {
                            let id = s.id.clone();
                            app.status = actions::delete_mcp(&id);
                            app.refresh();
                        }
                    } else {
                        app.status = "cancelled".into();
                    }
                }
            }
        }
        KeyCode::Backspace => { app.input_buf.pop(); }
        KeyCode::Char(c) => { app.input_buf.push(c); }
        _ => {}
    }
}

/// Enter key — settings inline edit, otherwise delegates to `app.enter()`.
pub fn handle_enter(app: &mut App) {
    // Settings: Enter edits the value inline
    if app.tab == Tab::Settings && app.screen == Screen::List {
        if let Some((key, val)) = app.settings.get(app.sel_setting) {
            app.input_mode = InputMode::TextInput(InputAction::EditSetting);
            app.input_prompt = format!("{key} = ");
            app.input_buf = val.clone();
        }
        return;
    }
    app.enter();
}

/// `a` — activate agent, add skill, or toggle auto-refresh.
pub fn handle_activate_or_add(app: &mut App) {
    if app.screen != Screen::List { return; }
    match app.tab {
        Tab::Agents => {
            if let Some((id, _)) = app.agents.get(app.sel_agent) {
                let id = id.clone();
                app.status = actions::activate_agent(&id);
                app.refresh();
            }
        }
        Tab::Skills => {
            app.input_mode = InputMode::TextInput(InputAction::AddSkill);
            app.input_prompt = "Skill source (owner/repo or id@registry): ".into();
        }
        _ => {
            // Original 'a' behavior for other tabs: toggle auto-refresh
            app.auto_refresh = !app.auto_refresh;
            app.status = if app.auto_refresh {
                "auto-refresh on".into()
            } else {
                "auto-refresh off".into()
            };
        }
    }
}

/// `c` — start agent creation flow.
pub fn handle_create(app: &mut App) {
    if app.screen != Screen::List { return; }
    if app.tab == Tab::Agents {
        app.input_mode = InputMode::TextInput(InputAction::CreateAgent);
        app.input_prompt = "New agent name: ".into();
    }
}

/// `d` — start delete confirmation for agent, skill, or MCP server.
pub fn handle_delete(app: &mut App) {
    if app.screen != Screen::List { return; }
    match app.tab {
        Tab::Agents => {
            if let Some((id, _)) = app.agents.get(app.sel_agent) {
                app.input_mode = InputMode::TextInput(InputAction::ConfirmDeleteAgent);
                app.input_prompt = format!("Delete agent '{id}'? (y/n): ");
            }
        }
        Tab::Skills => {
            if let Some((id, _)) = app.skills.get(app.sel_skill) {
                app.input_mode = InputMode::TextInput(InputAction::ConfirmDeleteSkill);
                app.input_prompt = format!("Delete skill '{id}'? (y/n): ");
            }
        }
        Tab::Mcp => {
            if let Some(s) = app.mcp_servers.get(app.sel_mcp) {
                app.input_mode = InputMode::TextInput(InputAction::ConfirmDeleteMcp);
                app.input_prompt = format!("Delete MCP server '{}'? (y/n): ", s.id);
            }
        }
        _ => {}
    }
}
