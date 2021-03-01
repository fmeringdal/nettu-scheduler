mod inmemory;
mod mongo;

use std::error::Error;

pub use inmemory::InMemoryReminderRepo;
pub use mongo::ReminderRepo;
use nettu_scheduler_core::Reminder;

use crate::shared::repo::DeleteResult;

#[async_trait::async_trait]
pub trait IReminderRepo: Send + Sync {
    async fn bulk_insert(&self, reminders: &[Reminder]) -> Result<(), Box<dyn Error>>;
    async fn find_by_event_and_priority(&self, event_id: &str, priority: i64) -> Option<Reminder>;
    async fn delete_all_before(&self, before: i64) -> Vec<Reminder>;
    async fn delete_by_events(&self, event_ids: &[String]) -> Result<DeleteResult, Box<dyn Error>>;
}
