//! Top-level rendering: header tabs + sidebar + content area + modal + footer.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use super::data::ViewData;
use super::modal::Modal;
use super::nav::{NavState, Panel, Screen, Section};
use super::theme::*;
use super::{render_config, render_docs, render_events, render_settings, render_workflow};

pub fn draw(frame: &mut Frame, nav: &NavState, data: &ViewData, modal: &Option<Modal>) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header
            Constraint::Min(0),    // body
            Constraint::Length(1), // footer
        ])
        .split(frame.area());

    draw_header(frame, nav, outer[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(15), Constraint::Min(40)])
        .split(outer[1]);

    draw_sidebar(frame, nav, body[0]);
    let content_area = body[1];

    match &nav.screen {
        Screen::Detail {
            title,
            body: detail_body,
            scroll,
        } => {
            draw_detail_page(frame, title, detail_body, *scroll, content_area);
        }
        Screen::List => {
            draw_content(frame, nav, data, content_area);
        }
    }

    draw_footer(frame, nav, outer[2]);

    if let Some(m) = modal {
        draw_modal(frame, m);
    }
}

fn draw_header(frame: &mut Frame, nav: &NavState, area: Rect) {
    let mut spans = vec![Span::styled(
        " \u{25c6} ship ",
        Style::default().fg(C_PRI).add_modifier(Modifier::BOLD),
    )];
    spans.push(Span::styled("\u{2502} ", Style::default().fg(C_BOR)));

    for (i, section) in Section::ALL.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" \u{2502} ", Style::default().fg(C_BOR)));
        }
        let style = if i == nav.section_idx {
            Style::default()
                .fg(C_PRI)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default().fg(C_MUT)
        };
        spans.push(Span::styled(
            format!("{} {}", section.key_hint(), section.label()),
            style,
        ));
    }

    let header = Paragraph::new(Line::from(spans)).style(Style::default().bg(C_BG));
    frame.render_widget(header, area);
}

fn draw_sidebar(frame: &mut Frame, nav: &NavState, area: Rect) {
    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(C_BOR))
        .style(Style::default().bg(C_BG));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();
    let panels = nav.section().panels();
    for (i, p) in panels.iter().enumerate() {
        let (marker, style) = if i == nav.panel_idx {
            (
                "\u{25b8} ",
                Style::default().fg(C_PRI).add_modifier(Modifier::BOLD),
            )
        } else {
            ("  ", Style::default().fg(C_FG))
        };
        lines.push(Line::from(vec![
            Span::styled(marker, style),
            Span::styled(p.label(), style),
        ]));
    }

    let sidebar = Paragraph::new(lines);
    frame.render_widget(sidebar, inner);
}

fn draw_content(frame: &mut Frame, nav: &NavState, data: &ViewData, area: Rect) {
    match nav.section() {
        Section::Workflow => render_workflow::draw(frame, nav, data, area),
        Section::Docs => render_docs::draw(frame, nav, data, area),
        Section::Agents => render_config::draw(frame, nav, data, area),
        Section::Events => render_events::draw(frame, nav, data, area),
        Section::Settings => render_settings::draw(frame, data, area),
    }
}

/// Render a full-page detail view in the content area (right of sidebar).
fn draw_detail_page(frame: &mut Frame, title: &str, body: &str, scroll: usize, area: Rect) {
    let block = panel(title.to_string()).border_style(Style::default().fg(C_PRI));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines: Vec<Line> = body
        .lines()
        .skip(scroll)
        .map(|l| super::theme::style_md_line(l))
        .collect();
    let p = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(p, inner);
}

fn draw_footer(frame: &mut Frame, nav: &NavState, area: Rect) {
    let hint = if nav.is_detail() {
        "  \u{2191}\u{2193} jk scroll \u{00b7} Esc/Backspace back \u{00b7} q quit".to_string()
    } else {
        build_footer_hint(nav)
    };
    let footer = Paragraph::new(Line::from(Span::styled(hint, Style::default().fg(C_MUT))))
        .style(Style::default().bg(C_BG));
    frame.render_widget(footer, area);
}

fn build_footer_hint(nav: &NavState) -> String {
    let panel = nav.panel();
    let mut parts: Vec<&str> = vec!["  \u{2191}\u{2193} jk nav", "\u{23ce} open"];
    if panel == Panel::Notes {
        parts.extend(&["n new", "e edit", "d delete"]);
    } else if panel == Panel::Adrs {
        parts.extend(&["n new", "d delete"]);
    } else if panel == Panel::ProjectSettings {
        parts.push("e edit");
    }
    if panel.has_status_filter() {
        parts.push("f filter");
    }
    parts.extend(&["Tab panel", "H/L section", "r reload", "q quit"]);
    parts.join(" \u{00b7} ")
}

fn draw_modal(frame: &mut Frame, modal: &Modal) {
    let area = centered_rect(70, 70, frame.area());
    frame.render_widget(Clear, area);

    match modal {
        Modal::Form(form) => {
            let block = panel(form.title.clone()).border_style(Style::default().fg(C_AMBER));
            let inner = block.inner(area);
            frame.render_widget(block, area);

            let mut lines: Vec<Line> = Vec::new();
            for (i, field) in form.fields.iter().enumerate() {
                let label_style = if i == form.focused_field {
                    Style::default().fg(C_PRI).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(C_MUT)
                };
                lines.push(Line::from(Span::styled(
                    format!("{}:", field.label),
                    label_style,
                )));
                let cursor = if i == form.focused_field { "_" } else { "" };
                lines.push(Line::from(Span::styled(
                    format!("  {}{}", field.value, cursor),
                    Style::default().fg(C_FG),
                )));
                lines.push(Line::from(""));
            }
            lines.push(Line::from(Span::styled(
                "Tab=next field  Enter=submit  Esc=cancel",
                Style::default().fg(C_MUT),
            )));
            let p = Paragraph::new(lines).wrap(Wrap { trim: false });
            frame.render_widget(p, inner);
        }
        Modal::Confirm { title, message, .. } => {
            let block = panel(title.clone()).border_style(Style::default().fg(C_RED));
            let inner = block.inner(area);
            frame.render_widget(block, area);
            let lines = vec![
                Line::from(""),
                Line::from(Span::styled(message.clone(), Style::default().fg(C_FG))),
                Line::from(""),
                Line::from(Span::styled(
                    "y=confirm  n/Esc=cancel",
                    Style::default().fg(C_MUT),
                )),
            ];
            let p = Paragraph::new(lines).wrap(Wrap { trim: false });
            frame.render_widget(p, inner);
        }
    }
}

fn centered_rect(pct_x: u16, pct_y: u16, r: Rect) -> Rect {
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - pct_y) / 2),
            Constraint::Percentage(pct_y),
            Constraint::Percentage((100 - pct_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - pct_x) / 2),
            Constraint::Percentage(pct_x),
            Constraint::Percentage((100 - pct_x) / 2),
        ])
        .split(vert[1])[1]
}
