//! Key event handling. Returns actions that the main loop applies.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::data::ViewData;
use super::modal::{ConfirmAction, FormState, Modal};
use super::nav::{NavState, Panel};

/// What the main loop should do after processing a key.
pub enum Action {
    None,
    Quit,
    Refresh,
    OpenModal(Modal),
    CloseModal,
    SubmitForm,
    ConfirmYes,
}

/// Handle key events when in a detail screen (full-page).
pub fn handle_detail(key: KeyEvent, nav: &mut NavState) -> Action {
    match key.code {
        KeyCode::Esc | KeyCode::Backspace => {
            nav.back_to_list();
            Action::None
        }
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
        KeyCode::Down | KeyCode::Char('j') => {
            nav.scroll_detail_down();
            Action::None
        }
        KeyCode::Up | KeyCode::Char('k') => {
            nav.scroll_detail_up();
            Action::None
        }
        _ => Action::None,
    }
}

/// Handle key events when no modal is open and we are on list screen.
pub fn handle_main(key: KeyEvent, nav: &mut NavState, data: &ViewData) -> Action {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => return Action::Quit,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Action::Quit;
        }
        KeyCode::Char('r') => return Action::Refresh,

        // Section switching: number keys
        KeyCode::Char('1') => nav.set_section(0),
        KeyCode::Char('2') => nav.set_section(1),
        KeyCode::Char('3') => nav.set_section(2),
        KeyCode::Char('4') => nav.set_section(3),
        KeyCode::Char('5') => nav.set_section(4),

        // Section cycling
        KeyCode::Char('H') | KeyCode::BackTab => nav.prev_section(),
        KeyCode::Char('L') => nav.next_section(),

        // Panel switching within section
        KeyCode::Tab => nav.next_panel(),

        // List navigation
        KeyCode::Down | KeyCode::Char('j') => {
            let len = list_len(nav, data);
            nav.select_down(len);
        }
        KeyCode::Up | KeyCode::Char('k') => nav.select_up(),

        // Status filter cycling (any panel with status)
        KeyCode::Char('f') if nav.panel().has_status_filter() => {
            let statuses = collect_statuses(nav, data);
            nav.cycle_status_filter(&statuses);
        }

        // Enter = full-page detail for selected item
        KeyCode::Enter => {
            open_detail(nav, data);
            return Action::None;
        }

        // CRUD shortcuts
        KeyCode::Char('n') => return open_create_form(nav),
        KeyCode::Char('e') => return open_edit_form(nav, data),
        KeyCode::Char('d') => return open_delete_confirm(nav, data),

        _ => {}
    }
    Action::None
}

/// Handle key events when a modal is open.
pub fn handle_modal(key: KeyEvent, modal: &mut Modal) -> Action {
    match modal {
        Modal::Form(form) => match key.code {
            KeyCode::Esc => Action::CloseModal,
            KeyCode::Tab => {
                form.next_field();
                Action::None
            }
            KeyCode::BackTab => {
                form.prev_field();
                Action::None
            }
            KeyCode::Enter => Action::SubmitForm,
            KeyCode::Backspace => {
                form.backspace();
                Action::None
            }
            KeyCode::Char(c) => {
                form.type_char(c);
                Action::None
            }
            _ => Action::None,
        },
        Modal::Confirm { .. } => match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => Action::ConfirmYes,
            KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => Action::CloseModal,
            _ => Action::None,
        },
    }
}

fn list_len(nav: &NavState, data: &ViewData) -> usize {
    let filter = nav.status_filter.as_deref();
    match nav.panel() {
        Panel::Targets => count_filtered(data.targets.iter().map(|t| t.status.as_str()), filter),
        Panel::Capabilities => {
            count_filtered(data.capabilities.iter().map(|c| c.status.as_str()), filter)
        }
        Panel::Jobs => count_filtered(data.all_jobs.iter().map(|j| j.status.as_str()), filter),
        Panel::Notes => data.notes.len(),
        Panel::Adrs => count_filtered(data.adrs.iter().map(|a| a.status.as_str()), filter),
        Panel::EventLog => data.events.len(),
        Panel::AgentProfiles => super::render_config::discover_agents().len(),
        Panel::Skills => super::render_config::discover_all_skills().len(),
        _ => 0,
    }
}

fn count_filtered<'a>(statuses: impl Iterator<Item = &'a str>, filter: Option<&str>) -> usize {
    match filter {
        None => statuses.count(),
        Some(f) => statuses.filter(|s| *s == f).count(),
    }
}

fn collect_statuses<'a>(nav: &NavState, data: &'a ViewData) -> Vec<&'a str> {
    match nav.panel() {
        Panel::Targets => data.targets.iter().map(|t| t.status.as_str()).collect(),
        Panel::Capabilities => data
            .capabilities
            .iter()
            .map(|c| c.status.as_str())
            .collect(),
        Panel::Jobs => data.all_jobs.iter().map(|j| j.status.as_str()).collect(),
        Panel::Adrs => data.adrs.iter().map(|a| a.status.as_str()).collect(),
        _ => vec![],
    }
}

fn truncate_id(id: &str) -> &str {
    &id[..8.min(id.len())]
}

