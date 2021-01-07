use crate::account::domain::Account;
mod inmemory_account_repo;
mod mongo_account_repo;
use std::error::Error;

pub use inmemory_account_repo::InMemoryAccountRepo;
pub use mongo_account_repo::AccountRepo;

#[async_trait::async_trait]
pub trait IAccountRepo: Send + Sync {
    async fn insert(&self, account: &Account) -> Result<(), Box<dyn Error>>;
    async fn save(&self, account: &Account) -> Result<(), Box<dyn Error>>;
    async fn find(&self, account_id: &str) -> Option<Account>;
    async fn delete(&self, account_id: &str) -> Option<Account>;
    async fn find_by_apikey(&self, api_key: &str) -> Option<Account>;
}
