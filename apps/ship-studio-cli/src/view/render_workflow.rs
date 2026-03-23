//! Render the Workflow section: Targets, Capabilities, Jobs.

use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState, Paragraph, Row, Table};

use super::data::ViewData;
use super::nav::{NavState, Panel};
use super::theme::*;

pub fn draw(frame: &mut Frame, nav: &NavState, data: &ViewData, area: Rect) {
    match nav.panel() {
        Panel::Targets => draw_targets(frame, nav, data, area),
        Panel::Capabilities => draw_capabilities(frame, nav, data, area),
        Panel::Jobs => draw_jobs(frame, nav, data, area),
        _ => {}
    }
}

fn draw_targets(frame: &mut Frame, nav: &NavState, data: &ViewData, area: Rect) {
    let filter = nav.status_filter.as_deref();
    let targets: Vec<&_> = data
        .targets
        .iter()
        .filter(|t| filter.is_none() || filter == Some(t.status.as_str()))
        .collect();
    let title = match filter {
        Some(f) => format!("Targets  ({})  [{}]", targets.len(), f),
        None => format!("Targets  ({})", targets.len()),
    };
    if targets.is_empty() {
        frame.render_widget(
            Paragraph::new("  No targets.")
                .style(Style::default().fg(C_MUT).bg(C_BG))
                .block(panel(title)),
            area,
        );
        return;
    }

    let items: Vec<ListItem> = targets
        .iter()
        .map(|t| {
            let sc = status_color(&t.status);
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {} ", status_sym(&t.status)),
                    Style::default().fg(sc),
                ),
                Span::styled(format!("{:<32}", truncate(&t.title, 30)), Style::default().fg(C_FG)),
                Span::styled(format!(" {:<10}", t.kind), Style::default().fg(C_MUT)),
                Span::styled(t.status.clone(), Style::default().fg(sc)),
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

fn draw_capabilities(frame: &mut Frame, nav: &NavState, data: &ViewData, area: Rect) {
    let filter = nav.status_filter.as_deref();
    let caps: Vec<&_> = data
        .capabilities
        .iter()
        .filter(|c| filter.is_none() || filter == Some(c.status.as_str()))
        .collect();
    let title = match filter {
        Some(f) => format!("Capabilities  ({})  [{}]", caps.len(), f),
        None => format!("Capabilities  ({})", caps.len()),
    };
    if caps.is_empty() {
        frame.render_widget(
            Paragraph::new("  No capabilities.")
                .style(Style::default().fg(C_MUT).bg(C_BG))
                .block(panel(title)),
            area,
        );
        return;
    }

    let items: Vec<ListItem> = caps
        .iter()
        .map(|c| {
            let sc = status_color(&c.status);
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {} ", status_sym(&c.status)),
                    Style::default().fg(sc),
                ),
                Span::styled(format!("{:<32}", truncate(&c.title, 30)), Style::default().fg(C_FG)),
                Span::styled(c.status.clone(), Style::default().fg(sc)),
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

fn draw_jobs(frame: &mut Frame, nav: &NavState, data: &ViewData, area: Rect) {
    let filter = nav.status_filter.as_deref();
    let jobs: Vec<&_> = data
        .all_jobs
        .iter()
        .filter(|j| filter.is_none() || filter == Some(j.status.as_str()))
        .collect();
    let title = match filter {
        Some(f) => format!("Jobs  ({})  [{}]", jobs.len(), f),
        None => format!("Jobs  ({})", jobs.len()),
    };

    if jobs.is_empty() {
        frame.render_widget(
            Paragraph::new("  No jobs.")
                .style(Style::default().fg(C_MUT).bg(C_BG))
                .block(panel(title)),
            area,
        );
        return;
    }

    let header =
        Row::new(vec!["", "ID", "Kind", "Status", "Branch", "Claimed"]).style(header_style());
    let rows: Vec<Row> = jobs
        .iter()
        .enumerate()
        .map(|(i, j)| {
            let sc = status_color(&j.status);
            let style = if i == nav.list_selected {
                selected_style().fg(sc)
            } else {
                normal_style().fg(sc)
            };
            let id_short = truncate(&j.id, 8);
            let branch = j.branch.as_deref().unwrap_or("\u{2014}");
            let claimed = j.claimed_by.as_deref().unwrap_or("\u{2014}");
            Row::new(vec![
                format!(" {} ", status_sym(&j.status)),
                id_short,
                j.kind.clone(),
                j.status.clone(),
                branch.to_string(),
                claimed.to_string(),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(4),
            Constraint::Length(10),
            Constraint::Length(14),
            Constraint::Length(10),
            Constraint::Min(20),
            Constraint::Length(14),
        ],
    )
    .header(header)
    .block(panel(title));
    frame.render_widget(table, area);
}
