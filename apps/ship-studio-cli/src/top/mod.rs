//! `ship top` — real-time TUI dashboard for Ship runtime state.

mod state;
mod ui;

use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{DefaultTerminal, prelude::CrosstermBackend};

const TICK_RATE: Duration = Duration::from_secs(2);
const PANEL_COUNT: usize = 4;

struct App {
    ship_dir: std::path::PathBuf,
    state: state::TopState,
    tab: usize,
    scroll: [usize; PANEL_COUNT],
    quit: bool,
}

impl App {
    fn new(ship_dir: std::path::PathBuf) -> Self {
        let mut state = state::TopState::empty();
        state::refresh(&mut state, &ship_dir);
        Self {
            ship_dir,
            state,
            tab: 0,
            scroll: [0; PANEL_COUNT],
            quit: false,
        }
    }

    fn tick(&mut self) {
        state::refresh(&mut self.state, &self.ship_dir);
    }

    fn panel_len(&self) -> usize {
        match self.tab {
            0 => self.state.workspaces.len(),
            1 => self.state.agents.len(),
            2 => self.state.sessions.len(),
            _ => self.state.events.len(),
        }
    }

    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        match code {
            KeyCode::Char('q') => self.quit = true,
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => self.quit = true,
            KeyCode::Tab => self.tab = (self.tab + 1) % PANEL_COUNT,
            KeyCode::BackTab => self.tab = (self.tab + PANEL_COUNT - 1) % PANEL_COUNT,
            KeyCode::Char('j') | KeyCode::Down => {
                self.scroll[self.tab] = self.scroll[self.tab].saturating_add(1);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.scroll[self.tab] = self.scroll[self.tab].saturating_sub(1);
            }
            KeyCode::Char('g') | KeyCode::Home => {
                self.scroll[self.tab] = 0;
            }
            KeyCode::Char('G') | KeyCode::End => {
                self.scroll[self.tab] = self.panel_len().saturating_sub(1);
            }
            _ => {}
        }
    }
}

pub fn run(ship_dir: std::path::PathBuf) -> Result<()> {
    if !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
        anyhow::bail!("ship top requires an interactive terminal");
    }
    enable_raw_mode()?;
    crossterm::execute!(std::io::stdout(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = ratatui::Terminal::new(backend)?;

    let result = run_loop(&mut terminal, ship_dir);

    disable_raw_mode()?;
    crossterm::execute!(std::io::stdout(), LeaveAlternateScreen)?;
    result
}

fn run_loop(terminal: &mut DefaultTerminal, ship_dir: std::path::PathBuf) -> Result<()> {
    let mut app = App::new(ship_dir);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui::draw(f, &app.state, app.tab, &app.scroll))?;

        let timeout = TICK_RATE
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::ZERO);

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    app.handle_key(key.code, key.modifiers);
                }
            }
        }

        if last_tick.elapsed() >= TICK_RATE {
            app.tick();
            last_tick = Instant::now();
        }

        if app.quit {
            return Ok(());
        }
    }
}
