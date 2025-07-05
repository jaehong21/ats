use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{
    io,
    time::{Duration, Instant},
};
use tokio::time::sleep;

mod app;
mod services;
mod ui;
mod utils;

use app::App;
use services::{
    ecr::ECRService,
    traits::{ServiceId, ViewState, ViewType},
};
use std::sync::Arc;
use ui::layout::render_layout;

#[derive(Parser)]
#[command(name = "ats")]
#[command(about = "AWS Terminal Service - Terminal UI for managing AWS services")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Args {
    #[arg(short = 'p', long = "profile", help = "AWS profile to use")]
    profile: Option<String>,

    #[arg(short = 'r', long = "region", help = "AWS region to use")]
    region: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Clear the terminal
    terminal.clear()?;

    // Determine actual profile and region being used
    let actual_profile = args
        .profile
        .clone()
        .or_else(|| std::env::var("AWS_PROFILE").ok())
        .unwrap_or_else(|| "default".to_string());

    let actual_region = args
        .region
        .clone()
        .or_else(|| std::env::var("AWS_REGION").ok())
        .or_else(|| std::env::var("AWS_DEFAULT_REGION").ok())
        .unwrap_or_else(|| "us-east-1".to_string());

    // Create app state with actual AWS config
    let mut app = App::new(actual_profile, actual_region);

    // Create ECR client and service
    let ecr_client = utils::aws::create_ecr_client(args.profile, args.region).await?;
    let ecr_service = ECRService::new(ecr_client);

    // Register ECR service
    app.service_manager.register_service(Arc::new(ecr_service));

    // Set initial view to ECR
    let ecr_service_id = ServiceId::new("ecr");
    let initial_view = ViewState::new(ecr_service_id, ViewType::List);
    app.current_view = Some(initial_view);

    // Initial data load
    app.refresh_data();
    if let Err(e) = app.load_current_service_data().await {
        app.set_error(e.to_string());
    } else {
        app.finish_loading();
    }

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
                if let Err(e) = app.load_current_service_data().await {
                    app.set_error(e.to_string());
                } else {
                    app.finish_loading();
                }
            }
            last_tick = Instant::now();
        }

        // Clear expired copy status
        app.clear_expired_copy_status();

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
