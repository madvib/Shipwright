//! App navigation — tab cycling, list movement, enter/back.

use super::{App, Screen, Tab, data};

impl App {
    pub fn cycle_tab(&mut self) {
        if self.screen != Screen::List { return; }
        self.tab = match self.tab {
            Tab::Targets => Tab::Jobs,
            Tab::Jobs => Tab::Events,
            Tab::Events => Tab::Notes,
            Tab::Notes => Tab::Adrs,
            Tab::Adrs => Tab::Agents,
            Tab::Agents => Tab::Skills,
            Tab::Skills => Tab::Mcp,
            Tab::Mcp => Tab::Settings,
            Tab::Settings => Tab::Targets,
        };
        self.status.clear();
    }

    pub fn reverse_cycle_tab(&mut self) {
        if self.screen != Screen::List { return; }
        self.tab = match self.tab {
            Tab::Targets => Tab::Settings,
            Tab::Settings => Tab::Mcp,
            Tab::Mcp => Tab::Skills,
            Tab::Skills => Tab::Agents,
            Tab::Agents => Tab::Adrs,
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
            (Tab::Agents, Screen::List) => {
                if !self.agents.is_empty() {
                    self.sel_agent = (self.sel_agent + 1).min(self.agents.len() - 1);
                }
            }
            (Tab::Skills, Screen::List) => {
                if !self.skills.is_empty() {
                    self.sel_skill = (self.sel_skill + 1).min(self.skills.len() - 1);
                }
            }
            (Tab::Mcp, Screen::List) => {
                if !self.mcp_servers.is_empty() {
                    self.sel_mcp = (self.sel_mcp + 1).min(self.mcp_servers.len() - 1);
                }
            }
            (Tab::Settings, Screen::List) => {
                if !self.settings.is_empty() {
                    self.sel_setting = (self.sel_setting + 1).min(self.settings.len() - 1);
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
            (Tab::Adrs, Screen::List) => self.sel_adr = self.sel_adr.saturating_sub(1),
            (Tab::Jobs, Screen::List) => self.sel_job = self.sel_job.saturating_sub(1),
            (Tab::Jobs, Screen::JobDetail) => {
                self.log_scroll = self.log_scroll.saturating_sub(3);
            }
            (Tab::Events, Screen::List) => self.sel_event = self.sel_event.saturating_sub(1),
            (Tab::Agents, Screen::List) => self.sel_agent = self.sel_agent.saturating_sub(1),
            (Tab::Skills, Screen::List) => self.sel_skill = self.sel_skill.saturating_sub(1),
            (Tab::Mcp, Screen::List) => self.sel_mcp = self.sel_mcp.saturating_sub(1),
            (Tab::Settings, Screen::List) => self.sel_setting = self.sel_setting.saturating_sub(1),
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
            (Tab::Agents, Screen::List) => {
                if let Some((id, _)) = self.agents.get(self.sel_agent) {
                    self.agent_detail_text = data::load_agent_detail(id);
                    self.screen = Screen::AgentDetail;
                }
            }
            (Tab::Mcp, Screen::List) => {
                if !self.mcp_servers.is_empty() {
                    self.screen = Screen::McpDetail;
                }
            }
            _ => {}
        }
        self.status.clear();
    }

    pub fn back(&mut self) {
        self.screen = match self.screen {
            Screen::CapDetail => Screen::TargetDetail,
            Screen::TargetDetail | Screen::NoteDetail | Screen::AdrDetail
            | Screen::JobDetail | Screen::EventDetail
            | Screen::AgentDetail | Screen::McpDetail => Screen::List,
            Screen::List => Screen::List,
        };
        self.status.clear();
    }
}
