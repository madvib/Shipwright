//! Ship TUI dashboard -- `ship view`.
//!
//! Section-based navigation with full-page detail views and modal forms.
//! Built on ratatui + crossterm.

mod data;
mod input;
mod modal;
mod nav;
mod render;
mod render_config;
mod render_docs;
mod render_events;
mod render_settings;
mod render_workflow;
mod theme;

use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::DefaultTerminal;

use input::Action;
use modal::{ConfirmAction, FormAction, Modal};
use nav::NavState;

/// Entry point called from `ship view`.
pub fn run_view() -> Result<()> {
    let ship_dir = resolve_ship_dir()?;

    let mut terminal = ratatui::init();
    let result = run_loop(&mut terminal, &ship_dir);
    ratatui::restore();

    result
}

fn run_loop(terminal: &mut DefaultTerminal, ship_dir: &std::path::Path) -> Result<()> {
    let mut nav = NavState::default();
    let mut view_data = data::load_all(ship_dir);
    let mut modal: Option<Modal> = None;

    loop {
        terminal.draw(|frame| render::draw(frame, &nav, &view_data, &modal))?;

        if !event::poll(Duration::from_millis(200))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        let action = if let Some(ref mut m) = modal {
            input::handle_modal(key, m)
        } else if nav.is_detail() {
            input::handle_detail(key, &mut nav)
        } else {
            input::handle_main(key, &mut nav, &view_data)
        };

        match action {
            Action::None => {}
            Action::Quit => break,
            Action::Refresh => {
                view_data = data::load_all(ship_dir);
            }
            Action::OpenModal(m) => {
                modal = Some(m);
            }
            Action::CloseModal => {
                modal = None;
            }
            Action::SubmitForm => {
                if let Some(Modal::Form(ref form)) = modal {
                    execute_form(ship_dir, form);
                    modal = None;
                    view_data = data::load_all(ship_dir);
                }
            }
            Action::ConfirmYes => {
                if let Some(Modal::Confirm { ref on_confirm, .. }) = modal {
                    execute_confirm(ship_dir, on_confirm);
                    modal = None;
                    nav.reset_list();
                    view_data = data::load_all(ship_dir);
                }
            }
        }
    }
    Ok(())
}

fn execute_form(ship_dir: &std::path::Path, form: &modal::FormState) {
    match &form.on_submit {
        FormAction::CreateNote => {
            let title = form.fields.first().map(|f| f.value.as_str()).unwrap_or("");
            let content = form.fields.get(1).map(|f| f.value.as_str()).unwrap_or("");
            let _ = data::create_note(ship_dir, title, content);
        }
        FormAction::EditNote(id) => {
            let title = form.fields.first().map(|f| f.value.as_str());
            let content = form.fields.get(1).map(|f| f.value.as_str());
            let _ = data::update_note(ship_dir, id, title, content);
        }
        FormAction::CreateAdr => {
            let title = form.fields.first().map(|f| f.value.as_str()).unwrap_or("");
            let context = form.fields.get(1).map(|f| f.value.as_str()).unwrap_or("");
            let decision = form.fields.get(2).map(|f| f.value.as_str()).unwrap_or("");
            let _ = data::create_adr(ship_dir, title, context, decision);
        }
        FormAction::EditUserPrefs => {
            let pairs: Vec<(String, String)> = data::USER_PREF_KEYS
                .iter()
                .enumerate()
                .map(|(i, (key, _))| {
                    let val = form
                        .fields
                        .get(i)
                        .map(|f| f.value.clone())
                        .unwrap_or_default();
                    (key.to_string(), val)
                })
                .collect();
            let _ = data::save_user_prefs(&pairs);
        }
    }
}

fn execute_confirm(ship_dir: &std::path::Path, action: &ConfirmAction) {
    match action {
        ConfirmAction::DeleteNote(id) => {
            let _ = data::delete_note(ship_dir, id);
        }
        ConfirmAction::DeleteAdr(id) => {
            let _ = data::delete_adr(ship_dir, id);
        }
        ConfirmAction::UpdateJobStatus { job_id, new_status } => {
            let _ = data::update_job_status(ship_dir, job_id, new_status);
        }
    }
}

fn resolve_ship_dir() -> Result<std::path::PathBuf> {
    let cwd = std::env::current_dir()?;
    let ship_dir = cwd.join(".ship");
    if !ship_dir.exists() {
        anyhow::bail!(".ship/ not found in {}. Run: ship init", cwd.display());
    }
    runtime::db::ensure_db()?;
    Ok(ship_dir)
}
