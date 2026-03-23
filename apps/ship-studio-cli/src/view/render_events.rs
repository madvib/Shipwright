//! Render the Events section: append-only event log.

use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Paragraph, Row, Table};

use super::data::ViewData;
use super::nav::NavState;
use super::theme::*;

pub fn draw(frame: &mut Frame, nav: &NavState, data: &ViewData, area: Rect) {
    if data.events.is_empty() {
        frame.render_widget(
            Paragraph::new("  No events recorded. Events are logged during workspace sessions.")
                .style(Style::default().fg(C_MUT).bg(C_BG))
                .block(panel("Event Log")),
            area,
        );
        return;
    }

    let header =
        Row::new(vec!["ID", "Time", "Actor", "Entity", "Action", "Subject"]).style(header_style());

    let rows: Vec<Row> = data
        .events
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let ts = e.timestamp.format("%m-%d %H:%M").to_string();
            let style = if i == nav.list_selected {
                selected_style()
            } else {
                Style::default().fg(C_FG)
            };
            Row::new(vec![
                e.id[..8.min(e.id.len())].to_string(),
                ts,
                e.actor.clone(),
                format!("{:?}", e.entity),
                format!("{:?}", e.action),
                e.subject.clone(),
            ])
            .style(style)
        })
        .collect();

    let title = format!("Event Log  ({} entries)", data.events.len());
    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(14),
            Constraint::Min(16),
        ],
    )
    .header(header)
    .block(panel(title));

    frame.render_widget(table, area);
}
