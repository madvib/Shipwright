//! `ship view` — read-only terminal UI for browsing Ship workflow state.

mod data;
mod render;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal,
};
use ratatui::{Terminal, backend::CrosstermBackend};
use runtime::db::{jobs::Job, jobs::JobLogEntry, notes::Note, targets::{Capability, Target}};
use std::{io, path::PathBuf, time::Duration};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab { Targets, Notes, Jobs }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen { List, TargetDetail, CapDetail, NoteDetail, JobDetail }

pub struct App {
    pub tab: Tab,
    pub screen: Screen,
    pub targets: Vec<Target>,
    pub sel_target: usize,
    pub caps: Vec<Capability>,
    pub sel_cap: usize,
    pub notes: Vec<Note>,
    pub sel_note: usize,
    pub note_scroll: u16,
    pub jobs: Vec<Job>,
    pub sel_job: usize,
    pub logs: Vec<JobLogEntry>,
    pub ship_dir: PathBuf,
    pub status: String,
}

impl App {
    pub fn new(ship_dir: PathBuf) -> Self {
        let targets = data::load_targets(&ship_dir);
        let notes = data::load_notes(&ship_dir);
        let jobs = data::load_jobs(&ship_dir);
        Self {
            tab: Tab::Targets,
            screen: Screen::List,
            targets,
            sel_target: 0,
            caps: Vec::new(),
            sel_cap: 0,
            notes,
            sel_note: 0,
            note_scroll: 0,
            jobs,
            sel_job: 0,
            logs: Vec::new(),
            ship_dir,
            status: String::new(),
        }
    }

    pub fn refresh(&mut self) {
        self.targets = data::load_targets(&self.ship_dir);
        self.notes = data::load_notes(&self.ship_dir);
        self.jobs = data::load_jobs(&self.ship_dir);
        match (self.tab, self.screen) {
            (Tab::Targets, Screen::TargetDetail) | (Tab::Targets, Screen::CapDetail) => {
                if let Some(t) = self.targets.get(self.sel_target) {
                    let id = t.id.clone();
                    self.caps = data::load_caps(&self.ship_dir, &id);
                }
            }
            (Tab::Jobs, Screen::JobDetail) => {
                if let Some(j) = self.jobs.get(self.sel_job) {
                    let id = j.id.clone();
                    self.logs = data::load_logs(&self.ship_dir, &id);
                }
            }
            _ => {}
        }
        self.sel_target = self.sel_target.min(self.targets.len().saturating_sub(1));
        self.sel_note = self.sel_note.min(self.notes.len().saturating_sub(1));
        self.sel_job = self.sel_job.min(self.jobs.len().saturating_sub(1));
        self.status = "refreshed".into();
    }

    pub fn cycle_tab(&mut self) {
        if self.screen != Screen::List { return; }
        self.tab = match self.tab {
            Tab::Targets => Tab::Notes,
            Tab::Notes => Tab::Jobs,
            Tab::Jobs => Tab::Targets,
        };
        self.status.clear();
    }

    pub fn move_down(&mut self) {
        match (self.tab, self.screen) {
            (Tab::Targets, Screen::List) => {
                if !self.targets.is_empty() {
                    self.sel_target = (self.sel_target + 1).min(self.targets.len() - 1);
                }
            }
            (Tab::Targets, Screen::TargetDetail) => {
                if !self.caps.is_empty() {
                    self.sel_cap = (self.sel_cap + 1).min(self.caps.len() - 1);
                }
            }
            (Tab::Notes, Screen::List) => {
                if !self.notes.is_empty() {
                    self.sel_note = (self.sel_note + 1).min(self.notes.len() - 1);
                }
            }
            (Tab::Notes, Screen::NoteDetail) => {
                self.note_scroll = self.note_scroll.saturating_add(3);
            }
            (Tab::Jobs, Screen::List) => {
                if !self.jobs.is_empty() {
                    self.sel_job = (self.sel_job + 1).min(self.jobs.len() - 1);
                }
            }
            _ => {}
        }
    }

    pub fn move_up(&mut self) {
        match (self.tab, self.screen) {
            (Tab::Targets, Screen::List) => self.sel_target = self.sel_target.saturating_sub(1),
            (Tab::Targets, Screen::TargetDetail) => self.sel_cap = self.sel_cap.saturating_sub(1),
            (Tab::Notes, Screen::List) => self.sel_note = self.sel_note.saturating_sub(1),
            (Tab::Notes, Screen::NoteDetail) => {
                self.note_scroll = self.note_scroll.saturating_sub(3);
            }
            (Tab::Jobs, Screen::List) => self.sel_job = self.sel_job.saturating_sub(1),
            _ => {}
        }
    }

    pub fn enter(&mut self) {
        match (self.tab, self.screen) {
            (Tab::Targets, Screen::List) => {
                if let Some(t) = self.targets.get(self.sel_target) {
                    let id = t.id.clone();
                    self.caps = data::load_caps(&self.ship_dir, &id);
                    self.sel_cap = 0;
                    self.screen = Screen::TargetDetail;
                }
            }
            (Tab::Targets, Screen::TargetDetail) => {
                if !self.caps.is_empty() {
                    self.screen = Screen::CapDetail;
                }
            }
            (Tab::Notes, Screen::List) => {
                if !self.notes.is_empty() {
                    self.note_scroll = 0;
                    self.screen = Screen::NoteDetail;
                }
            }
            (Tab::Jobs, Screen::List) => {
                if let Some(j) = self.jobs.get(self.sel_job) {
                    let id = j.id.clone();
                    self.logs = data::load_logs(&self.ship_dir, &id);
                    self.screen = Screen::JobDetail;
                }
            }
            _ => {}
        }
        self.status.clear();
    }

    pub fn back(&mut self) {
        self.screen = match self.screen {
            Screen::CapDetail => Screen::TargetDetail,
            Screen::TargetDetail | Screen::NoteDetail | Screen::JobDetail => Screen::List,
            Screen::List => Screen::List,
        };
        self.status.clear();
    }
}

pub fn run_view(ship_dir: PathBuf) -> Result<()> {
    let mut app = App::new(ship_dir);
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, &mut app);

    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
    result
}

fn run_loop<B: ratatui::backend::Backend>(
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
                    KeyCode::Tab => app.cycle_tab(),
                    KeyCode::Down | KeyCode::Char('j') => app.move_down(),
                    KeyCode::Up | KeyCode::Char('k') => app.move_up(),
                    KeyCode::Enter => app.enter(),
                    KeyCode::Esc | KeyCode::Backspace => app.back(),
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
