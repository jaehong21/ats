use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io,
    time::{Duration, Instant},
};
use tokio::time::sleep;

mod app;
mod services;
mod ui;
mod utils;

use app::{App, CurrentView};
use services::ecr::ECRService;
use ui::layout::render_layout;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Clear the terminal
    terminal.clear()?;

    // Create app state
    let mut app = App::new();

    // Create ECR client
    let ecr_client = utils::aws::create_ecr_client().await?;
    let ecr_service = ECRService::new(ecr_client);

    // Initial data load
    app.refresh_data();
    load_ecr_data(&mut app, &ecr_service).await?;

    // Main application loop
    let mut last_tick = Instant::now();
    let _tick_rate = Duration::from_millis(250);

    while app.running {
        // Handle events
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                app.handle_key_event(key)?;
            }
        }

        // Auto-refresh data every 30 seconds or when requested
        if app.loading || last_tick.elapsed() >= Duration::from_secs(30) {
            if app.loading {
                match app.current_view {
                    CurrentView::ECR => load_ecr_data(&mut app, &ecr_service).await?,
                    CurrentView::ECRImages => load_ecr_images_data(&mut app, &ecr_service).await?,
                }
            }
            last_tick = Instant::now();
        }

        // Draw UI
        terminal.draw(|f| render_layout(f, &app))?;

        // Control the loop timing
        sleep(Duration::from_millis(16)).await; // ~60 FPS
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

async fn load_ecr_data(app: &mut App, ecr_service: &ECRService) -> Result<()> {
    match ecr_service.list_repositories().await {
        Ok(repositories) => {
            app.set_ecr_repositories(repositories);
        }
        Err(e) => {
            // Store error in app state instead of printing to terminal
            app.set_error(format!("Failed to load ECR repositories: {}", e));
        }
    }
    Ok(())
}

async fn load_ecr_images_data(app: &mut App, ecr_service: &ECRService) -> Result<()> {
    if let Some(repo_name) = &app.current_repository {
        match ecr_service.get_repository_images(repo_name).await {
            Ok(images) => {
                app.set_ecr_images(images);
            }
            Err(e) => {
                // Store error in app state instead of printing to terminal
                app.set_error(format!(
                    "Failed to load ECR images for {}: {}",
                    repo_name, e
                ));
            }
        }
    }
    Ok(())
}
