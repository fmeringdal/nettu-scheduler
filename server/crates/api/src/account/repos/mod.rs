use crate::account::domain::Account;
mod inmemory;
mod mongo;
use std::error::Error;

pub use inmemory::InMemoryAccountRepo;
pub use mongo::AccountRepo;

#[async_trait::async_trait]
pub trait IAccountRepo: Send + Sync {
    async fn insert(&self, account: &Account) -> Result<(), Box<dyn Error>>;
    async fn save(&self, account: &Account) -> Result<(), Box<dyn Error>>;
    async fn find(&self, account_id: &str) -> Option<Account>;
    async fn find_many(&self, account_ids: &[String]) -> Result<Vec<Account>, Box<dyn Error>>;
    async fn delete(&self, account_id: &str) -> Option<Account>;
    async fn find_by_apikey(&self, api_key: &str) -> Option<Account>;
    async fn find_by_webhook_url(&self, url: &str) -> Option<Account>;
}
