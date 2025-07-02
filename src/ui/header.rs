use chrono::Local;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::App;

pub fn render_header(f: &mut Frame, area: Rect, _app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20), // ATS info
            Constraint::Min(0),     // Spacer
            Constraint::Length(40), // AWS info
        ])
        .split(area);

    // Get AWS info
    let profile = crate::utils::aws::get_current_profile();
    let region = crate::utils::aws::get_current_region();
    let current_time = Local::now().format("%H:%M:%S").to_string();

    // Left side - Application info
    let version = env!("CARGO_PKG_VERSION");
    let app_info = Paragraph::new(Line::from(vec![
        Span::styled("ATS", Style::default().fg(Color::Yellow)),
        Span::raw(format!(" v{}", version)),
    ]))
    .block(Block::default().borders(Borders::NONE));
    f.render_widget(app_info, chunks[0]);

    // Right side - AWS info and time
    let aws_info = Paragraph::new(Line::from(vec![
        Span::styled("Profile: ", Style::default().fg(Color::Gray)),
        Span::styled(&profile, Style::default().fg(Color::Green)),
        Span::raw(" | "),
        Span::styled("Region: ", Style::default().fg(Color::Gray)),
        Span::styled(&region, Style::default().fg(Color::Green)),
        Span::raw(" | "),
        Span::styled(&current_time, Style::default().fg(Color::Yellow)),
    ]))
    .block(Block::default().borders(Borders::NONE));
    f.render_widget(aws_info, chunks[2]);
}
