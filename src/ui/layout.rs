use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::app::App;

pub fn render_layout(f: &mut Frame, app: &App) {
    let size = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Length(3), // Input bar
            Constraint::Min(0),    // Main content
            Constraint::Length(1), // Footer
        ])
        .split(size);

    // Render each section
    super::header::render_header(f, chunks[0], app);
    super::input::render_input(f, chunks[1], app);
    super::content::render_content(f, chunks[2], app);
    super::footer::render_footer(f, chunks[3], app);
}
