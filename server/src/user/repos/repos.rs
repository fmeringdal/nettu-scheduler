use crate::user::domain::User;

use std::error::Error;

#[async_trait::async_trait]
pub trait IUserRepo: Send + Sync {
    async fn insert(&self, user: &User) -> Result<(), Box<dyn Error>>;
    async fn save(&self, user: &User) -> Result<(), Box<dyn Error>>;
    async fn delete(&self, user_id: &str) -> Option<User>;
    async fn find(&self, external_id: &str, company_id: &str) -> Option<User>;
}
