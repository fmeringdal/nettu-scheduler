mod inmemory;
mod mongo;

pub use inmemory::InMemoryEventRemindersExpansionJobsRepo;
pub use mongo::MongoEventRemindersExpansionsJobRepo;
use nettu_scheduler_domain::{EventRemindersExpansionJob, ID};

use crate::repos::shared::repo::DeleteResult;

#[async_trait::async_trait]
pub trait IEventRemindersExpansionJobsRepo: Send + Sync {
    async fn bulk_insert(&self, job: &[EventRemindersExpansionJob]) -> anyhow::Result<()>;
    async fn delete_all_before(&self, before: i64) -> Vec<EventRemindersExpansionJob>;
    async fn delete_by_event(&self, event_id: &ID) -> anyhow::Result<DeleteResult>;
}
