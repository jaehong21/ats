use anyhow::Result;
use copypasta::{ClipboardContext, ClipboardProvider};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    Command,
    Search,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub enum CurrentView {
    ECR,
    ECRImages,
}

pub struct App {
    pub running: bool,
    pub input_mode: InputMode,
    pub current_view: CurrentView,
    pub input_buffer: String,
    pub search_filter: String,
    pub selected_index: usize,
    pub last_refresh: Instant,
    pub ecr_repositories: Vec<crate::services::ecr::ECRRepository>,
    pub ecr_images: Vec<crate::services::ecr::ECRImage>,
    pub current_repository: Option<String>,
    pub view_stack: Vec<CurrentView>,
    pub loading: bool,
    pub error_message: Option<String>,
    pub copy_status: Option<(String, Instant)>, // (message, created_at timestamp)
    pub aws_profile: String,
    pub aws_region: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            input_mode: InputMode::Normal,
            current_view: CurrentView::ECR,
            input_buffer: String::new(),
            search_filter: String::new(),
            selected_index: 0,
            last_refresh: Instant::now(),
            ecr_repositories: Vec::new(),
            ecr_images: Vec::new(),
            current_repository: None,
            view_stack: Vec::new(),
            loading: false,
            error_message: None,
            copy_status: None,
            aws_profile: "default".to_string(),
            aws_region: "us-east-1".to_string(),
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
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::NONE) => {
                let max_index = match self.current_view {
                    CurrentView::ECR => self.filtered_repositories().len().saturating_sub(1),
                    CurrentView::ECRImages => self.filtered_images().len().saturating_sub(1),
                };
                if self.selected_index < max_index {
                    self.selected_index += 1;
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
                self.search_filter.clear();
                self.selected_index = 0;
            }
            KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
                self.search_filter = self.input_buffer.clone();
                self.input_buffer.clear();
                self.selected_index = 0;
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
                self.search_filter = self.input_buffer.clone();
                self.selected_index = 0;
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
                self.search_filter = self.input_buffer.clone();
                self.selected_index = 0;
            }
            _ => {}
        }
        Ok(())
    }

    fn execute_command(&mut self) -> Result<()> {
        match self.input_buffer.as_str() {
            "quit" | "q" => self.running = false,
            "ecr" => {
                self.current_view = CurrentView::ECR;
                self.selected_index = 0;
                self.refresh_data();
            }
            "refresh" | "r" => {
                self.refresh_data();
            }
            _ => {}
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

    pub fn filtered_repositories(&self) -> Vec<&crate::services::ecr::ECRRepository> {
        if self.search_filter.is_empty() {
            self.ecr_repositories.iter().collect()
        } else {
            self.ecr_repositories
                .iter()
                .filter(|repo| {
                    repo.repository_name
                        .to_lowercase()
                        .contains(&self.search_filter.to_lowercase())
                })
                .collect()
        }
    }

    pub fn set_ecr_repositories(&mut self, repositories: Vec<crate::services::ecr::ECRRepository>) {
        self.ecr_repositories = repositories;
        self.loading = false;
        self.error_message = None;

        let max_index = self.filtered_repositories().len().saturating_sub(1);
        if self.selected_index > max_index {
            self.selected_index = max_index;
        }
    }

    pub fn set_error(&mut self, error: String) {
        self.loading = false;
        self.error_message = Some(error);
    }

    fn handle_enter_key(&mut self) {
        match self.current_view {
            CurrentView::ECR => {
                let repositories = self.filtered_repositories();
                if self.selected_index < repositories.len() {
                    let repo_name = repositories[self.selected_index].repository_name.clone();
                    self.view_stack.push(self.current_view);
                    self.current_view = CurrentView::ECRImages;
                    self.current_repository = Some(repo_name);
                    self.selected_index = 0;
                    self.search_filter.clear();
                    self.refresh_data();
                }
            }
            CurrentView::ECRImages => {
                // Future: handle image drill-down
            }
        }
    }

    fn handle_escape_key(&mut self) {
        if let Some(previous_view) = self.view_stack.pop() {
            self.current_view = previous_view;
            self.current_repository = None;
            self.selected_index = 0;
            self.search_filter.clear();
        }
    }

    pub fn filtered_images(&self) -> Vec<&crate::services::ecr::ECRImage> {
        if self.search_filter.is_empty() {
            self.ecr_images.iter().collect()
        } else {
            self.ecr_images
                .iter()
                .filter(|image| {
                    image
                        .image_tag
                        .as_ref()
                        .map(|tag| {
                            tag.to_lowercase()
                                .contains(&self.search_filter.to_lowercase())
                        })
                        .unwrap_or(false)
                })
                .collect()
        }
    }

    pub fn set_ecr_images(&mut self, mut images: Vec<crate::services::ecr::ECRImage>) {
        // Sort images by pushed_at date, latest first
        images.sort_by(|a, b| {
            match (a.image_pushed_at, b.image_pushed_at) {
                (Some(a_date), Some(b_date)) => b_date.cmp(&a_date), // Latest first
                (Some(_), None) => std::cmp::Ordering::Less,         // Images with dates come first
                (None, Some(_)) => std::cmp::Ordering::Greater, // Images without dates come last
                (None, None) => std::cmp::Ordering::Equal,      // Both have no date
            }
        });

        self.ecr_images = images;
        self.loading = false;
        self.error_message = None;

        let max_index = self.filtered_images().len().saturating_sub(1);
        if self.selected_index > max_index {
            self.selected_index = max_index;
        }
    }

    fn copy_selected_to_clipboard(&mut self) {
        let mut ctx = match ClipboardContext::new() {
            Ok(ctx) => ctx,
            Err(_) => return, // Silently fail if clipboard is not available
        };

        let (content, display_name) = match self.current_view {
            CurrentView::ECR => {
                let repositories = self.filtered_repositories();
                if self.selected_index < repositories.len() {
                    let repo = &repositories[self.selected_index];
                    let content = format!("{} for {}", repo.repository_uri, repo.repository_name);
                    let display_name = repo.repository_name.clone();
                    (content, display_name)
                } else {
                    return;
                }
            }
            CurrentView::ECRImages => {
                let images = self.filtered_images();
                if self.selected_index < images.len() {
                    let image = &images[self.selected_index];
                    if let (Some(repo_name), Some(tag)) =
                        (&self.current_repository, &image.image_tag)
                    {
                        let content = format!(
                            "{}:{}",
                            self.ecr_repositories
                                .iter()
                                .find(|r| r.repository_name == *repo_name)
                                .map(|r| &r.repository_uri)
                                .unwrap_or(&format!(
                                    "unknown.dkr.ecr.region.amazonaws.com/{}",
                                    repo_name
                                )),
                            tag
                        );
                        let display_name = format!("{}:{}", repo_name, tag);
                        (content, display_name)
                    } else if let Some(repo_name) = &self.current_repository {
                        let content = format!(
                            "{}@{}",
                            self.ecr_repositories
                                .iter()
                                .find(|r| r.repository_name == *repo_name)
                                .map(|r| &r.repository_uri)
                                .unwrap_or(&format!(
                                    "unknown.dkr.ecr.region.amazonaws.com/{}",
                                    repo_name
                                )),
                            image.image_digest
                        );
                        let short_digest = if image.image_digest.len() > 36 {
                            format!("{}...", &image.image_digest[..36])
                        } else {
                            image.image_digest.clone()
                        };
                        let display_name = format!("{}@{}", repo_name, short_digest);
                        (content, display_name)
                    } else {
                        return;
                    }
                } else {
                    return;
                }
            }
        };

        if ctx.set_contents(content).is_ok() {
            self.copy_status = Some((format!("✓ {} copied", display_name), Instant::now()));
        }
    }
}
