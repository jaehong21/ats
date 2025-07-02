use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::time::Duration;

use crate::app::{App, InputMode};

pub fn render_footer(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),     // Status message
            Constraint::Length(50), // Hotkeys
        ])
        .split(area);

    // Left side - Status
    let mut status_spans = Vec::new();

    // Main status
    let status_text = if app.loading { "Loading..." } else { "Ready" };
    status_spans.push(Span::styled(status_text, Style::default().fg(Color::Green)));

    // Copy status (if present and not expired)
    if let Some((copy_msg, copy_time)) = &app.copy_status {
        if copy_time.elapsed() < Duration::from_secs(2) {
            status_spans.push(Span::raw(" | "));
            status_spans.push(Span::styled(copy_msg, Style::default().fg(Color::Green)));
        }
    }

    let status =
        Paragraph::new(Line::from(status_spans)).block(Block::default().borders(Borders::NONE));
    f.render_widget(status, chunks[0]);

    // Right side - Hotkeys
    let hotkeys = match app.input_mode {
        InputMode::Normal => vec![
            Span::styled("q ", Style::default().fg(Color::Yellow)),
            Span::raw("Quit | "),
            Span::styled(": ", Style::default().fg(Color::Yellow)),
            Span::raw("Command | "),
            Span::styled("/ ", Style::default().fg(Color::Yellow)),
            Span::raw("Search | "),
            Span::styled("c ", Style::default().fg(Color::Yellow)),
            Span::raw("Copy"),
        ],
        InputMode::Command => vec![
            Span::styled("Enter ", Style::default().fg(Color::Yellow)),
            Span::raw("Execute | "),
            Span::styled("Esc ", Style::default().fg(Color::Yellow)),
            Span::raw("Cancel"),
        ],
        InputMode::Search => vec![
            Span::styled("Enter ", Style::default().fg(Color::Yellow)),
            Span::raw("Apply | "),
            Span::styled("Esc ", Style::default().fg(Color::Yellow)),
            Span::raw("Cancel"),
        ],
    };

    let hotkeys_paragraph =
        Paragraph::new(Line::from(hotkeys)).block(Block::default().borders(Borders::NONE));
    f.render_widget(hotkeys_paragraph, chunks[1]);
}
