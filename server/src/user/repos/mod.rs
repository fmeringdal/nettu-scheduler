mod inmemory_user_repo;
mod mongo_user_repo;

use crate::user::domain::User;
pub use inmemory_user_repo::InMemoryUserRepo;
pub use mongo_user_repo::UserRepo;

use std::error::Error;

#[async_trait::async_trait]
pub trait IUserRepo: Send + Sync {
    async fn insert(&self, user: &User) -> Result<(), Box<dyn Error>>;
    async fn save(&self, user: &User) -> Result<(), Box<dyn Error>>;
    async fn delete(&self, user_id: &str) -> Option<User>;
    async fn find(&self, user_id: &str) -> Option<User>;
}
