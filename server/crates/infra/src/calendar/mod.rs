mod inmemory;
mod mongo;

pub use inmemory::InMemoryCalendarRepo;
pub use mongo::CalendarRepo;
use nettu_scheduler_core::Calendar;

use std::error::Error;

#[async_trait::async_trait]
pub trait ICalendarRepo: Send + Sync {
    async fn insert(&self, calendar: &Calendar) -> Result<(), Box<dyn Error>>;
    async fn save(&self, calendar: &Calendar) -> Result<(), Box<dyn Error>>;
    async fn find(&self, calendar_id: &str) -> Option<Calendar>;
    async fn find_by_user(&self, user_id: &str) -> Vec<Calendar>;
    async fn delete(&self, calendar_id: &str) -> Option<Calendar>;
}
