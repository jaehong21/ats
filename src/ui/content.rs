use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders},
};

use crate::app::App;

pub fn render_content(f: &mut Frame, area: Rect, app: &App) {
    // First check if there's an error to display
    if let Some(ref error_message) = app.error_message {
        let title = if let Some(view_state) = &app.current_view {
            format!("{} - Error", view_state.service_id)
        } else {
            "Error".to_string()
        };

        let error_paragraph = ratatui::widgets::Paragraph::new(error_message.as_str())
            .block(Block::default().borders(Borders::ALL).title(title))
            .style(Style::default().fg(Color::Red));

        f.render_widget(error_paragraph, area);
        return;
    }

    // Try to render service content
    if let Some(view_state) = &app.current_view {
        if let Some(service) = app.service_manager.get_service(&view_state.service_id) {
            if let Some(data) = app.service_manager.get_service_data(&view_state.service_id) {
                service.render(f, area, app, view_state, data);
                return;
            }

            // Service exists but no data - show loading or empty state
            let message = if app.loading {
                "Loading..."
            } else {
                "No data available"
            };

            let loading_paragraph = ratatui::widgets::Paragraph::new(message)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!("{}", view_state.service_id)),
                )
                .style(Style::default().fg(Color::Yellow));

            f.render_widget(loading_paragraph, area);
            return;
        }
    }

    // Show default message when no service is active
    let message = "No service selected. Use :ecr to start.";
    let empty_paragraph = ratatui::widgets::Paragraph::new(message)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("ATS - AWS Terminal Service"),
        )
        .style(Style::default().fg(Color::Gray));

    f.render_widget(empty_paragraph, area);
}
