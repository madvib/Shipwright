//! Render the Docs section: Notes and ADRs.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState, Paragraph};

use super::data::ViewData;
use super::nav::{NavState, Panel};
use super::theme::*;

pub fn draw(frame: &mut Frame, nav: &NavState, data: &ViewData, area: Rect) {
    match nav.panel() {
        Panel::Notes => draw_notes(frame, nav, data, area),
        Panel::Adrs => draw_adrs(frame, nav, data, area),
        _ => {}
    }
}

fn draw_notes(frame: &mut Frame, nav: &NavState, data: &ViewData, area: Rect) {
    if data.notes.is_empty() {
        frame.render_widget(
            Paragraph::new("  No notes. Press 'n' to create one.")
                .style(Style::default().fg(C_MUT).bg(C_BG))
                .block(panel("Notes")),
            area,
        );
        return;
    }

    let items: Vec<ListItem> = data
        .notes
        .iter()
        .map(|n| {
            let branch = n.branch.as_deref().unwrap_or("");
            let ts = truncate_ts(&n.updated_at);
            ListItem::new(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(format!("{:<32}", truncate(&n.title, 30)), Style::default().fg(C_FG)),
                Span::styled(format!(" {:<14}", branch), Style::default().fg(C_MUT)),
                Span::styled(ts, Style::default().fg(C_MUT)),
            ]))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(nav.list_selected));
    frame.render_stateful_widget(
        List::new(items)
            .block(panel(format!("Notes  ({})", data.notes.len())))
            .highlight_style(selected_style())
            .highlight_symbol("\u{25b6} "),
        area,
        &mut state,
    );
}

fn draw_adrs(frame: &mut Frame, nav: &NavState, data: &ViewData, area: Rect) {
    let filter = nav.status_filter.as_deref();
    let adrs: Vec<&_> = data
        .adrs
        .iter()
        .filter(|a| filter.is_none() || filter == Some(a.status.as_str()))
        .collect();
    let title = match filter {
        Some(f) => format!("ADRs  ({})  [{}]", adrs.len(), f),
        None => format!("ADRs  ({})", adrs.len()),
    };
    if adrs.is_empty() {
        frame.render_widget(
            Paragraph::new("  No ADRs. Press 'n' to create one.")
                .style(Style::default().fg(C_MUT).bg(C_BG))
                .block(panel(title)),
            area,
        );
        return;
    }

    let items: Vec<ListItem> = adrs
        .iter()
        .map(|a| {
            let sc = status_color(&a.status);
            let ts = truncate_ts(&a.date);
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {} ", status_sym(&a.status)),
                    Style::default().fg(sc),
                ),
                Span::styled(format!("{:<32}", truncate(&a.title, 30)), Style::default().fg(C_FG)),
                Span::styled(format!(" {:<10}", a.status), Style::default().fg(sc)),
                Span::styled(ts, Style::default().fg(C_MUT)),
            ]))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(nav.list_selected));
    frame.render_stateful_widget(
        List::new(items)
            .block(panel(title))
            .highlight_style(selected_style())
            .highlight_symbol("\u{25b6} "),
        area,
        &mut state,
    );
}
