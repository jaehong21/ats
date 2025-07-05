use anyhow::Result;
use async_trait::async_trait;
use ratatui::{Frame, layout::Rect};
use serde::{Deserialize, Serialize};
use std::any::Any;

use crate::app::App;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServiceId(pub String);

impl ServiceId {
    pub fn new(id: &str) -> Self {
        Self(id.to_string())
    }
}

impl std::fmt::Display for ServiceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub command: String,
}

pub trait ResourceItem: Send + Sync + std::fmt::Debug {
    fn id(&self) -> String;
    fn as_any(&self) -> &dyn Any;
    fn clone_box(&self) -> Box<dyn ResourceItem>;
}

impl Clone for Box<dyn ResourceItem> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Debug, Clone)]
pub struct ResourceData {
    pub items: Vec<Box<dyn ResourceItem>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewType {
    List,
    Detail,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct ViewState {
    pub service_id: ServiceId,
    pub view_type: ViewType,
    pub selected_index: usize,
    pub search_filter: String,
    pub context: Option<String>, // For drill-down context (e.g., repository name)
}

impl ViewState {
    pub fn new(service_id: ServiceId, view_type: ViewType) -> Self {
        Self {
            service_id,
            view_type,
            selected_index: 0,
            search_filter: String::new(),
            context: None,
        }
    }
}

#[async_trait]
pub trait AwsService: Send + Sync {
    fn metadata(&self) -> ServiceMetadata;

    async fn load_data(&self, view_state: &ViewState) -> Result<ResourceData>;

    fn render(
        &self,
        f: &mut Frame,
        area: Rect,
        app: &App,
        view_state: &ViewState,
        data: &ResourceData,
    );

    fn handle_enter(&self, view_state: &mut ViewState, data: &ResourceData) -> Option<ViewState>;

    fn get_copy_content(
        &self,
        view_state: &ViewState,
        data: &ResourceData,
    ) -> Option<(String, String)>;

    #[allow(clippy::borrowed_box)]
    fn filter_data<'a>(
        &self,
        data: &'a ResourceData,
        filter: &str,
    ) -> Vec<&'a Box<dyn ResourceItem>> {
        if filter.is_empty() {
            data.items.iter().collect()
        } else {
            data.items
                .iter()
                .filter(|item| self.matches_filter(item.as_ref(), filter))
                .collect()
        }
    }

    fn matches_filter(&self, _item: &dyn ResourceItem, _filter: &str) -> bool {
        true // Default implementation - override in service implementations
    }
}
