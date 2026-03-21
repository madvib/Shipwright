//! TUI rendering — Ship design token palette translated to terminal RGB.

mod adrs;
mod events;
mod jobs;
mod notes;
mod targets;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
};

use crate::view::{App, Screen, Tab};

// ── Ship design tokens → terminal RGB ─────────────────────────────────────────
// Dark mode: oklch(0.18 0.01 250) bg, oklch(0.77 0.16 70) primary (amber)
pub(super) const C_BG: Color = Color::Rgb(28, 30, 38);
pub(super) const C_FG: Color = Color::Rgb(200, 196, 190);
pub(super) const C_PRI: Color = Color::Rgb(200, 156, 74); // amber — primary
pub(super) const C_MUT: Color = Color::Rgb(110, 100, 88); // muted foreground
pub(super) const C_BOR: Color = Color::Rgb(55, 58, 72);   // border
pub(super) const C_SEL: Color = Color::Rgb(44, 47, 60);   // selection bg

// Status palette — mirrors web tokens
pub(super) const C_GREEN: Color = Color::Rgb(82, 168, 112);
pub(super) const C_BLUE: Color = Color::Rgb(75, 135, 195);
pub(super) const C_RED: Color = Color::Rgb(190, 80, 60);
pub(super) const C_AMBER: Color = Color::Rgb(192, 162, 52);
pub(super) const C_PURPLE: Color = Color::Rgb(140, 96, 180);

pub(super) fn status_color(s: &str) -> Color {
    match s {
        "actual" | "complete" | "done" => C_GREEN,
        "pending" => C_BLUE,
        "aspirational" => C_PURPLE,
        "active" | "running" => C_AMBER,
        "failed" | "blocked" => C_RED,
        _ => C_MUT,
    }
}

pub(super) fn status_sym(s: &str) -> &'static str {
    match s {
        "actual" | "complete" | "done" => "●",
        "pending" => "○",
        "aspirational" => "◎",
        "active" | "running" => "◆",
        "failed" | "blocked" => "✖",
        _ => "·",
    }
}

pub(super) fn panel(title: impl Into<String>) -> Block<'static> {
    Block::default()
        .title(format!(" {} ", title.into()))
        .title_style(Style::default().fg(C_PRI).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(C_BOR))
        .style(Style::default().bg(C_BG))
}

fn header_tabs(app: &App) -> Tabs<'static> {
    let selected = match app.tab {
        Tab::Targets => 0,
        Tab::Jobs => 1,
        Tab::Events => 2,
        Tab::Notes => 3,
        Tab::Adrs => 4,
    };
    Tabs::new(vec!["  Targets  ", "  Jobs  ", "  Events  ", "  Notes  ", "  ADRs  "])
        .select(selected)
        .highlight_style(
            Style::default().fg(C_PRI).add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )
        .style(Style::default().fg(C_MUT).bg(C_BG))
        .divider(Span::styled("│", Style::default().fg(C_BOR)))
        .block(
            Block::default()
                .title(Span::styled(
                    " ◆ ship  │",
                    Style::default().fg(C_PRI).add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(C_BOR))
                .style(Style::default().bg(C_BG)),
        )
}

fn footer(app: &App) -> Paragraph<'static> {
    let auto = if app.auto_refresh { "on" } else { "off" };
    let hint = match app.screen {
        Screen::List => format!("  ↑↓ jk · ⏎ open · Tab/⇧Tab switch · r reload · a auto({auto}) · q quit"),
        _ => format!("  ↑↓ jk scroll · ⌫ Esc back · r reload · a auto({auto}) · q quit"),
    };
    let mut spans = vec![Span::styled(hint, Style::default().fg(C_MUT))];
    if !app.status.is_empty() {
        spans.push(Span::styled(
            format!("   ·  {}", app.status),
            Style::default().fg(C_GREEN),
        ));
    }
    Paragraph::new(Line::from(spans)).style(Style::default().bg(C_BG))
}

fn outer(f: &Frame) -> [Rect; 3] {
    let c = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());
    [c[0], c[1], c[2]]
}

pub fn draw(f: &mut Frame, app: &App) {
    let [hdr, body, ftr] = outer(f);
    f.render_widget(header_tabs(app), hdr);
    f.render_widget(footer(app), ftr);
    match (app.tab, app.screen) {
        (Tab::Targets, Screen::List) => targets::draw_targets(f, app, body),
        (Tab::Targets, Screen::TargetDetail) => targets::draw_target_detail(f, app, body),
        (Tab::Targets, Screen::CapDetail) => targets::draw_cap_detail(f, app, body),
        (Tab::Notes, Screen::List) => notes::draw_notes(f, app, body),
        (Tab::Notes, Screen::NoteDetail) => notes::draw_note_detail(f, app, body),
        (Tab::Adrs, Screen::List) => adrs::draw_adrs(f, app, body),
        (Tab::Adrs, Screen::AdrDetail) => adrs::draw_adr_detail(f, app, body),
        (Tab::Jobs, Screen::List) => jobs::draw_jobs(f, app, body),
        (Tab::Jobs, Screen::JobDetail) => jobs::draw_job_detail(f, app, body),
        (Tab::Events, Screen::List) => events::draw_events(f, app, body),
        (Tab::Events, Screen::EventDetail) => events::draw_event_detail(f, app, body),
        _ => {}
    }
}
