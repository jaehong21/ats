use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
};

use crate::app::{App, CurrentView};

pub fn render_content(f: &mut Frame, area: Rect, app: &App) {
    match app.current_view {
        CurrentView::ECR => render_ecr_content(f, area, app),
        CurrentView::ECRImages => render_ecr_images_content(f, area, app),
    }
}

fn render_ecr_content(f: &mut Frame, area: Rect, app: &App) {
    let repositories = app.filtered_repositories();

    let title = if app.loading {
        "ECR Repositories (Loading...)".to_string()
    } else if let Some(ref _error) = app.error_message {
        "ECR Repositories - Error".to_string()
    } else if app.search_filter.is_empty() {
        format!("ECR Repositories ({})", repositories.len())
    } else {
        format!(
            "ECR Repositories ({}/{}) - Filter: {}",
            repositories.len(),
            app.ecr_repositories.len(),
            app.search_filter
        )
    };

    if let Some(ref error) = app.error_message {
        let error_paragraph = ratatui::widgets::Paragraph::new(error.as_str())
            .block(Block::default().borders(Borders::ALL).title(title))
            .style(Style::default().fg(Color::Red));

        f.render_widget(error_paragraph, area);
        return;
    }

    if repositories.is_empty() {
        let message = if app.loading {
            "Loading ECR repositories..."
        } else if !app.search_filter.is_empty() {
            "No repositories match the current filter"
        } else {
            "No ECR repositories found"
        };

        let empty_paragraph = ratatui::widgets::Paragraph::new(message)
            .block(Block::default().borders(Borders::ALL).title(title))
            .style(Style::default().fg(Color::Gray));

        f.render_widget(empty_paragraph, area);
        return;
    }

    let header_cells = [
        "REPOSITORY NAME",
        "REGISTRY ID",
        "CREATED",
        "TAG MUTABILITY",
        "SCAN ON PUSH",
        "ENCRYPTION",
    ]
    .iter()
    .map(|h| {
        Cell::from(*h).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    });

    let header = Row::new(header_cells).height(1).bottom_margin(0);

    let rows: Vec<Row> = repositories
        .iter()
        .enumerate()
        .map(|(i, repo)| {
            let created_str = repo
                .created_at
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            let scan_on_push = if repo.image_scanning_configuration {
                "Yes"
            } else {
                "No"
            };

            let style = if i == app.selected_index {
                Style::default().bg(Color::Yellow).fg(Color::Black)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(repo.repository_name.clone()),
                Cell::from(repo.registry_id.clone()),
                Cell::from(created_str),
                Cell::from(repo.image_tag_mutability.clone()),
                Cell::from(scan_on_push),
                Cell::from(repo.encryption_configuration.clone()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(30), // Repository name
            Constraint::Length(15), // Registry ID
            Constraint::Length(20), // Created
            Constraint::Length(16), // Tag mutability
            Constraint::Length(12), // Scan on push
            Constraint::Length(12), // Encryption
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title))
    .row_highlight_style(Style::default().bg(Color::Blue).fg(Color::White));

    f.render_widget(table, area);
}

fn render_ecr_images_content(f: &mut Frame, area: Rect, app: &App) {
    let images = app.filtered_images();
    let default_repo = "Unknown".to_string();
    let repo_name = app.current_repository.as_ref().unwrap_or(&default_repo);

    let title = if app.loading {
        format!("ECR Repositories: {} > ECR Images (Loading...)", repo_name)
    } else if let Some(ref _error) = app.error_message {
        format!("ECR Repositories: {} > ECR Images - Error", repo_name)
    } else if app.search_filter.is_empty() {
        format!(
            "ECR Repositories: {} > ECR Images ({})",
            repo_name,
            images.len()
        )
    } else {
        format!(
            "ECR Repositories: {} > ECR Images ({}/{}) - Filter: {}",
            repo_name,
            images.len(),
            app.ecr_images.len(),
            app.search_filter
        )
    };

    if let Some(ref error) = app.error_message {
        let error_paragraph = ratatui::widgets::Paragraph::new(error.as_str())
            .block(Block::default().borders(Borders::ALL).title(title))
            .style(Style::default().fg(Color::Red));

        f.render_widget(error_paragraph, area);
        return;
    }

    if images.is_empty() {
        let message = if app.loading {
            "Loading ECR images..."
        } else if !app.search_filter.is_empty() {
            "No images match the current filter"
        } else {
            "No ECR images found"
        };

        let empty_paragraph = ratatui::widgets::Paragraph::new(message)
            .block(Block::default().borders(Borders::ALL).title(title))
            .style(Style::default().fg(Color::Gray));

        f.render_widget(empty_paragraph, area);
        return;
    }

    let header_cells = [
        "IMAGE TAG",
        "DIGEST",
        "PUSHED AT",
        "SIZE",
        "VULNERABILITIES",
    ]
    .iter()
    .map(|h| {
        Cell::from(*h).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    });

    let header = Row::new(header_cells).height(1).bottom_margin(0);

    let rows: Vec<Row> = images
        .iter()
        .enumerate()
        .map(|(i, image)| {
            let tag = image
                .image_tag
                .as_ref()
                .unwrap_or(&"<none>".to_string())
                .clone();
            let digest = image.image_digest.to_string(); // Show first 12 chars
            let pushed_at = image
                .image_pushed_at
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            let size = image
                .image_size_in_bytes
                .map(|s| format!("{:.1} MB", s as f64 / 1_048_576.0))
                .unwrap_or_else(|| "Unknown".to_string());
            let vulnerabilities = image
                .vulnerability_scan_summary
                .as_ref()
                .unwrap_or(&"Not scanned".to_string())
                .clone();

            let style = if i == app.selected_index {
                Style::default().bg(Color::Yellow).fg(Color::Black)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(tag),
                Cell::from(digest),
                Cell::from(pushed_at),
                Cell::from(size),
                Cell::from(vulnerabilities),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(25), // Image tag
            Constraint::Length(30), // Digest
            Constraint::Length(20), // Pushed at
            Constraint::Length(12), // Size
            Constraint::Length(20), // Vulnerabilities
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title))
    .row_highlight_style(Style::default().bg(Color::Blue).fg(Color::White));

    f.render_widget(table, area);
}