/// Open a full-page detail view by setting nav.screen.
fn open_detail(nav: &mut NavState, data: &ViewData) {
    let filter = nav.status_filter.as_deref();
    match nav.panel() {
        Panel::Notes => {
            if let Some(n) = data.notes.get(nav.list_selected) {
                nav.enter_detail(n.title.clone(), n.content.clone());
            }
        }
        Panel::Adrs => {
            let adrs: Vec<&_> = data
                .adrs
                .iter()
                .filter(|a| filter.is_none() || filter == Some(a.status.as_str()))
                .collect();
            if let Some(a) = adrs.get(nav.list_selected) {
                let body = format!(
                    "Status: {}\nDate: {}\n\n## Context\n{}\n\n## Decision\n{}",
                    a.status, a.date, a.context, a.decision
                );
                nav.enter_detail(a.title.clone(), body);
            }
        }
        Panel::Jobs => {
            let jobs: Vec<&_> = data
                .all_jobs
                .iter()
                .filter(|j| filter.is_none() || filter == Some(j.status.as_str()))
                .collect();
            if let Some(j) = jobs.get(nav.list_selected) {
                let body = format!(
                    "**Kind:** {}\n**Status:** {}\n**Branch:** {}\n**Created by:** {}\n**Claimed by:** {}\n**Created:** {}\n**Updated:** {}\n\n## Payload\n```json\n{}\n```",
                    j.kind,
                    j.status,
                    j.branch.as_deref().unwrap_or("-"),
                    j.created_by.as_deref().unwrap_or("-"),
                    j.claimed_by.as_deref().unwrap_or("-"),
                    j.created_at,
                    j.updated_at,
                    serde_json::to_string_pretty(&j.payload).unwrap_or_default(),
                );
                nav.enter_detail(format!("Job {}", j.id), body);
            }
        }
        Panel::Targets => {
            let targets: Vec<&_> = data
                .targets
                .iter()
                .filter(|t| filter.is_none() || filter == Some(t.status.as_str()))
                .collect();
            if let Some(t) = targets.get(nav.list_selected) {
                let body = format!(
                    "**Kind:** {}\n**Status:** {}\n**Goal:** {}\n\n{}",
                    t.kind,
                    t.status,
                    t.goal.as_deref().unwrap_or("-"),
                    t.description.as_deref().unwrap_or(""),
                );
                nav.enter_detail(t.title.clone(), body);
            }
        }
        Panel::Capabilities => {
            let caps: Vec<&_> = data
                .capabilities
                .iter()
                .filter(|c| filter.is_none() || filter == Some(c.status.as_str()))
                .collect();
            if let Some(c) = caps.get(nav.list_selected) {
                let target_name = data
                    .targets
                    .iter()
                    .find(|t| t.id == c.target_id)
                    .map(|t| format!("{} ({})", t.title, truncate_id(&c.target_id)))
                    .unwrap_or_else(|| c.target_id.clone());
                let milestone = c
                    .milestone_id
                    .as_ref()
                    .map(|mid| {
                        data.targets
                            .iter()
                            .find(|t| t.id == *mid)
                            .map(|t| format!("{} ({})", t.title, truncate_id(mid)))
                            .unwrap_or_else(|| mid.clone())
                    })
                    .unwrap_or_else(|| "-".to_string());
                let body = format!(
                    "**Target:** {target_name}\n**Status:** {}\n**Evidence:** {}\n**Milestone:** {milestone}",
                    c.status,
                    c.evidence.as_deref().unwrap_or("-"),
                );
                nav.enter_detail(c.title.clone(), body);
            }
        }
        Panel::EventLog => {
            if let Some(e) = data.events.get(nav.list_selected) {
                let body = format!(
                    "**ID:** {}\n**Timestamp:** {}\n**Actor:** {}\n**Entity:** {:?}\n**Action:** {:?}\n**Subject:** {}\n\n{}",
                    e.id,
                    e.timestamp,
                    e.actor,
                    e.entity,
                    e.action,
                    e.subject,
                    e.details.as_deref().unwrap_or(""),
                );
                nav.enter_detail(format!("Event {}", &e.id[..8.min(e.id.len())]), body);
            }
        }
        Panel::AgentProfiles => {
            let agents = super::render_config::discover_agents();
            if let Some(name) = agents.get(nav.list_selected) {
                super::render_config::open_agent_detail(nav, name);
            }
        }
        Panel::Skills => {
            let skills = super::render_config::discover_all_skills();
            if let Some(entry) = skills.get(nav.list_selected) {
                super::render_config::open_skill_detail_by_entry(nav, entry);
            }
        }
        _ => {}
    }
}

fn open_create_form(nav: &NavState) -> Action {
    match nav.panel() {
        Panel::Notes => Action::OpenModal(Modal::Form(FormState::new_note())),
        Panel::Adrs => Action::OpenModal(Modal::Form(FormState::new_adr())),
        _ => Action::None,
    }
}

fn open_edit_form(nav: &NavState, data: &ViewData) -> Action {
    match nav.panel() {
        Panel::ProjectSettings => Action::OpenModal(Modal::Form(FormState::edit_user_prefs(
            &data.config.user_prefs,
        ))),
        Panel::Notes => {
            if let Some(n) = data.notes.get(nav.list_selected) {
                Action::OpenModal(Modal::Form(FormState::edit_note(
                    &n.id, &n.title, &n.content,
                )))
            } else {
                Action::None
            }
        }
        _ => Action::None,
    }
}

fn open_delete_confirm(nav: &NavState, data: &ViewData) -> Action {
    match nav.panel() {
        Panel::Notes => {
            if let Some(n) = data.notes.get(nav.list_selected) {
                Action::OpenModal(Modal::Confirm {
                    title: "Delete Note".to_string(),
                    message: format!("Delete note '{}'?", n.title),
                    on_confirm: ConfirmAction::DeleteNote(n.id.clone()),
                })
            } else {
                Action::None
            }
        }
        Panel::Adrs => {
            if let Some(a) = data.adrs.get(nav.list_selected) {
                Action::OpenModal(Modal::Confirm {
                    title: "Delete ADR".to_string(),
                    message: format!("Delete ADR '{}'?", a.title),
                    on_confirm: ConfirmAction::DeleteAdr(a.id.clone()),
                })
            } else {
                Action::None
            }
        }
        _ => Action::None,
    }
}
