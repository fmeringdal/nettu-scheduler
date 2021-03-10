mod inmemory;
mod mongo;

pub use inmemory::InMemoryServiceRepo;
pub use mongo::MongoServiceRepo;
use nettu_scheduler_domain::{Service, ID};

use super::shared::query_structs::MetadataFindQuery;

#[async_trait::async_trait]
pub trait IServiceRepo: Send + Sync {
    async fn insert(&self, service: &Service) -> anyhow::Result<()>;
    async fn save(&self, service: &Service) -> anyhow::Result<()>;
    async fn find(&self, service_id: &ID) -> Option<Service>;
    async fn delete(&self, service_id: &ID) -> Option<Service>;
    async fn remove_calendar_from_services(&self, calendar_id: &ID) -> anyhow::Result<()>;
    async fn remove_schedule_from_services(&self, schedule_id: &ID) -> anyhow::Result<()>;
    async fn remove_user_from_services(&self, user_id: &ID) -> anyhow::Result<()>;
    async fn find_by_metadata(&self, query: MetadataFindQuery) -> Vec<Service>;
}
