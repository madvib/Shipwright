//! Rendering for `ship top` — draws workspace, session, agent, and event panels.

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use super::state::TopState;

const HEADER: Style = Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD);
const ACTIVE: Style = Style::new().fg(Color::Green);
const DRAIN: Style = Style::new().fg(Color::Yellow);
const DIM: Style = Style::new().fg(Color::DarkGray);

pub fn draw(f: &mut Frame, state: &TopState, tab: usize, scroll: &[usize; 4]) {
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(5),
        Constraint::Length(1),
    ])
    .split(f.area());

    draw_header(f, chunks[0], state);

    // Top half: workspaces + agents side by side. Bottom: sessions + events stacked.
    let halves =
        Layout::vertical([Constraint::Percentage(35), Constraint::Percentage(65)]).split(chunks[1]);

    let top_cols =
        Layout::horizontal([Constraint::Percentage(65), Constraint::Percentage(35)]).split(halves[0]);

    let bottom_rows =
        Layout::vertical([Constraint::Percentage(40), Constraint::Percentage(60)]).split(halves[1]);

    draw_workspaces(f, top_cols[0], state, tab == 0, scroll[0]);
    draw_agents(f, top_cols[1], state, tab == 1, scroll[1]);
    draw_sessions(f, bottom_rows[0], state, tab == 2, scroll[2]);
    draw_events(f, bottom_rows[1], state, tab == 3, scroll[3]);
    draw_footer(f, chunks[2], state);
}

fn draw_header(f: &mut Frame, area: Rect, state: &TopState) {
    let net = match &state.network_port {
        Some(p) => Span::styled(format!("net:{p}"), ACTIVE),
        None => Span::styled("net:off", Style::new().fg(Color::Red)),
    };
    let actors = Span::styled(
        format!("actors:{}", state.actor_count),
        if state.actor_count > 0 { ACTIVE } else { DIM },
    );
    let line = Line::from(vec![
        Span::styled(" ship top ", HEADER),
        Span::raw(" "),
        net,
        Span::raw("  "),
        actors,
        Span::raw("  "),
        Span::styled("Tab:panel  j/k:scroll  q:quit", DIM),
    ]);
    f.render_widget(Paragraph::new(line), area);
}

fn draw_footer(f: &mut Frame, area: Rect, state: &TopState) {
    let text = if let Some(err) = &state.error {
        Line::from(Span::styled(format!(" {err}"), Style::new().fg(Color::Red)))
    } else {
        Line::from(Span::styled(
            format!(
                " {} workspaces  {} sessions  {} agents  {} events",
                state.workspaces.len(),
                state.sessions.len(),
                state.agents.len(),
                state.events.len(),
            ),
            DIM,
        ))
    };
    f.render_widget(Paragraph::new(text), area);
}

fn draw_workspaces(f: &mut Frame, area: Rect, state: &TopState, focused: bool, scroll: usize) {
    let header =
        Row::new(["Branch", "Status", "Agent", "Providers", "Session"]).style(HEADER);
    let rows: Vec<Row> = state
        .workspaces
        .iter()
        .skip(scroll)
        .map(|w| {
            Row::new([
                Cell::from(w.branch.as_str()),
                Cell::from(w.status.as_str()).style(status_color(&w.status)),
                Cell::from(w.agent.as_str()),
                Cell::from(w.providers.as_str()),
                Cell::from(w.session_status.as_str()).style(session_color(&w.session_status)),
            ])
        })
        .collect();
    let widths = [
        Constraint::Percentage(28),
        Constraint::Percentage(12),
        Constraint::Percentage(20),
        Constraint::Percentage(25),
        Constraint::Percentage(15),
    ];
    let table = Table::new(rows, widths)
        .header(header)
        .block(panel_block("Workspaces", focused));
    f.render_widget(table, area);
}

