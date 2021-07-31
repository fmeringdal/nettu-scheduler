mod postgres;

use nettu_scheduler_domain::{SyncedCalendarEvent, ID};
pub use postgres::PostgresEventSyncedRepo;

#[async_trait::async_trait]
pub trait IEventSyncedRepo: Send + Sync {
    async fn insert(&self, e: &SyncedCalendarEvent) -> anyhow::Result<()>;
    async fn find_by_event(&self, event_id: &ID) -> anyhow::Result<Vec<SyncedCalendarEvent>>;
}

#[cfg(test)]
mod tests {}
