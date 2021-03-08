mod inmemory;
mod mongo;

pub use inmemory::InMemoryAccountRepo;
pub use mongo::MongoAccountRepo;
use nettu_scheduler_domain::Account;

use nettu_scheduler_domain::ID;

#[async_trait::async_trait]
pub trait IAccountRepo: Send + Sync {
    async fn insert(&self, account: &Account) -> anyhow::Result<()>;
    async fn save(&self, account: &Account) -> anyhow::Result<()>;
    async fn find(&self, account_id: &ID) -> Option<Account>;
    async fn find_many(&self, account_ids: &[ID]) -> anyhow::Result<Vec<Account>>;
    async fn delete(&self, account_id: &ID) -> Option<Account>;
    async fn find_by_apikey(&self, api_key: &str) -> Option<Account>;
    async fn find_by_webhook_url(&self, url: &str) -> Option<Account>;
}

#[cfg(test)]
mod tests {
    use crate::{setup_context, NettuContext};
    use nettu_scheduler_domain::{Entity, PEMKey};

    /// Creates inmemory and mongo context when mongo is running,
    /// otherwise it will create two inmemory
    async fn create_contexts() -> Vec<NettuContext> {
        vec![NettuContext::create_inmemory(), setup_context().await]
    }

    #[tokio::test]
    async fn create_and_delete() {
        for ctx in create_contexts().await {
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
    }

    #[tokio::test]
    async fn update() {
        for ctx in create_contexts().await {
            let mut account = Default::default();

            // Insert
            assert!(ctx.repos.account_repo.insert(&account).await.is_ok());

            let pubkey = std::fs::read("../api/config/test_public_rsa_key.crt").unwrap();
            let pubkey = String::from_utf8(pubkey).unwrap();

            let pubkey = PEMKey::new(pubkey).unwrap();
            account.set_public_jwt_key(Some(pubkey));

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
}
