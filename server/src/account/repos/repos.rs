use crate::account::domain::Account;

use std::error::Error;

#[async_trait::async_trait]
pub trait IAccountRepo: Send + Sync {
    async fn insert(&self, account: &Account) -> Result<(), Box<dyn Error>>;
    async fn save(&self, account: &Account) -> Result<(), Box<dyn Error>>;
    async fn find(&self, account_id: &str) -> Option<Account>;
    async fn delete(&self, account_id: &str) -> Option<Account>;
    async fn find_by_apikey(&self, api_key: &str) -> Option<Account>;
}
