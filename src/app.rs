use anyhow::Result;
use copypasta::{ClipboardContext, ClipboardProvider};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::Instant;

use crate::services::{
    manager::ServiceManager,
    traits::{ViewState, ViewType},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    Command,
    Search,
}

// Removed hardcoded CurrentView enum - now using ViewState from services

pub struct App {
    pub running: bool,
    pub input_mode: InputMode,
    pub current_view: Option<ViewState>,
    pub input_buffer: String,
    pub view_stack: Vec<ViewState>,
    pub last_refresh: Instant,
    pub loading: bool,
    pub error_message: Option<String>,
    pub copy_status: Option<(String, Instant)>, // (message, created_at timestamp)
    pub aws_profile: String,
    pub aws_region: String,
    pub service_manager: ServiceManager,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            input_mode: InputMode::Normal,
            current_view: None,
            input_buffer: String::new(),
            view_stack: Vec::new(),
            last_refresh: Instant::now(),
            loading: false,
            error_message: None,
            copy_status: None,
            aws_profile: "default".to_string(),
            aws_region: "us-east-1".to_string(),
            service_manager: ServiceManager::new(),
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_aws_config(profile: String, region: String) -> Self {
        let mut app = Self::default();
        app.aws_profile = profile;
        app.aws_region = region;
        app
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_mode(key),
            InputMode::Command => self.handle_command_mode(key),
            InputMode::Search => self.handle_search_mode(key),
        }
    }

    fn handle_normal_mode(&mut self, key: KeyEvent) -> Result<()> {
        match (key.code, key.modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => self.running = false,
            (KeyCode::Char('r'), KeyModifiers::CONTROL) => {
                self.refresh_data();
            }
            (KeyCode::Char('q'), KeyModifiers::NONE) => self.running = false,
            (KeyCode::Char(':'), KeyModifiers::NONE) => {
                self.input_mode = InputMode::Command;
                self.input_buffer.clear();
            }
            (KeyCode::Char('/'), KeyModifiers::NONE) => {
                self.input_mode = InputMode::Search;
                self.input_buffer.clear();
            }
            (KeyCode::Enter, KeyModifiers::NONE) => {
                self.handle_enter_key();
            }
            (KeyCode::Esc, KeyModifiers::NONE) => {
                self.handle_escape_key();
            }
            (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::NONE) => {
                if let Some(view_state) = &mut self.current_view {
                    if view_state.selected_index > 0 {
                        view_state.selected_index -= 1;
                    }
                }
            }
            (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::NONE) => {
                if let Some(view_state) = &self.current_view {
                    let max_index = self.get_filtered_data_count(view_state).saturating_sub(1);
                    if let Some(current_view) = &mut self.current_view {
                        if current_view.selected_index < max_index {
                            current_view.selected_index += 1;
                        }
                    }
                }
            }
            (KeyCode::Char('r'), KeyModifiers::NONE) => {
                self.refresh_data();
            }
            (KeyCode::Char('c'), KeyModifiers::NONE) => {
                self.copy_selected_to_clipboard();
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_command_mode(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.input_buffer.clear();
            }
            KeyCode::Enter => {
                self.execute_command()?;
                self.input_mode = InputMode::Normal;
                self.input_buffer.clear();
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_search_mode(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.input_buffer.clear();
                if let Some(view_state) = &mut self.current_view {
                    view_state.search_filter.clear();
                    view_state.selected_index = 0;
                }
            }
            KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
                if let Some(view_state) = &mut self.current_view {
                    view_state.search_filter = self.input_buffer.clone();
                    view_state.selected_index = 0;
                }
                self.input_buffer.clear();
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
                if let Some(view_state) = &mut self.current_view {
                    view_state.search_filter = self.input_buffer.clone();
                    view_state.selected_index = 0;
                }
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
                if let Some(view_state) = &mut self.current_view {
                    view_state.search_filter = self.input_buffer.clone();
                    view_state.selected_index = 0;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn execute_command(&mut self) -> Result<()> {
        match self.input_buffer.as_str() {
            "quit" | "q" => self.running = false,
            "refresh" | "r" => {
                self.refresh_data();
            }
            command => {
                // Try to find service by command
                if let Some((service_id, _)) = self.service_manager.get_service_by_command(command)
                {
                    let view_state = ViewState::new(service_id.clone(), ViewType::List);
                    self.current_view = Some(view_state);
                    self.refresh_data();
                }
            }
        }
        Ok(())
    }

    pub fn refresh_data(&mut self) {
        self.loading = true;
        self.last_refresh = Instant::now();
        self.error_message = None;
        self.clear_expired_copy_status();
    }

    pub fn clear_expired_copy_status(&mut self) {
        if let Some((_, copy_time)) = &self.copy_status {
            if copy_time.elapsed() >= std::time::Duration::from_secs(3) {
                self.copy_status = None;
            }
        }
    }

    pub fn get_filtered_data_count(&self, view_state: &ViewState) -> usize {
        if let Some(service) = self.service_manager.get_service(&view_state.service_id) {
            if let Some(data) = self
                .service_manager
                .get_service_data(&view_state.service_id)
            {
                return service.filter_data(data, &view_state.search_filter).len();
            }
        }
        0
    }

    pub async fn load_current_service_data(&mut self) -> Result<()> {
        if let Some(view_state) = &self.current_view {
            self.service_manager
                .load_service_data(&view_state.service_id, view_state)
                .await?;
        }
        Ok(())
    }

    pub fn set_error(&mut self, error: String) {
        self.loading = false;
        self.error_message = Some(error);
    }

    pub fn finish_loading(&mut self) {
        self.loading = false;
        self.error_message = None;

        // Reset selected index if it's out of bounds
        let max_index = if let Some(view_state) = &self.current_view {
            self.get_filtered_data_count(view_state).saturating_sub(1)
        } else {
            0
        };

        if let Some(view_state) = &mut self.current_view {
            if view_state.selected_index > max_index {
                view_state.selected_index = max_index;
            }
        }
    }

    fn handle_enter_key(&mut self) {
        if let Some(current_view) = &mut self.current_view {
            if let Some(service) = self.service_manager.get_service(&current_view.service_id) {
                if let Some(data) = self
                    .service_manager
                    .get_service_data(&current_view.service_id)
                {
                    if let Some(new_view) = service.handle_enter(current_view, data) {
                        self.view_stack.push(current_view.clone());
                        self.current_view = Some(new_view);
                        self.refresh_data();
                    }
                }
            }
        }
    }

    fn handle_escape_key(&mut self) {
        if let Some(previous_view) = self.view_stack.pop() {
            self.current_view = Some(previous_view);
            self.refresh_data(); // Reload data for the previous view
        }
    }

    fn copy_selected_to_clipboard(&mut self) {
        let mut ctx = match ClipboardContext::new() {
            Ok(ctx) => ctx,
            Err(_) => return, // Silently fail if clipboard is not available
        };

        if let Some(view_state) = &self.current_view {
            if let Some(service) = self.service_manager.get_service(&view_state.service_id) {
                if let Some(data) = self
                    .service_manager
                    .get_service_data(&view_state.service_id)
                {
                    if let Some((content, display_name)) =
                        service.get_copy_content(view_state, data)
                    {
                        if ctx.set_contents(content).is_ok() {
                            self.copy_status =
                                Some((format!("✓ {} copied", display_name), Instant::now()));
                        }
                    }
                }
            }
        }
    }
}
