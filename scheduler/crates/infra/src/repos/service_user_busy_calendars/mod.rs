mod postgres;

use nettu_scheduler_domain::{BusyCalendar, IntegrationProvider, ID};
pub use postgres::PostgresServiceUseBusyCalendarRepo;

pub struct BusyCalendarIdentifier {
    pub service_id: ID,
    pub user_id: ID,
    pub calendar_id: ID,
}

pub struct ExternalBusyCalendarIdentifier {
    pub service_id: ID,
    pub user_id: ID,
    pub ext_calendar_id: String,
    pub provider: IntegrationProvider,
}

#[async_trait::async_trait]
pub trait IServiceUserBusyCalendarRepo: Send + Sync {
    async fn exists(&self, input: BusyCalendarIdentifier) -> anyhow::Result<bool>;
    async fn exists_ext(&self, input: ExternalBusyCalendarIdentifier) -> anyhow::Result<bool>;
    async fn insert(&self, input: BusyCalendarIdentifier) -> anyhow::Result<()>;
    async fn insert_ext(&self, input: ExternalBusyCalendarIdentifier) -> anyhow::Result<()>;
    async fn delete(&self, input: BusyCalendarIdentifier) -> anyhow::Result<()>;
    async fn delete_ext(&self, input: ExternalBusyCalendarIdentifier) -> anyhow::Result<()>;
    async fn find(&self, service_id: &ID, user_id: &ID) -> anyhow::Result<Vec<BusyCalendar>>;
}

#[cfg(test)]
mod tests {}
