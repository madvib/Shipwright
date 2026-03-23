//! Navigation model: sections with panels inside each section.

/// Whether we are on the list view or a full-page detail view.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Screen {
    #[default]
    List,
    /// Full-page detail: title + body text, with scroll offset.
    Detail {
        title: String,
        body: String,
        scroll: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Workflow,
    Docs,
    Agents,
    Events,
    Settings,
}

impl Section {
    pub const ALL: [Section; 5] = [
        Section::Workflow,
        Section::Docs,
        Section::Agents,
        Section::Events,
        Section::Settings,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Section::Workflow => "Workflow",
            Section::Docs => "Docs",
            Section::Agents => "Agents",
            Section::Events => "Events",
            Section::Settings => "Settings",
        }
    }

    pub fn key_hint(self) -> &'static str {
        match self {
            Section::Workflow => "1",
            Section::Docs => "2",
            Section::Agents => "3",
            Section::Events => "4",
            Section::Settings => "5",
        }
    }

    pub fn panels(self) -> &'static [Panel] {
        match self {
            Section::Workflow => &[Panel::Targets, Panel::Capabilities, Panel::Jobs],
            Section::Docs => &[Panel::Notes, Panel::Adrs],
            Section::Agents => &[Panel::AgentProfiles, Panel::Skills, Panel::McpServers],
            Section::Events => &[Panel::EventLog],
            Section::Settings => &[Panel::ProjectSettings],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Targets,
    Capabilities,
    Jobs,
    Notes,
    Adrs,
    AgentProfiles,
    Skills,
    McpServers,
    EventLog,
    ProjectSettings,
}

impl Panel {
    pub fn label(self) -> &'static str {
        match self {
            Panel::Targets => "Targets",
            Panel::Capabilities => "Capabilities",
            Panel::Jobs => "Jobs",
            Panel::Notes => "Notes",
            Panel::Adrs => "ADRs",
            Panel::AgentProfiles => "Profiles",
            Panel::Skills => "Skills",
            Panel::McpServers => "MCP Servers",
            Panel::EventLog => "Event Log",
            Panel::ProjectSettings => "Project",
        }
    }

    /// Whether this panel supports status filtering.
    pub fn has_status_filter(self) -> bool {
        matches!(
            self,
            Panel::Targets | Panel::Capabilities | Panel::Jobs | Panel::Adrs
        )
    }
}

/// Navigation state -- which section, panel, and screen are active.
#[derive(Debug, Default)]
pub struct NavState {
    pub section_idx: usize,
    pub panel_idx: usize,
    pub list_offset: usize,
    pub list_selected: usize,
    /// Status filter: None = show all, Some(status) = filter to that status.
    pub status_filter: Option<String>,
    pub screen: Screen,
}

/// Known statuses to cycle through per panel.
const FILTER_CYCLE: &[&str] = &[
    "pending",
    "running",
    "in_progress",
    "active",
    "done",
    "complete",
    "actual",
    "aspirational",
    "proposed",
    "accepted",
    "failed",
    "blocked",
];

impl NavState {
    pub fn section(&self) -> Section {
        Section::ALL[self.section_idx]
    }

    pub fn panel(&self) -> Panel {
        let panels = self.section().panels();
        panels[self.panel_idx.min(panels.len().saturating_sub(1))]
    }

    pub fn set_section(&mut self, idx: usize) {
        if idx < Section::ALL.len() {
            self.section_idx = idx;
            self.panel_idx = 0;
            self.reset_list();
            self.screen = Screen::List;
        }
    }

    pub fn next_section(&mut self) {
        self.set_section((self.section_idx + 1) % Section::ALL.len());
    }

    pub fn prev_section(&mut self) {
        self.set_section((self.section_idx + Section::ALL.len() - 1) % Section::ALL.len());
    }

    pub fn next_panel(&mut self) {
        let panels = self.section().panels();
        self.panel_idx = (self.panel_idx + 1) % panels.len();
        self.reset_list();
        self.screen = Screen::List;
    }

    #[allow(dead_code)]
    pub fn prev_panel(&mut self) {
        let panels = self.section().panels();
        self.panel_idx = (self.panel_idx + panels.len() - 1) % panels.len();
        self.reset_list();
        self.screen = Screen::List;
    }

    pub fn select_down(&mut self, list_len: usize) {
        if list_len > 0 && self.list_selected < list_len.saturating_sub(1) {
            self.list_selected += 1;
        }
    }

    pub fn select_up(&mut self) {
        self.list_selected = self.list_selected.saturating_sub(1);
    }

    pub fn reset_list(&mut self) {
        self.list_offset = 0;
        self.list_selected = 0;
        self.status_filter = None;
    }

