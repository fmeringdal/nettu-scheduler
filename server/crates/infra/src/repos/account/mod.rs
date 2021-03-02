mod inmemory;
mod mongo;

use std::error::Error;

pub use inmemory::InMemoryAccountRepo;
pub use mongo::MongoAccountRepo;
use nettu_scheduler_domain::Account;

#[async_trait::async_trait]
pub trait IAccountRepo: Send + Sync {
    async fn insert(&self, account: &Account) -> anyhow::Result<()>;
    async fn save(&self, account: &Account) -> anyhow::Result<()>;
    async fn find(&self, account_id: &str) -> Option<Account>;
    async fn find_many(&self, account_ids: &[String]) -> anyhow::Result<Vec<Account>>;
    async fn delete(&self, account_id: &str) -> Option<Account>;
    async fn find_by_apikey(&self, api_key: &str) -> Option<Account>;
    async fn find_by_webhook_url(&self, url: &str) -> Option<Account>;
}

#[cfg(test)]
mod tests {
    use crate::setup_context;
    use nettu_scheduler_domain::Entity;

    #[tokio::test]
    async fn create_and_delete() {
        let ctx = setup_context().await;
        let account = Default::default();

        // Insert
        assert!(ctx.repos.account_repo.insert(&account).await.is_ok());

        // Different find methods
        let res = ctx.repos.account_repo.find(&account.id).await.unwrap();
        assert!(res.eq(&account));
        let res = ctx
            .repos
            .account_repo
            .find_many(&[account.id.clone()])
            .await
            .unwrap();
        assert!(res[0].eq(&account));
        let res = ctx
            .repos
            .account_repo
            .find_by_apikey(&account.secret_api_key)
            .await
            .unwrap();
        assert!(res.eq(&account));

        // Delete
        let res = ctx.repos.account_repo.delete(&account.id).await;
        assert!(res.is_some());
        assert!(res.unwrap().eq(&account));

        // Find
        assert!(ctx.repos.account_repo.find(&account.id).await.is_none());
    }

    #[tokio::test]
    async fn update() {
        let ctx = setup_context().await;
        let mut account = Default::default();

        // Insert
        assert!(ctx.repos.account_repo.insert(&account).await.is_ok());

        let pubkey = String::from("12312412");
        account.set_public_key_b64(Some(pubkey.clone())).unwrap();

        // Save
        assert!(ctx.repos.account_repo.save(&account).await.is_ok());

        // Find
        assert!(ctx
            .repos
            .account_repo
            .find(&account.id)
            .await
            .unwrap()
            .eq(&account));
    }
}
