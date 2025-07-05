use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::{App, InputMode};

pub fn render_input(f: &mut Frame, area: Rect, app: &App) {
    let (prompt, content, mode_indicator) = match app.input_mode {
        InputMode::Normal => {
            let current_service = if let Some(view_state) = &app.current_view {
                match view_state.view_type {
                    crate::services::traits::ViewType::List => view_state.service_id.to_string(),
                    crate::services::traits::ViewType::Detail => {
                        if let Some(context) = &view_state.context {
                            format!("{}/{}", view_state.service_id, context)
                        } else {
                            format!("{}/detail", view_state.service_id)
                        }
                    }
                    crate::services::traits::ViewType::Custom(ref name) => {
                        format!("{}/{}", view_state.service_id, name)
                    }
                }
            } else {
                "no service".to_string()
            };
            ("> ", current_service, "".to_string())
        }
        InputMode::Command => (":", app.input_buffer.clone(), "[:]".to_string()),
        InputMode::Search => ("/", app.input_buffer.clone(), "[/]".to_string()),
    };

    let input_style = match app.input_mode {
        InputMode::Normal => Style::default().fg(Color::White),
        InputMode::Command => Style::default().fg(Color::Cyan),
        InputMode::Search => Style::default().fg(Color::Yellow),
    };

    let content_len = content.len();
    let mut spans = vec![
        Span::styled(prompt, input_style),
        Span::styled(content, input_style),
    ];

    if !mode_indicator.is_empty() {
        spans.push(Span::raw(
            " ".repeat(
                area.width
                    .saturating_sub(
                        prompt.len() as u16 + content_len as u16 + mode_indicator.len() as u16,
                    )
                    .saturating_sub(2) as usize,
            ),
        ));
        spans.push(Span::styled(
            mode_indicator,
            Style::default().fg(Color::Gray),
        ));
    }

    let input_paragraph =
        Paragraph::new(Line::from(spans)).block(Block::default().borders(Borders::ALL));

    f.render_widget(input_paragraph, area);

    // Set cursor position for input modes
    if matches!(app.input_mode, InputMode::Command | InputMode::Search) {
        f.set_cursor_position((
            area.x + prompt.len() as u16 + content_len as u16 + 1,
            area.y + 1,
        ));
    }
}
