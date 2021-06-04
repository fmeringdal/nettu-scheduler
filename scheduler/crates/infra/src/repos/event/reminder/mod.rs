mod inmemory;
mod mongo;
mod postgres;

pub use inmemory::InMemoryReminderRepo;
pub use mongo::MongoReminderRepo;
use nettu_scheduler_domain::{Reminder, ID};

use crate::repos::shared::repo::DeleteResult;

#[async_trait::async_trait]
pub trait IReminderRepo: Send + Sync {
    async fn bulk_insert(&self, reminders: &[Reminder]) -> anyhow::Result<()>;
    async fn find_by_event_and_priority(&self, event_id: &ID, priority: i64) -> Option<Reminder>;
    async fn delete_all_before(&self, before: i64) -> Vec<Reminder>;
    async fn delete_by_events(&self, event_ids: &[ID]) -> anyhow::Result<DeleteResult>;
}
