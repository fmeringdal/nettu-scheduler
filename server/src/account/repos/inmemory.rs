use super::IAccountRepo;
use crate::account::domain::Account;
use crate::shared::inmemory_repo::*;
use std::error::Error;

pub struct InMemoryAccountRepo {
    accounts: std::sync::Mutex<Vec<Account>>,
}

impl InMemoryAccountRepo {
    pub fn new() -> Self {
        Self {
            accounts: std::sync::Mutex::new(vec![]),
        }
    }
}

#[async_trait::async_trait]
impl IAccountRepo for InMemoryAccountRepo {
    async fn insert(&self, account: &Account) -> Result<(), Box<dyn Error>> {
        insert(account, &self.accounts);
        Ok(())
    }

    async fn save(&self, account: &Account) -> Result<(), Box<dyn Error>> {
        save(account, &self.accounts);
        Ok(())
    }

    async fn find(&self, account_id: &str) -> Option<Account> {
        find(account_id, &self.accounts)
    }

    async fn delete(&self, account_id: &str) -> Option<Account> {
        delete(account_id, &self.accounts)
    }

    async fn find_many(&self, account_ids: &[String]) -> Result<Vec<Account>, Box<dyn Error>> {
        let res = find_by(&self.accounts, |a| account_ids.contains(&a.id));
        Ok(res)
    }

    async fn find_by_apikey(&self, api_key: &str) -> Option<Account> {
        let accounts = find_by(&self.accounts, |account| account.secret_api_key == api_key);
        if accounts.is_empty() {
            return None;
        }
        Some(accounts[0].clone())
    }

    async fn find_by_webhook_url(&self, url: &str) -> Option<Account> {
        let accounts = find_by(&self.accounts, |account| {
            match &account.settings.webhook {
                Some(webhook_settings) => webhook_settings.url == url.to_string(),
                None => false
            }
        });
        if accounts.is_empty() {
            return None;
        }
        Some(accounts[0].clone())
    }
}