fn draw_agents(f: &mut Frame, area: Rect, state: &TopState, focused: bool, scroll: usize) {
    let header = Row::new(["Agent", "Status", "Capabilities", "Since"]).style(HEADER);
    let rows: Vec<Row> = state
        .agents
        .iter()
        .skip(scroll)
        .map(|a| {
            Row::new([
                Cell::from(a.agent_id.as_str()),
                Cell::from(a.status.as_str()).style(agent_status_color(&a.status)),
                Cell::from(a.capabilities.as_str()).style(DIM),
                Cell::from(a.registered.as_str()).style(DIM),
            ])
        })
        .collect();
    let widths = [
        Constraint::Percentage(30),
        Constraint::Length(8),
        Constraint::Fill(1),
        Constraint::Length(9),
    ];
    let table = Table::new(rows, widths)
        .header(header)
        .block(panel_block("Mesh Agents", focused));
    f.render_widget(table, area);
}

fn draw_sessions(f: &mut Frame, area: Rect, state: &TopState, focused: bool, scroll: usize) {
    let header = Row::new(["ID", "Branch", "Status", "Provider", "Agent", "Tools", "Started", "Goal"])
        .style(HEADER);
    let rows: Vec<Row> = state
        .sessions
        .iter()
        .skip(scroll)
        .map(|s| {
            Row::new([
                Cell::from(s.id_short.as_str()).style(DIM),
                Cell::from(s.branch.as_str()),
                Cell::from(s.status.as_str()).style(session_color(&s.status)),
                Cell::from(s.provider.as_str()),
                Cell::from(s.agent.as_str()),
                Cell::from(s.tool_calls.to_string()),
                Cell::from(s.started.as_str()).style(DIM),
                Cell::from(s.goal.as_str()).style(DIM),
            ])
        })
        .collect();
    let widths = [
        Constraint::Length(9),
        Constraint::Percentage(16),
        Constraint::Length(9),
        Constraint::Percentage(12),
        Constraint::Percentage(12),
        Constraint::Length(6),
        Constraint::Length(9),
        Constraint::Fill(1),
    ];
    let table = Table::new(rows, widths)
        .header(header)
        .block(panel_block("Sessions", focused));
    f.render_widget(table, area);
}

fn draw_events(f: &mut Frame, area: Rect, state: &TopState, focused: bool, scroll: usize) {
    let header = Row::new(["Time", "Event", "Actor", "Entity"]).style(HEADER);
    let rows: Vec<Row> = state
        .events
        .iter()
        .skip(scroll)
        .map(|e| {
            Row::new([
                Cell::from(e.time.as_str()).style(DIM),
                Cell::from(e.event_type.as_str()).style(event_color(&e.event_type)),
                Cell::from(e.actor.as_str()),
                Cell::from(e.entity.as_str()).style(DIM),
            ])
        })
        .collect();
    let widths = [
        Constraint::Length(9),
        Constraint::Percentage(28),
        Constraint::Percentage(22),
        Constraint::Fill(1),
    ];
    let table = Table::new(rows, widths)
        .header(header)
        .block(panel_block("Events (live)", focused));
    f.render_widget(table, area);
}

fn panel_block(title: &str, focused: bool) -> Block<'_> {
    let style = if focused {
        Style::new().fg(Color::Cyan)
    } else {
        Style::new().fg(Color::DarkGray)
    };
    Block::default()
        .title(format!(" {title} "))
        .borders(Borders::ALL)
        .border_style(style)
}

fn status_color(s: &str) -> Style {
    match s {
        "Active" | "active" | "Compiled" | "compiled" => ACTIVE,
        _ => DIM,
    }
}

fn session_color(s: &str) -> Style {
    match s {
        "active" | "Active" => ACTIVE,
        "draining" | "Draining" => DRAIN,
        _ => DIM,
    }
}

fn agent_status_color(s: &str) -> Style {
    match s {
        "active" => ACTIVE,
        "busy" => DRAIN,
        "idle" => DIM,
        _ => DIM,
    }
}

fn event_color(t: &str) -> Style {
    if t.starts_with("session.") {
        Style::new().fg(Color::Green)
    } else if t.starts_with("workspace.") {
        Style::new().fg(Color::Blue)
    } else if t.starts_with("actor.") {
        Style::new().fg(Color::Magenta)
    } else if t.starts_with("config.") {
        Style::new().fg(Color::Yellow)
    } else if t.starts_with("mesh.") {
        Style::new().fg(Color::Cyan)
    } else if t.starts_with("gate.") {
        Style::new().fg(Color::Red)
    } else {
        Style::new().fg(Color::White)
    }
}
