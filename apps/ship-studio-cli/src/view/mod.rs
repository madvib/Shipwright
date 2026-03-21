//! `ship view` — interactive terminal UI for browsing and managing Ship workflow state.

pub mod actions;
mod data;
mod input;
mod input_crud;
mod nav;
mod render;

use anyhow::Result;
use crossterm::{execute, terminal};
use ratatui::{Terminal, backend::CrosstermBackend};
use runtime::EventRecord;
use runtime::db::{adrs::AdrRecord, jobs::Job, jobs::JobLogEntry, notes::Note, targets::{Capability, Target}};
use std::{io, path::PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab { Targets, Notes, Adrs, Jobs, Events, Agents, Skills, Mcp, Settings }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen { List, TargetDetail, CapDetail, NoteDetail, AdrDetail, JobDetail, EventDetail, AgentDetail, McpDetail }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode { Normal, TextInput(InputAction) }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputAction {
    CreateAgent,
    AddSkill,
    EditSetting,
    ConfirmDeleteAgent,
    ConfirmDeleteSkill,
    ConfirmDeleteMcp,
}

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
    // Agents tab
    pub agents: Vec<(String, String)>,
    pub sel_agent: usize,
    pub active_agent: Option<String>,
    pub compiled_at: Option<String>,
    pub agent_detail_text: String,
    // Skills tab
    pub skills: Vec<(String, String)>,
    pub sel_skill: usize,
    // MCP tab
    pub mcp_servers: Vec<crate::mcp::McpEntry>,
    pub sel_mcp: usize,
    // Settings tab
    pub settings: Vec<(String, String)>,
    pub sel_setting: usize,
    // Input mode
    pub input_mode: InputMode,
    pub input_buf: String,
    pub input_prompt: String,
    // General
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
        let agents = data::load_agents();
        let (active_agent, compiled_at) = data::load_workspace_state(&ship_dir);
        let skills = data::load_skills();
        let mcp_servers = data::load_mcp_servers();
        let settings = data::load_settings();
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
            agents,
            sel_agent: 0,
            active_agent,
            compiled_at,
            agent_detail_text: String::new(),
            skills,
            sel_skill: 0,
            mcp_servers,
            sel_mcp: 0,
            settings,
            sel_setting: 0,
            input_mode: InputMode::Normal,
            input_buf: String::new(),
            input_prompt: String::new(),
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
        self.agents = data::load_agents();
        let (active_agent, compiled_at) = data::load_workspace_state(&self.ship_dir);
        self.active_agent = active_agent;
        self.compiled_at = compiled_at;
        self.skills = data::load_skills();
        self.mcp_servers = data::load_mcp_servers();
        self.settings = data::load_settings();
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
        self.sel_agent = self.sel_agent.min(self.agents.len().saturating_sub(1));
        self.sel_skill = self.sel_skill.min(self.skills.len().saturating_sub(1));
        self.sel_mcp = self.sel_mcp.min(self.mcp_servers.len().saturating_sub(1));
        self.sel_setting = self.sel_setting.min(self.settings.len().saturating_sub(1));
        self.status = "refreshed".into();
    }

    pub fn load_filtered_jobs(&self) -> Vec<Job> {
        let filter = match self.job_filter {
            JobFilter::All => None,
            f => Some(f.label()),
        };
        data::load_jobs_filtered(&self.ship_dir, filter)
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
