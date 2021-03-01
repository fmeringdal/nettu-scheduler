mod inmemory;
mod mongo;

use crate::repos::shared::repo::DeleteResult;
pub use inmemory::InMemoryEventRepo;
pub use mongo::MongoEventRepo;
use nettu_scheduler_core::{CalendarEvent, CalendarView};
use std::error::Error;

#[async_trait::async_trait]
pub trait IEventRepo: Send + Sync {
    async fn insert(&self, e: &CalendarEvent) -> Result<(), Box<dyn Error>>;
    async fn save(&self, e: &CalendarEvent) -> Result<(), Box<dyn Error>>;
    async fn find(&self, event_id: &str) -> Option<CalendarEvent>;
    async fn find_many(&self, event_ids: &[String]) -> Result<Vec<CalendarEvent>, Box<dyn Error>>;
    async fn find_by_calendar(
        &self,
        calendar_id: &str,
        view: Option<&CalendarView>,
    ) -> Result<Vec<CalendarEvent>, Box<dyn Error>>;
    async fn delete(&self, event_id: &str) -> Option<CalendarEvent>;
    async fn delete_by_calendar(&self, calendar_id: &str) -> Result<DeleteResult, Box<dyn Error>>;
    async fn delete_by_user(&self, user_id: &str) -> anyhow::Result<DeleteResult>;
}
