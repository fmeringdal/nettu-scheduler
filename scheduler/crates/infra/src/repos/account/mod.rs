mod postgres;

use nettu_scheduler_domain::Account;
use nettu_scheduler_domain::ID;
pub use postgres::PostgresAccountRepo;

#[async_trait::async_trait]
pub trait IAccountRepo: Send + Sync {
    async fn insert(&self, account: &Account) -> anyhow::Result<()>;
    async fn save(&self, account: &Account) -> anyhow::Result<()>;
    async fn find(&self, account_id: &ID) -> Option<Account>;
    async fn find_many(&self, account_ids: &[ID]) -> anyhow::Result<Vec<Account>>;
    async fn delete(&self, account_id: &ID) -> Option<Account>;
    async fn find_by_apikey(&self, api_key: &str) -> Option<Account>;
}

#[cfg(test)]
mod tests {
    use crate::setup_context;
    use nettu_scheduler_domain::{Account, Entity, PEMKey};

    #[tokio::test]
    async fn create_and_delete() {
        let ctx = setup_context().await;
        let account = Account::default();

        // Insert
        assert!(ctx.repos.accounts.insert(&account).await.is_ok());

        // Different find methods
        let res = ctx.repos.accounts.find(&account.id).await.unwrap();
        assert!(res.eq(&account));
        let res = ctx
            .repos
            .accounts
            .find_many(&[account.id.clone()])
            .await
            .unwrap();
        assert!(res[0].eq(&account));
        let res = ctx
            .repos
            .accounts
            .find_by_apikey(&account.secret_api_key)
            .await
            .unwrap();
        assert!(res.eq(&account));

        // Delete
        let res = ctx.repos.accounts.delete(&account.id).await;
        assert!(res.is_some());
        assert!(res.unwrap().eq(&account));

        // Find
        assert!(ctx.repos.accounts.find(&account.id).await.is_none());
    }

    #[tokio::test]
    async fn update() {
        let ctx = setup_context().await;
        let mut account = Account::default();

        // Insert
        assert!(ctx.repos.accounts.insert(&account).await.is_ok());

        let pubkey = std::fs::read("../api/config/test_public_rsa_key.crt").unwrap();
        let pubkey = String::from_utf8(pubkey).unwrap();

        let pubkey = PEMKey::new(pubkey).unwrap();
        account.set_public_jwt_key(Some(pubkey));

        // Save
        assert!(ctx.repos.accounts.save(&account).await.is_ok());

        // Find
        assert!(ctx
            .repos
            .accounts
            .find(&account.id)
            .await
            .unwrap()
            .eq(&account));
    }
}