    /// Cycle the status filter: None → first matching → next → … → None.
    pub fn cycle_status_filter(&mut self, active_statuses: &[&str]) {
        let relevant: Vec<&str> = FILTER_CYCLE
            .iter()
            .copied()
            .filter(|s| active_statuses.contains(s))
            .collect();
        if relevant.is_empty() {
            self.status_filter = None;
            return;
        }
        let next = match &self.status_filter {
            None => Some(relevant[0].to_string()),
            Some(current) => {
                let idx = relevant.iter().position(|s| s == current);
                match idx {
                    Some(i) if i + 1 < relevant.len() => {
                        Some(relevant[i + 1].to_string())
                    }
                    _ => None,
                }
            }
        };
        self.status_filter = next;
        self.list_selected = 0;
    }

    pub fn enter_detail(&mut self, title: String, body: String) {
        self.screen = Screen::Detail {
            title,
            body,
            scroll: 0,
        };
    }

    pub fn back_to_list(&mut self) {
        self.screen = Screen::List;
    }

    pub fn is_detail(&self) -> bool {
        matches!(self.screen, Screen::Detail { .. })
    }

    pub fn scroll_detail_down(&mut self) {
        if let Screen::Detail { ref mut scroll, .. } = self.screen {
            *scroll += 1;
        }
    }

    pub fn scroll_detail_up(&mut self) {
        if let Screen::Detail { ref mut scroll, .. } = self.screen {
            *scroll = scroll.saturating_sub(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_nav_starts_at_workflow() {
        let nav = NavState::default();
        assert_eq!(nav.section(), Section::Workflow);
        assert_eq!(nav.panel(), Panel::Targets);
        assert!(!nav.is_detail());
    }

    #[test]
    fn section_cycling_wraps() {
        let mut nav = NavState::default();
        for _ in 0..5 {
            nav.next_section();
        }
        assert_eq!(nav.section(), Section::Workflow);
        nav.prev_section();
        assert_eq!(nav.section(), Section::Settings);
    }

    #[test]
    fn panel_cycling_wraps() {
        let mut nav = NavState::default();
        assert_eq!(nav.panel(), Panel::Targets);
        nav.next_panel();
        assert_eq!(nav.panel(), Panel::Capabilities);
        nav.next_panel();
        assert_eq!(nav.panel(), Panel::Jobs);
        nav.next_panel();
        assert_eq!(nav.panel(), Panel::Targets);
    }

    #[test]
    fn set_section_resets_panel_and_list() {
        let mut nav = NavState::default();
        nav.next_panel();
        nav.list_selected = 5;
        nav.set_section(1);
        assert_eq!(nav.panel(), Panel::Notes);
        assert_eq!(nav.list_selected, 0);
    }

    #[test]
    fn status_filter_cycles() {
        let mut nav = NavState::default();
        let statuses = vec!["pending", "running", "done"];
        assert!(nav.status_filter.is_none());
        nav.cycle_status_filter(&statuses);
        assert_eq!(nav.status_filter.as_deref(), Some("pending"));
        nav.cycle_status_filter(&statuses);
        assert_eq!(nav.status_filter.as_deref(), Some("running"));
        nav.cycle_status_filter(&statuses);
        assert_eq!(nav.status_filter.as_deref(), Some("done"));
        nav.cycle_status_filter(&statuses);
        assert!(nav.status_filter.is_none());
    }

    #[test]
    fn detail_enter_and_back() {
        let mut nav = NavState::default();
        nav.enter_detail("Title".to_string(), "Body".to_string());
        assert!(nav.is_detail());
        nav.back_to_list();
        assert!(!nav.is_detail());
    }

    #[test]
    fn detail_scrolling() {
        let mut nav = NavState::default();
        nav.enter_detail("T".to_string(), "B\nB\nB".to_string());
        nav.scroll_detail_down();
        nav.scroll_detail_down();
        if let Screen::Detail { scroll, .. } = &nav.screen {
            assert_eq!(*scroll, 2);
        }
        nav.scroll_detail_up();
        if let Screen::Detail { scroll, .. } = &nav.screen {
            assert_eq!(*scroll, 1);
        }
    }

    #[test]
    fn set_section_exits_detail() {
        let mut nav = NavState::default();
        nav.enter_detail("T".to_string(), "B".to_string());
        nav.set_section(2);
        assert!(!nav.is_detail());
        assert_eq!(nav.section(), Section::Agents);
    }

    #[test]
    fn next_panel_exits_detail() {
        let mut nav = NavState::default();
        nav.enter_detail("T".to_string(), "B".to_string());
        nav.next_panel();
        assert!(!nav.is_detail());
    }
}
