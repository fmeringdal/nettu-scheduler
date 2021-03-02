mod inmemory;
mod mongo;

pub use inmemory::InMemoryServiceRepo;
pub use mongo::MongoServiceRepo;
use nettu_scheduler_domain::Service;

#[async_trait::async_trait]
pub trait IServiceRepo: Send + Sync {
    async fn insert(&self, service: &Service) -> anyhow::Result<()>;
    async fn save(&self, service: &Service) -> anyhow::Result<()>;
    async fn find(&self, service_id: &str) -> Option<Service>;
    async fn delete(&self, service_id: &str) -> Option<Service>;
    async fn remove_calendar_from_services(&self, calendar_id: &str) -> anyhow::Result<()>;
    async fn remove_schedule_from_services(&self, schedule_id: &str) -> anyhow::Result<()>;
    async fn remove_user_from_services(&self, user_id: &str) -> anyhow::Result<()>;
}
