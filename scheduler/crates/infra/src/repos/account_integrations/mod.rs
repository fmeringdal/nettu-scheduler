mod postgres;

use nettu_scheduler_domain::{AccountIntegration, IntegrationProvider};
pub use postgres::PostgresAccountIntegrationRepo;

use nettu_scheduler_domain::ID;

#[async_trait::async_trait]
pub trait IAccountIntegrationRepo: Send + Sync {
    async fn insert(&self, integration: &AccountIntegration) -> anyhow::Result<()>;
    async fn find(&self, account_id: &ID) -> anyhow::Result<Vec<AccountIntegration>>;
    async fn delete(&self, account_id: &ID, provider: IntegrationProvider) -> anyhow::Result<()>;
}

#[cfg(test)]
mod tests {}
