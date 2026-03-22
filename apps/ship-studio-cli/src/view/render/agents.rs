use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph, Wrap},
};

use super::{C_BG, C_FG, C_GREEN, C_MUT, C_SEL, panel};
use crate::view::App;

pub fn draw_agents(f: &mut Frame, app: &App, area: Rect) {
    if app.agents.is_empty() {
        f.render_widget(
            Paragraph::new("  No agents found.  c create")
                .style(Style::default().fg(C_MUT).bg(C_BG))
                .block(panel("Agents")),
            area,
        );
        return;
    }
    let items: Vec<ListItem> = app
        .agents
        .iter()
        .map(|(id, scope)| {
            let is_active = app.active_agent.as_deref() == Some(id.as_str());
            let marker = if is_active { "●" } else { " " };
            let marker_color = if is_active { C_GREEN } else { C_MUT };
            let line = Line::from(vec![
                Span::styled(format!(" {marker} "), Style::default().fg(marker_color)),
                Span::styled(
                    format!("{:<30}", id),
                    Style::default().fg(C_FG).add_modifier(if is_active {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
                ),
                Span::styled(format!("  {scope}"), Style::default().fg(C_MUT)),
            ]);
            ListItem::new(line)
        })
        .collect();
    let mut state = ListState::default();
    state.select(Some(app.sel_agent));
    f.render_stateful_widget(
        List::new(items)
            .block(panel(format!("Agents  ({})", app.agents.len())))
            .highlight_style(Style::default().bg(C_SEL))
            .highlight_symbol("▶ "),
        area,
        &mut state,
    );
}

pub fn draw_agent_detail(f: &mut Frame, app: &App, area: Rect) {
    let Some((id, scope)) = app.agents.get(app.sel_agent) else {
        return;
    };
    let is_active = app.active_agent.as_deref() == Some(id.as_str());
    let active_label = if is_active { "  (active)" } else { "" };
    let title = format!("{id}  ·  {scope}{active_label}");
    f.render_widget(
        Paragraph::new(app.agent_detail_text.clone())
            .block(panel(title))
            .wrap(Wrap { trim: false })
            .style(Style::default().fg(C_FG).bg(C_BG)),
        area,
    );
}
