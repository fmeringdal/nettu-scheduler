mod postgres;

use nettu_scheduler_domain::{UserIntegration, UserIntegrationProvider};
pub use postgres::{PostgresUserIntegrationRepo, UserIntegrationRaw};

use nettu_scheduler_domain::ID;

#[async_trait::async_trait]
pub trait IUserIntegrationRepo: Send + Sync {
    async fn insert(&self, integration: &UserIntegration) -> anyhow::Result<()>;
    async fn save(&self, integration: &UserIntegration) -> anyhow::Result<()>;
    async fn find(&self, user_id: &ID) -> anyhow::Result<Vec<UserIntegration>>;
    async fn delete(&self, user_id: &ID, provider: UserIntegrationProvider) -> anyhow::Result<()>;
}

#[cfg(test)]
mod tests {}
