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

    async fn find_by_apikey(&self, api_key: &str) -> Option<Account> {
        let accounts = self.accounts.lock().unwrap();
        for i in 0..accounts.len() {
            if accounts[i].secret_api_key == api_key {
                return Some(accounts[i].clone());
            }
        }
        None
    }
}
