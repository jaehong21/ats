use anyhow::Result;
use async_trait::async_trait;
use aws_sdk_ecr::{
    Client,
    types::{ImageDetail, Repository},
};
use chrono::{DateTime, Utc};
use ratatui::{
    Frame,
    layout::Constraint,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
};
use serde::{Deserialize, Serialize};
use std::any::Any;

use super::traits::{AwsService, ResourceData, ResourceItem, ServiceMetadata, ViewState, ViewType};
use crate::app::App;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ECRRepository {
    pub repository_name: String,
    pub repository_uri: String,
    pub registry_id: String,
    pub created_at: Option<DateTime<Utc>>,
    pub image_tag_mutability: String,
    pub image_scanning_configuration: bool,
    pub encryption_configuration: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ECRImage {
    pub image_tag: Option<String>,
    pub image_digest: String,
    pub image_pushed_at: Option<DateTime<Utc>>,
    pub image_size_in_bytes: Option<i64>,
    pub vulnerability_scan_summary: Option<String>,
}

impl ResourceItem for ECRRepository {
    fn id(&self) -> String {
        self.repository_name.clone()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn ResourceItem> {
        Box::new(self.clone())
    }
}

impl ResourceItem for ECRImage {
    fn id(&self) -> String {
        self.image_digest.clone()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn ResourceItem> {
        Box::new(self.clone())
    }
}

impl From<&Repository> for ECRRepository {
    fn from(repo: &Repository) -> Self {
        Self {
            repository_name: repo.repository_name().unwrap_or("").to_string(),
            repository_uri: repo.repository_uri().unwrap_or("").to_string(),
            registry_id: repo.registry_id().unwrap_or("").to_string(),
            created_at: repo.created_at().map(|dt| -> DateTime<Utc> {
                DateTime::from_timestamp(dt.secs(), dt.subsec_nanos()).unwrap_or_else(Utc::now)
            }),
            image_tag_mutability: repo
                .image_tag_mutability()
                .map(|itm| format!("{:?}", itm))
                .unwrap_or_else(|| "MUTABLE".to_string()),
            image_scanning_configuration: repo
                .image_scanning_configuration()
                .map(|isc| isc.scan_on_push())
                .unwrap_or(false),
            encryption_configuration: repo
                .encryption_configuration()
                .map(|ec| format!("{:?}", ec.encryption_type()))
                .unwrap_or_else(|| "AES256".to_string()),
        }
    }
}

impl From<&ImageDetail> for ECRImage {
    fn from(image: &ImageDetail) -> Self {
        let image_tag = image.image_tags().first().map(|tag| tag.to_string());

        let vulnerability_summary = image.image_scan_findings_summary().map(|summary| {
            if let Some(counts) = summary.finding_severity_counts() {
                let total: i32 = counts.values().sum();
                if total > 0 {
                    format!("{} findings", total)
                } else {
                    "No vulnerabilities".to_string()
                }
            } else {
                "Scan pending".to_string()
            }
        });

        Self {
            image_tag,
            image_digest: image.image_digest().unwrap_or("").to_string(),
            image_pushed_at: image.image_pushed_at().map(|dt| -> DateTime<Utc> {
                DateTime::from_timestamp(dt.secs(), dt.subsec_nanos()).unwrap_or_else(Utc::now)
            }),
            image_size_in_bytes: image.image_size_in_bytes(),
            vulnerability_scan_summary: vulnerability_summary,
        }
    }
}

pub struct ECRService {
    client: Client,
}

impl ECRService {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn list_repositories(&self) -> Result<Vec<ECRRepository>> {
        let resp = self.client.describe_repositories().send().await?;

        let repositories = resp
            .repositories()
            .iter()
            .map(ECRRepository::from)
            .collect();

        Ok(repositories)
    }

    pub async fn get_repository_images(&self, repository_name: &str) -> Result<Vec<ECRImage>> {
        let resp = self
            .client
            .describe_images()
            .repository_name(repository_name)
            .send()
            .await?;

        let mut images: Vec<ECRImage> = resp.image_details().iter().map(ECRImage::from).collect();

        // Sort images by pushed_at date, latest first
        images.sort_by(|a, b| {
            match (a.image_pushed_at, b.image_pushed_at) {
                (Some(a_date), Some(b_date)) => b_date.cmp(&a_date), // Latest first
                (Some(_), None) => std::cmp::Ordering::Less,         // Images with dates come first
                (None, Some(_)) => std::cmp::Ordering::Greater, // Images without dates come last
                (None, None) => std::cmp::Ordering::Equal,      // Both have no date
            }
        });

        Ok(images)
    }
}

#[async_trait]
impl AwsService for ECRService {
    fn metadata(&self) -> ServiceMetadata {
        ServiceMetadata {
            id: "ecr".to_string(),
            name: "Elastic Container Registry".to_string(),
            description: "AWS Container Registry for Docker images".to_string(),
            command: "ecr".to_string(),
        }
    }

    async fn load_data(&self, view_state: &ViewState) -> Result<ResourceData> {
        match view_state.view_type {
            ViewType::List => {
                let repositories = self.list_repositories().await?;
                Ok(ResourceData {
                    items: repositories
                        .into_iter()
                        .map(|repo| Box::new(repo) as Box<dyn ResourceItem>)
                        .collect(),
                })
            }
            ViewType::Detail => {
                if let Some(context) = &view_state.context {
                    // Extract repository name from "name|uri" format
                    let repo_name = context.split('|').next().unwrap_or(context);
                    let images = self.get_repository_images(repo_name).await?;
                    Ok(ResourceData {
                        items: images
                            .into_iter()
                            .map(|img| Box::new(img) as Box<dyn ResourceItem>)
                            .collect(),
                    })
                } else {
                    Ok(ResourceData { items: Vec::new() })
                }
            }
            ViewType::Custom(_) => Ok(ResourceData { items: Vec::new() }),
        }
    }

    fn render(
        &self,
        f: &mut Frame,
        area: Rect,
        app: &App,
        view_state: &ViewState,
        data: &ResourceData,
    ) {
        match view_state.view_type {
            ViewType::List => self.render_repositories(f, area, app, view_state, data),
            ViewType::Detail => self.render_images(f, area, app, view_state, data),
            ViewType::Custom(_) => {}
        }
    }

    fn handle_enter(&self, view_state: &mut ViewState, data: &ResourceData) -> Option<ViewState> {
        match view_state.view_type {
            ViewType::List => {
                let filtered_items = self.filter_data(data, &view_state.search_filter);
                if view_state.selected_index < filtered_items.len() {
                    if let Some(repo) = filtered_items[view_state.selected_index]
                        .as_any()
                        .downcast_ref::<ECRRepository>()
                    {
                        let mut new_view =
                            ViewState::new(view_state.service_id.clone(), ViewType::Detail);
                        // Store both repository name and URI separated by "|"
                        new_view.context =
                            Some(format!("{}|{}", repo.repository_name, repo.repository_uri));
                        return Some(new_view);
                    }
                }
                None
            }
            ViewType::Detail => None, // Future: handle image drill-down
            ViewType::Custom(_) => None,
        }
    }

    fn get_copy_content(
        &self,
        view_state: &ViewState,
        data: &ResourceData,
    ) -> Option<(String, String)> {
        let filtered_items = self.filter_data(data, &view_state.search_filter);
        if view_state.selected_index >= filtered_items.len() {
            return None;
        }

        match view_state.view_type {
            ViewType::List => {
                if let Some(repo) = filtered_items[view_state.selected_index]
                    .as_any()
                    .downcast_ref::<ECRRepository>()
                {
                    let content = repo.repository_uri.clone();
                    let display_name = repo.repository_name.clone();
                    Some((content, display_name))
                } else {
                    None
                }
            }
            ViewType::Detail => {
                if let Some(image) = filtered_items[view_state.selected_index]
                    .as_any()
                    .downcast_ref::<ECRImage>()
                {
                    if let Some(context) = &view_state.context {
                        // Parse "repo_name|repo_uri" format
                        let parts: Vec<&str> = context.split('|').collect();
                        let repo_name = parts.get(0).unwrap_or(&"unknown");
                        let repo_uri = parts
                            .get(1)
                            .unwrap_or(&"unknown.dkr.ecr.region.amazonaws.com");

                        if let Some(tag) = &image.image_tag {
                            let content = format!("{}:{}", repo_uri, tag);
                            let display_name = format!("{}:{}", repo_name, tag);
                            Some((content, display_name))
                        } else {
                            let short_digest = if image.image_digest.len() > 36 {
                                format!("{}...", &image.image_digest[..36])
                            } else {
                                image.image_digest.clone()
                            };
                            let content = format!("{}@{}", repo_uri, image.image_digest);
                            let display_name = format!("{}@{}", repo_name, short_digest);
                            Some((content, display_name))
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            ViewType::Custom(_) => None,
        }
    }

    fn matches_filter(&self, item: &dyn ResourceItem, filter: &str) -> bool {
        if let Some(repo) = item.as_any().downcast_ref::<ECRRepository>() {
            repo.repository_name
                .to_lowercase()
                .contains(&filter.to_lowercase())
        } else if let Some(image) = item.as_any().downcast_ref::<ECRImage>() {
            image
                .image_tag
                .as_ref()
                .map(|tag| tag.to_lowercase().contains(&filter.to_lowercase()))
                .unwrap_or(false)
        } else {
            false
        }
    }
}

impl ECRService {
    fn render_repositories(
        &self,
        f: &mut Frame,
        area: Rect,
        app: &App,
        view_state: &ViewState,
        data: &ResourceData,
    ) {
        let filtered_items = self.filter_data(data, &view_state.search_filter);

        let title = if app.loading {
            "ECR Repositories (Loading...)".to_string()
        } else if view_state.search_filter.is_empty() {
            format!("ECR Repositories ({})", filtered_items.len())
        } else {
            format!(
                "ECR Repositories ({}/{}) - Filter: {}",
                filtered_items.len(),
                data.items.len(),
                view_state.search_filter
            )
        };

        if filtered_items.is_empty() {
            let message = if app.loading {
                "Loading ECR repositories..."
            } else if !view_state.search_filter.is_empty() {
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

        let rows: Vec<Row> = filtered_items
            .iter()
            .enumerate()
            .filter_map(|(i, item)| {
                if let Some(repo) = item.as_any().downcast_ref::<ECRRepository>() {
                    let created_str = repo
                        .created_at
                        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                        .unwrap_or_else(|| "Unknown".to_string());

                    let scan_on_push = if repo.image_scanning_configuration {
                        "Yes"
                    } else {
                        "No"
                    };

                    let style = if i == view_state.selected_index {
                        Style::default().bg(Color::Yellow).fg(Color::Black)
                    } else {
                        Style::default()
                    };

                    Some(
                        Row::new(vec![
                            Cell::from(repo.repository_name.clone()),
                            Cell::from(repo.registry_id.clone()),
                            Cell::from(created_str),
                            Cell::from(repo.image_tag_mutability.clone()),
                            Cell::from(scan_on_push),
                            Cell::from(repo.encryption_configuration.clone()),
                        ])
                        .style(style),
                    )
                } else {
                    None
                }
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

    fn render_images(
        &self,
        f: &mut Frame,
        area: Rect,
        app: &App,
        view_state: &ViewState,
        data: &ResourceData,
    ) {
        let filtered_items = self.filter_data(data, &view_state.search_filter);
        let default_repo = "Unknown".to_string();
        let repo_name = view_state
            .context
            .as_ref()
            .map(|context| context.split('|').next().unwrap_or(context))
            .unwrap_or(&default_repo);

        let title = if app.loading {
            format!("ECR Repositories: {} > ECR Images (Loading...)", repo_name)
        } else if view_state.search_filter.is_empty() {
            format!(
                "ECR Repositories: {} > ECR Images ({})",
                repo_name,
                filtered_items.len()
            )
        } else {
            format!(
                "ECR Repositories: {} > ECR Images ({}/{}) - Filter: {}",
                repo_name,
                filtered_items.len(),
                data.items.len(),
                view_state.search_filter
            )
        };

        if filtered_items.is_empty() {
            let message = if app.loading {
                "Loading ECR images..."
            } else if !view_state.search_filter.is_empty() {
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

        let rows: Vec<Row> = filtered_items
            .iter()
            .enumerate()
            .filter_map(|(i, item)| {
                if let Some(image) = item.as_any().downcast_ref::<ECRImage>() {
                    let tag = image
                        .image_tag
                        .as_ref()
                        .unwrap_or(&"<none>".to_string())
                        .clone();
                    let digest = image.image_digest.to_string();
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

                    let style = if i == view_state.selected_index {
                        Style::default().bg(Color::Yellow).fg(Color::Black)
                    } else {
                        Style::default()
                    };

                    Some(
                        Row::new(vec![
                            Cell::from(tag),
                            Cell::from(digest),
                            Cell::from(pushed_at),
                            Cell::from(size),
                            Cell::from(vulnerabilities),
                        ])
                        .style(style),
                    )
                } else {
                    None
                }
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
}
