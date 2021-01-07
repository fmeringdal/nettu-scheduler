mod inmemory;
mod mongo;

pub use inmemory::InMemoryEventRepo;
pub use mongo::EventRepo;

use crate::{calendar::domain::calendar_view::CalendarView, event::domain::event::CalendarEvent};
use std::error::Error;

pub struct DeleteResult {
    pub deleted_count: i64,
}

#[async_trait::async_trait]
pub trait IEventRepo: Send + Sync {
    async fn insert(&self, e: &CalendarEvent) -> Result<(), Box<dyn Error>>;
    async fn save(&self, e: &CalendarEvent) -> Result<(), Box<dyn Error>>;
    async fn find(&self, event_id: &str) -> Option<CalendarEvent>;
    async fn find_by_calendar(
        &self,
        calendar_id: &str,
        view: Option<&CalendarView>,
    ) -> Result<Vec<CalendarEvent>, Box<dyn Error>>;
    async fn delete(&self, event_id: &str) -> Option<CalendarEvent>;
    async fn delete_by_calendar(&self, calendar_id: &str) -> Result<DeleteResult, Box<dyn Error>>;
}
