mod postgres;

use nettu_scheduler_domain::{ServiceResource, ID};
pub use postgres::PostgresServiceUserRepo;
pub use postgres::ServiceUserRaw;

#[async_trait::async_trait]
pub trait IServiceUserRepo: Send + Sync {
    async fn insert(&self, user: &ServiceResource) -> anyhow::Result<()>;
    async fn save(&self, user: &ServiceResource) -> anyhow::Result<()>;
    async fn find(&self, service_id: &ID, user_id: &ID) -> Option<ServiceResource>;
    async fn delete(&self, service_id: &ID, user_uid: &ID) -> anyhow::Result<()>;
}
