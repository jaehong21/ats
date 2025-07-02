use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

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
    let status_text = if app.loading { "Loading..." } else { "Ready" };

    let status = Paragraph::new(Line::from(vec![Span::styled(
        status_text,
        Style::default().fg(Color::Green),
    )]))
    .block(Block::default().borders(Borders::NONE));
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
