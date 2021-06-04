use super::IAccountRepo;
use crate::repos::shared::inmemory_repo::*;
use nettu_scheduler_domain::{Account, ID};

pub struct InMemoryAccountRepo {
    accounts: std::sync::Mutex<Vec<Account>>,
}

impl InMemoryAccountRepo {
    pub fn new() -> Self {
        Self {
            accounts: std::sync::Mutex::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl IAccountRepo for InMemoryAccountRepo {
    async fn insert(&self, account: &Account) -> anyhow::Result<()> {
        insert(account, &self.accounts);
        Ok(())
    }

    async fn save(&self, account: &Account) -> anyhow::Result<()> {
        save(account, &self.accounts);
        Ok(())
    }

    async fn find(&self, account_id: &ID) -> Option<Account> {
        find(account_id, &self.accounts)
    }

    async fn find_many(&self, account_ids: &[ID]) -> anyhow::Result<Vec<Account>> {
        let res = find_by(&self.accounts, |a| account_ids.contains(&a.id));
        Ok(res)
    }

    async fn delete(&self, account_id: &ID) -> Option<Account> {
        delete(account_id, &self.accounts)
    }

    async fn find_by_apikey(&self, api_key: &str) -> Option<Account> {
        let accounts = find_by(&self.accounts, |account| account.secret_api_key == api_key);
        if accounts.is_empty() {
            return None;
        }
        Some(accounts[0].clone())
    }
}
