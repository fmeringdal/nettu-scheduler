mod postgres;

use nettu_scheduler_domain::{UserIntegrationProvider, ID};
pub use postgres::PostgresServiceUseBusyCalendarRepo;

pub struct BusyCalendarIdentifier {
    service_id: ID,
    user_id: ID,
    calendar_id: ID,
}

pub struct ExternalBusyCalendarIdentifier {
    service_id: ID,
    user_id: ID,
    ext_calendar_id: String,
    provider: UserIntegrationProvider,
}

#[async_trait::async_trait]
pub trait IServiceUserBusyCalendarRepo: Send + Sync {
    async fn exists(&self, input: BusyCalendarIdentifier) -> anyhow::Result<bool>;
    async fn exists_ext(&self, input: ExternalBusyCalendarIdentifier) -> anyhow::Result<bool>;
    async fn insert(&self, input: BusyCalendarIdentifier) -> anyhow::Result<()>;
    async fn insert_ext(&self, input: ExternalBusyCalendarIdentifier) -> anyhow::Result<()>;
    async fn delete(&self, input: BusyCalendarIdentifier) -> anyhow::Result<()>;
    async fn delete_ext(&self, input: ExternalBusyCalendarIdentifier) -> anyhow::Result<()>;
}

#[cfg(test)]
mod tests {}
