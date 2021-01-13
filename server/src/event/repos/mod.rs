mod event;
mod reminder;

pub use event::EventRepo;
pub use event::InMemoryEventRepo;
pub use reminder::ReminderRepo;
pub use reminder::InMemoryReminderRepo;

use crate::{calendar::domain::calendar_view::CalendarView, event::domain::event::CalendarEvent};
use std::error::Error;

use super::domain::Reminder;

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

#[async_trait::async_trait]
pub trait IReminderRepo: Send + Sync {
    async fn bulk_insert(&self, reminders: &[Reminder]) -> Result<(), Box<dyn Error>>;
    async fn delete_all_before(&self, before: i64) -> Vec<Reminder>;
    async fn delete_by_event(&self, event_id: &str) -> Result<DeleteResult, Box<dyn Error>>;
}
