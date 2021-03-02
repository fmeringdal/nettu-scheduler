mod inmemory;
mod mongo;

use std::error::Error;

pub use inmemory::InMemoryReminderRepo;
pub use mongo::MongoReminderRepo;
use nettu_scheduler_domain::Reminder;

use crate::repos::shared::repo::DeleteResult;

#[async_trait::async_trait]
pub trait IReminderRepo: Send + Sync {
    async fn bulk_insert(&self, reminders: &[Reminder]) -> anyhow::Result<()>;
    async fn find_by_event_and_priority(&self, event_id: &str, priority: i64) -> Option<Reminder>;
    async fn delete_all_before(&self, before: i64) -> Vec<Reminder>;
    async fn delete_by_events(&self, event_ids: &[String]) -> anyhow::Result<DeleteResult>;
}
