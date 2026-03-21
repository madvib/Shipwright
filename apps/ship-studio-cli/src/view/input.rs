//! Key handling and event loop for the TUI.

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::Terminal;
use std::time::Duration;

use super::{App, Screen, Tab, actions, render};

/// Main event loop — polls keyboard at 250ms intervals.
pub fn run_loop<B: ratatui::backend::Backend + std::io::Write>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| render::draw(f, app))?;
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    KeyCode::Char('r') => app.refresh(),
                    KeyCode::Char('a') => {
                        app.auto_refresh = !app.auto_refresh;
                        app.status = if app.auto_refresh {
                            "auto-refresh on".into()
                        } else {
                            "auto-refresh off".into()
                        };
                    }
                    KeyCode::Tab => app.cycle_tab(),
                    KeyCode::BackTab => app.reverse_cycle_tab(),
                    KeyCode::Down | KeyCode::Char('j') => app.move_down(),
                    KeyCode::Up | KeyCode::Char('k') => app.move_up(),
                    KeyCode::Enter => app.enter(),
                    KeyCode::Esc | KeyCode::Backspace => app.back(),
                    KeyCode::Char('s') => handle_status_cycle(app),
                    KeyCode::Char('f') => handle_filter(app),
                    KeyCode::Char('g') => handle_jump_top(app),
                    KeyCode::Char('G') => handle_jump_bottom(app),
                    KeyCode::Char('l') => handle_launch(terminal, app)?,
                    _ => {}
                }
            }
        } else if app.auto_refresh {
            app.refresh_counter += 1;
            if app.refresh_counter >= 20 {
                app.refresh_counter = 0;
                app.refresh();
                app.status = "auto".into();
            }
        }
    }
    Ok(())
}

/// `s` — cycle status on Jobs list or TargetDetail capability list.
fn handle_status_cycle(app: &mut App) {
    match (app.tab, app.screen) {
        (Tab::Jobs, Screen::List) => {
            if let Some(j) = app.jobs.get(app.sel_job) {
                let id = j.id.clone();
                let current = j.status.clone();
                app.status = actions::cycle_job_status(&app.ship_dir, &id, &current);
                app.refresh();
            }
        }
        (Tab::Targets, Screen::TargetDetail) => {
            if let Some(c) = app.caps.get(app.sel_cap) {
                let id = c.id.clone();
                let current = c.status.clone();
                app.status = actions::cycle_cap_status(&app.ship_dir, &id, &current);
                app.refresh();
            }
        }
        _ => {}
    }
}

/// `f` — cycle job filter (only on Jobs list).
fn handle_filter(app: &mut App) {
    if app.tab == Tab::Jobs && app.screen == Screen::List {
        app.job_filter = app.job_filter.next();
        app.jobs = app.load_filtered_jobs();
        app.sel_job = 0;
        app.status = format!("filter: {}", app.job_filter.label());
    }
}

/// `g` — jump to top of current list or scroll position.
fn handle_jump_top(app: &mut App) {
    match (app.tab, app.screen) {
        (_, Screen::List) => match app.tab {
            Tab::Targets => app.sel_target = 0,
            Tab::Jobs => app.sel_job = 0,
            Tab::Events => app.sel_event = 0,
            Tab::Notes => app.sel_note = 0,
            Tab::Adrs => app.sel_adr = 0,
        },
        (Tab::Targets, Screen::TargetDetail) => app.sel_cap = 0,
        (Tab::Notes, Screen::NoteDetail) => app.note_scroll = 0,
        (Tab::Jobs, Screen::JobDetail) => app.log_scroll = 0,
        _ => {}
    }
}

/// `G` — jump to bottom of current list or scroll position.
fn handle_jump_bottom(app: &mut App) {
    match (app.tab, app.screen) {
        (_, Screen::List) => match app.tab {
            Tab::Targets => app.sel_target = app.targets.len().saturating_sub(1),
            Tab::Jobs => app.sel_job = app.jobs.len().saturating_sub(1),
            Tab::Events => app.sel_event = app.events.len().saturating_sub(1),
            Tab::Notes => app.sel_note = app.notes.len().saturating_sub(1),
            Tab::Adrs => app.sel_adr = app.adrs.len().saturating_sub(1),
        },
        (Tab::Targets, Screen::TargetDetail) => {
            app.sel_cap = app.caps.len().saturating_sub(1);
        }
        (Tab::Notes, Screen::NoteDetail) => app.note_scroll = u16::MAX,
        (Tab::Jobs, Screen::JobDetail) => app.log_scroll = u16::MAX,
        _ => {}
    }
}

/// `l` — launch session from JobDetail. Suspends TUI, runs `ship use <agent>`.
fn handle_launch<B: ratatui::backend::Backend + std::io::Write>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    if app.tab != Tab::Jobs || app.screen != Screen::JobDetail {
        return Ok(());
    }
    let Some(j) = app.jobs.get(app.sel_job) else {
        return Ok(());
    };
    let Some(ref agent) = j.claimed_by else {
        app.status = "no agent claimed this job".into();
        return Ok(());
    };
    let agent = agent.clone();

    // Suspend TUI
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), crossterm::terminal::LeaveAlternateScreen)?;

    let exe = std::env::current_exe().unwrap_or_else(|_| "ship".into());
    let result = std::process::Command::new(&exe)
        .args(["use", &agent])
        .status();

    // Resume TUI
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), crossterm::terminal::EnterAlternateScreen)?;
    terminal.clear()?;

    app.status = match result {
        Ok(s) if s.success() => format!("session ended ({agent})"),
        Ok(s) => format!("session exited: {s}"),
        Err(e) => format!("launch error: {e}"),
    };
    app.refresh();
    Ok(())
}
