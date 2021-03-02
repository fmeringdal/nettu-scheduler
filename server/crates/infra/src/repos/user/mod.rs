mod inmemory;
mod mongo;

pub use inmemory::InMemoryUserRepo;
pub use mongo::MongoUserRepo;
use nettu_scheduler_domain::User;

#[async_trait::async_trait]
pub trait IUserRepo: Send + Sync {
    async fn insert(&self, user: &User) -> anyhow::Result<()>;
    async fn save(&self, user: &User) -> anyhow::Result<()>;
    async fn delete(&self, user_id: &str) -> Option<User>;
    async fn find(&self, user_id: &str) -> Option<User>;
    async fn find_by_account_id(&self, user_id: &str, account_id: &str) -> Option<User>;
}
