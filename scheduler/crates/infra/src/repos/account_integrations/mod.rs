mod postgres;

use nettu_scheduler_domain::{AccountIntegration, IntegrationProvider};
pub use postgres::PostgresAccountIntegrationRepo;

use nettu_scheduler_domain::ID;

#[async_trait::async_trait]
pub trait IAccountIntegrationRepo: Send + Sync {
    async fn insert(&self, integration: &AccountIntegration) -> anyhow::Result<()>;
    async fn find(&self, account_id: &ID) -> anyhow::Result<Vec<AccountIntegration>>;
    async fn delete(&self, account_id: &ID, provider: IntegrationProvider) -> anyhow::Result<()>;
}

#[cfg(test)]
mod tests {
    use crate::setup_context;
    use nettu_scheduler_domain::{Account, AccountIntegration, IntegrationProvider};

    #[tokio::test]
    async fn test_account_integrations() {
        let ctx = setup_context().await;

        let account = Account::new();
        ctx.repos
            .accounts
            .insert(&account)
            .await
            .expect("To insert account");

        for provider in [IntegrationProvider::Google, IntegrationProvider::Outlook] {
            let acc_integration = AccountIntegration {
                account_id: account.id.clone(),
                client_id: "".into(),
                client_secret: "".into(),
                redirect_uri: "".into(),
                provider: provider.clone(),
            };
            assert!(ctx
                .repos
                .account_integrations
                .insert(&acc_integration)
                .await
                .is_ok());
            assert!(ctx
                .repos
                .account_integrations
                .insert(&acc_integration)
                .await
                .is_err());
        }
        let acc_integrations = ctx
            .repos
            .account_integrations
            .find(&account.id)
            .await
            .expect("To find account integrations");
        assert_eq!(acc_integrations.len(), 2);
        assert_eq!(acc_integrations[0].account_id, account.id);
        assert_eq!(acc_integrations[1].account_id, account.id);
        assert!(acc_integrations
            .iter()
            .find(|c| c.provider == IntegrationProvider::Google)
            .is_some());
        assert!(acc_integrations
            .iter()
            .find(|c| c.provider == IntegrationProvider::Outlook)
            .is_some());

        assert!(ctx
            .repos
            .account_integrations
            .delete(&account.id, IntegrationProvider::Google)
            .await
            .is_ok());
        assert!(ctx
            .repos
            .account_integrations
            .delete(&account.id, IntegrationProvider::Google)
            .await
            .is_err());

        // Find after delete
        let acc_integrations = ctx
            .repos
            .account_integrations
            .find(&account.id)
            .await
            .expect("To find account integrations");
        assert_eq!(acc_integrations.len(), 1);
        assert_eq!(acc_integrations[0].account_id, account.id);
        assert_eq!(acc_integrations[0].provider, IntegrationProvider::Outlook);
    }
}
