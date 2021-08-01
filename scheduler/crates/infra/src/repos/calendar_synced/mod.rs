mod postgres;

use nettu_scheduler_domain::{SyncedCalendar, ID};
pub use postgres::PostgresCalendarSyncedRepo;

#[async_trait::async_trait]
pub trait ICalendarSyncedRepo: Send + Sync {
    async fn insert(&self, c: &SyncedCalendar) -> anyhow::Result<()>;
    async fn delete(&self, c: &SyncedCalendar) -> anyhow::Result<()>;
    async fn find_by_calendar(&self, calendar_id: &ID) -> anyhow::Result<Vec<SyncedCalendar>>;
}

#[cfg(test)]
mod tests {}
