mod event;
mod event_reminders_expansion_jobs;
mod reminder;

use nettu_scheduler_core::{CalendarEvent, CalendarView, EventRemindersExpansionJob, Reminder};
use std::error::Error;

pub use event::EventRepo;
pub use event::InMemoryEventRepo;
pub use event_reminders_expansion_jobs::EventRemindersExpansionsJobRepo;
pub use event_reminders_expansion_jobs::InMemoryEventRemindersExpansionJobsRepo;
pub use reminder::InMemoryReminderRepo;
pub use reminder::ReminderRepo;

use crate::shared::repo::DeleteResult;

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
}

#[async_trait::async_trait]
pub trait IReminderRepo: Send + Sync {
    async fn bulk_insert(&self, reminders: &[Reminder]) -> Result<(), Box<dyn Error>>;
    async fn delete_all_before(&self, before: i64) -> Vec<Reminder>;
    async fn delete_by_events(&self, event_ids: &[String]) -> Result<DeleteResult, Box<dyn Error>>;
}

#[async_trait::async_trait]
pub trait IEventRemindersExpansionJobsRepo: Send + Sync {
    async fn bulk_insert(&self, job: &[EventRemindersExpansionJob]) -> Result<(), Box<dyn Error>>;
    async fn delete_all_before(&self, before: i64) -> Vec<EventRemindersExpansionJob>;
    async fn delete_by_event(&self, event_id: &str) -> Result<DeleteResult, Box<dyn Error>>;
}
