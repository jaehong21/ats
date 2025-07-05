use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

use super::traits::{AwsService, ResourceData, ServiceId, ServiceMetadata, ViewState};

pub struct ServiceManager {
    services: HashMap<ServiceId, Arc<dyn AwsService>>,
    service_data: HashMap<ServiceId, ResourceData>,
}

impl ServiceManager {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
            service_data: HashMap::new(),
        }
    }

    pub fn register_service(&mut self, service: Arc<dyn AwsService>) {
        let metadata = service.metadata();
        let service_id = ServiceId::new(&metadata.id);
        self.services.insert(service_id, service);
    }

    pub fn get_service(&self, service_id: &ServiceId) -> Option<&Arc<dyn AwsService>> {
        self.services.get(service_id)
    }

    pub fn get_service_metadata(&self) -> Vec<ServiceMetadata> {
        self.services
            .values()
            .map(|service| service.metadata())
            .collect()
    }

    pub fn get_service_by_command(
        &self,
        command: &str,
    ) -> Option<(&ServiceId, &Arc<dyn AwsService>)> {
        self.services
            .iter()
            .find(|(_, service)| service.metadata().command == command)
    }

    pub async fn load_service_data(
        &mut self,
        service_id: &ServiceId,
        view_state: &ViewState,
    ) -> Result<()> {
        if let Some(service) = self.services.get(service_id) {
            let data = service.load_data(view_state).await?;
            self.service_data.insert(service_id.clone(), data);
        }
        Ok(())
    }

    pub fn get_service_data(&self, service_id: &ServiceId) -> Option<&ResourceData> {
        self.service_data.get(service_id)
    }

    pub fn clear_service_data(&mut self, service_id: &ServiceId) {
        self.service_data.remove(service_id);
    }

    pub fn has_service(&self, service_id: &ServiceId) -> bool {
        self.services.contains_key(service_id)
    }

    pub fn list_services(&self) -> Vec<ServiceId> {
        self.services.keys().cloned().collect()
    }
}

impl Default for ServiceManager {
    fn default() -> Self {
        Self::new()
    }
}
