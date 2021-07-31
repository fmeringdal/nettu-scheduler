mod postgres;

use nettu_scheduler_domain::{SyncedCalendar, ID};
pub use postgres::PostgresCalendarSyncedRepo;

#[async_trait::async_trait]
pub trait ICalendarSyncedRepo: Send + Sync {
    async fn insert(&self, e: &SyncedCalendar) -> anyhow::Result<()>;
    async fn find_by_calendar(&self, calenadr_id: &ID) -> anyhow::Result<Vec<SyncedCalendar>>;
}

#[cfg(test)]
mod tests {}
