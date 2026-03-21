//! `ship view` — interactive terminal UI for browsing and managing Ship workflow state.

pub mod actions;
mod data;
mod input;
mod render;

use anyhow::Result;
use crossterm::{execute, terminal};
use ratatui::{Terminal, backend::CrosstermBackend};
use runtime::EventRecord;
use runtime::db::{adrs::AdrRecord, jobs::Job, jobs::JobLogEntry, notes::Note, targets::{Capability, Target}};
use std::{io, path::PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab { Targets, Notes, Adrs, Jobs, Events }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen { List, TargetDetail, CapDetail, NoteDetail, AdrDetail, JobDetail, EventDetail }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobFilter { All, Pending, Running, Complete, Failed }

impl JobFilter {
    pub fn label(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Complete => "complete",
            Self::Failed => "failed",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::All => Self::Pending,
            Self::Pending => Self::Running,
            Self::Running => Self::Complete,
            Self::Complete => Self::Failed,
            Self::Failed => Self::All,
        }
    }
}

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
    pub adrs: Vec<AdrRecord>,
    pub sel_adr: usize,
    pub jobs: Vec<Job>,
    pub sel_job: usize,
    pub logs: Vec<JobLogEntry>,
    pub events: Vec<EventRecord>,
    pub sel_event: usize,
    pub log_scroll: u16,
    pub auto_refresh: bool,
    pub refresh_counter: u8,
    pub ship_dir: PathBuf,
    pub status: String,
    pub job_filter: JobFilter,
}

impl App {
    pub fn new(ship_dir: PathBuf) -> Self {
        let targets = data::load_targets(&ship_dir);
        let notes = data::load_notes(&ship_dir);
        let adrs = data::load_adrs(&ship_dir);
        let jobs = data::load_jobs_filtered(&ship_dir, None);
        let events = data::load_events(&ship_dir, 50);
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
            adrs,
            sel_adr: 0,
            jobs,
            sel_job: 0,
            logs: Vec::new(),
            events,
            sel_event: 0,
            log_scroll: 0,
            auto_refresh: true,
            refresh_counter: 0,
            ship_dir,
            status: String::new(),
            job_filter: JobFilter::All,
        }
    }

    pub fn refresh(&mut self) {
        self.targets = data::load_targets(&self.ship_dir);
        self.notes = data::load_notes(&self.ship_dir);
        self.adrs = data::load_adrs(&self.ship_dir);
        self.jobs = self.load_filtered_jobs();
        self.events = data::load_events(&self.ship_dir, 50);
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
        self.sel_adr = self.sel_adr.min(self.adrs.len().saturating_sub(1));
        self.sel_job = self.sel_job.min(self.jobs.len().saturating_sub(1));
        self.sel_event = self.sel_event.min(self.events.len().saturating_sub(1));
        self.status = "refreshed".into();
    }

    pub fn load_filtered_jobs(&self) -> Vec<Job> {
        let filter = match self.job_filter {
            JobFilter::All => None,
            f => Some(f.label()),
        };
        data::load_jobs_filtered(&self.ship_dir, filter)
    }

    pub fn cycle_tab(&mut self) {
        if self.screen != Screen::List { return; }
        self.tab = match self.tab {
            Tab::Targets => Tab::Jobs,
            Tab::Jobs => Tab::Events,
            Tab::Events => Tab::Notes,
            Tab::Notes => Tab::Adrs,
            Tab::Adrs => Tab::Targets,
        };
        self.status.clear();
    }

    pub fn reverse_cycle_tab(&mut self) {
        if self.screen != Screen::List { return; }
        self.tab = match self.tab {
            Tab::Targets => Tab::Adrs,
            Tab::Adrs => Tab::Notes,
            Tab::Notes => Tab::Events,
            Tab::Events => Tab::Jobs,
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
            (Tab::Adrs, Screen::List) => {
                if !self.adrs.is_empty() {
                    self.sel_adr = (self.sel_adr + 1).min(self.adrs.len() - 1);
                }
            }
            (Tab::Jobs, Screen::List) => {
                if !self.jobs.is_empty() {
                    self.sel_job = (self.sel_job + 1).min(self.jobs.len() - 1);
                }
            }
            (Tab::Jobs, Screen::JobDetail) => {
                self.log_scroll = self.log_scroll.saturating_add(3);
            }
            (Tab::Events, Screen::List) => {
                if !self.events.is_empty() {
                    self.sel_event = (self.sel_event + 1).min(self.events.len() - 1);
                }
            }
            (Tab::Events, Screen::EventDetail) => {}
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
            (Tab::Adrs, Screen::List) => self.sel_adr = self.sel_adr.saturating_sub(1),
            (Tab::Jobs, Screen::List) => self.sel_job = self.sel_job.saturating_sub(1),
            (Tab::Jobs, Screen::JobDetail) => {
                self.log_scroll = self.log_scroll.saturating_sub(3);
            }
            (Tab::Events, Screen::List) => self.sel_event = self.sel_event.saturating_sub(1),
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
            (Tab::Adrs, Screen::List) => {
                if !self.adrs.is_empty() {
                    self.screen = Screen::AdrDetail;
                }
            }
            (Tab::Jobs, Screen::List) => {
                if let Some(j) = self.jobs.get(self.sel_job) {
                    let id = j.id.clone();
                    self.logs = data::load_logs(&self.ship_dir, &id);
                    self.log_scroll = 0;
                    self.screen = Screen::JobDetail;
                }
            }
            (Tab::Events, Screen::List) => {
                if !self.events.is_empty() {
                    self.screen = Screen::EventDetail;
                }
            }
            _ => {}
        }
        self.status.clear();
    }

    pub fn back(&mut self) {
        self.screen = match self.screen {
            Screen::CapDetail => Screen::TargetDetail,
            Screen::TargetDetail | Screen::NoteDetail | Screen::AdrDetail | Screen::JobDetail | Screen::EventDetail => {
                Screen::List
            }
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

    let result = input::run_loop(&mut terminal, &mut app);

    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
    result
}
