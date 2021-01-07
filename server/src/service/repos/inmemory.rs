use super::IServiceRepo;
use crate::service::domain::Service;
use crate::shared::inmemory_repo::*;
use std::error::Error;

pub struct InMemoryServiceRepo {
    services: std::sync::Mutex<Vec<Service>>,
}

impl InMemoryServiceRepo {
    pub fn new() -> Self {
        Self {
            services: std::sync::Mutex::new(vec![]),
        }
    }
}

#[async_trait::async_trait]
impl IServiceRepo for InMemoryServiceRepo {
    async fn insert(&self, service: &Service) -> Result<(), Box<dyn Error>> {
        insert(service, &self.services);
        Ok(())
    }

    async fn save(&self, service: &Service) -> Result<(), Box<dyn Error>> {
        save(service, &self.services);
        Ok(())
    }

    async fn find(&self, service_id: &str) -> Option<Service> {
        find(service_id, &self.services)
    }

    async fn delete(&self, service_id: &str) -> Option<Service> {
        delete(service_id, &self.services)
    }
}
