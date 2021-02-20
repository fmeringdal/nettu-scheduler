mod inmemory;
mod mongo;

use std::error::Error;

pub use inmemory::InMemoryServiceRepo;
pub use mongo::ServiceRepo;
use nettu_scheduler_core::domain::Service;

#[async_trait::async_trait]
pub trait IServiceRepo: Send + Sync {
    async fn insert(&self, service: &Service) -> Result<(), Box<dyn Error>>;
    async fn save(&self, service: &Service) -> Result<(), Box<dyn Error>>;
    async fn find(&self, service_id: &str) -> Option<Service>;
    async fn delete(&self, service_id: &str) -> Option<Service>;
    async fn remove_calendar_from_services(&self, calendar_id: &str) -> Result<(), Box<dyn Error>>;
    async fn remove_schedule_from_services(&self, schedule_id: &str) -> Result<(), Box<dyn Error>>;
}
