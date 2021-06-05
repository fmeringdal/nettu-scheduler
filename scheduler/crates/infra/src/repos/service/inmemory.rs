use super::IServiceRepo;
use crate::repos::shared::{inmemory_repo::*, query_structs::MetadataFindQuery};
use nettu_scheduler_domain::{Service, ID};

pub struct InMemoryServiceRepo {
    services: std::sync::Mutex<Vec<Service>>,
}

impl InMemoryServiceRepo {
    pub fn new() -> Self {
        Self {
            services: std::sync::Mutex::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl IServiceRepo for InMemoryServiceRepo {
    async fn insert(&self, service: &Service) -> anyhow::Result<()> {
        insert(service, &self.services);
        Ok(())
    }

    async fn save(&self, service: &Service) -> anyhow::Result<()> {
        save(service, &self.services);
        Ok(())
    }

    async fn find(&self, service_id: &ID) -> Option<Service> {
        find(service_id, &self.services)
    }

    async fn delete(&self, service_id: &ID) -> anyhow::Result<()> {
        delete(service_id, &self.services);
        Ok(())
    }

    async fn find_by_metadata(&self, query: MetadataFindQuery) -> Vec<Service> {
        find_by_metadata(&self.services, query)
    }
}
