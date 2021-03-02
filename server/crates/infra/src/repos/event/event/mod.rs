mod inmemory;
mod mongo;

use crate::repos::shared::repo::DeleteResult;
pub use inmemory::InMemoryEventRepo;
pub use mongo::MongoEventRepo;
use nettu_scheduler_domain::{CalendarEvent, TimeSpan};

#[async_trait::async_trait]
pub trait IEventRepo: Send + Sync {
    async fn insert(&self, e: &CalendarEvent) -> anyhow::Result<()>;
    async fn save(&self, e: &CalendarEvent) -> anyhow::Result<()>;
    async fn find(&self, event_id: &str) -> Option<CalendarEvent>;
    async fn find_many(&self, event_ids: &[String]) -> anyhow::Result<Vec<CalendarEvent>>;
    async fn find_by_calendar(
        &self,
        calendar_id: &str,
        timespan: Option<&TimeSpan>,
    ) -> anyhow::Result<Vec<CalendarEvent>>;
    async fn delete(&self, event_id: &str) -> Option<CalendarEvent>;
    async fn delete_by_calendar(&self, calendar_id: &str) -> anyhow::Result<DeleteResult>;
    async fn delete_by_user(&self, user_id: &str) -> anyhow::Result<DeleteResult>;
}
