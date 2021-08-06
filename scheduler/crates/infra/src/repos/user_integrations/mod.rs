mod postgres;

use nettu_scheduler_domain::{IntegrationProvider, UserIntegration};
pub use postgres::{PostgresUserIntegrationRepo, UserIntegrationRaw};

use nettu_scheduler_domain::ID;

#[async_trait::async_trait]
pub trait IUserIntegrationRepo: Send + Sync {
    async fn insert(&self, integration: &UserIntegration) -> anyhow::Result<()>;
    async fn save(&self, integration: &UserIntegration) -> anyhow::Result<()>;
    async fn find(&self, user_id: &ID) -> anyhow::Result<Vec<UserIntegration>>;
    async fn delete(&self, user_id: &ID, provider: IntegrationProvider) -> anyhow::Result<()>;
}

#[cfg(test)]
mod tests {
    use crate::setup_context;
    use nettu_scheduler_domain::{
        Account, AccountIntegration, IntegrationProvider, User, UserIntegration,
    };

    #[tokio::test]
    async fn test_user_integrations() {
        let ctx = setup_context().await;

        let account = Account::new();
        ctx.repos
            .accounts
            .insert(&account)
            .await
            .expect("To insert account");

        let user = User::new(account.id.clone());
        ctx.repos.users.insert(&user).await.expect("To insert user");

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

            let user_integration = UserIntegration {
                access_token: "".into(),
                access_token_expires_ts: 0,
                refresh_token: "".into(),
                account_id: account.id.clone(),
                user_id: user.id.clone(),
                provider: provider.clone(),
            };
            assert!(ctx
                .repos
                .user_integrations
                .insert(&user_integration)
                .await
                .is_ok());
            let found_integration = ctx
                .repos
                .user_integrations
                .find(&user.id)
                .await
                .expect("Find user integrations")
                .into_iter()
                .find(|i| i.provider == provider)
                .expect("To find integration provider");
            assert_eq!(
                found_integration.access_token,
                user_integration.access_token
            );

            let updated_user_integration = UserIntegration {
                access_token: format!("{}_different", user_integration.access_token),
                access_token_expires_ts: 0,
                refresh_token: "".into(),
                account_id: account.id.clone(),
                user_id: user.id.clone(),
                provider: provider.clone(),
            };
            assert!(ctx
                .repos
                .user_integrations
                .save(&updated_user_integration)
                .await
                .is_ok());
            let found_integration = ctx
                .repos
                .user_integrations
                .find(&user.id)
                .await
                .expect("Find user integrations")
                .into_iter()
                .find(|i| i.provider == provider)
                .expect("To find integration provider");
            assert_eq!(
                found_integration.access_token,
                updated_user_integration.access_token
            );
        }
        let user_integrations = ctx
            .repos
            .user_integrations
            .find(&user.id)
            .await
            .expect("To find user integrations");
        assert_eq!(user_integrations.len(), 2);
        assert_eq!(user_integrations[0].user_id, user.id);
        assert_eq!(user_integrations[1].user_id, user.id);
        assert!(user_integrations
            .iter()
            .find(|c| c.provider == IntegrationProvider::Google)
            .is_some());
        assert!(user_integrations
            .iter()
            .find(|c| c.provider == IntegrationProvider::Outlook)
            .is_some());

        assert!(ctx
            .repos
            .user_integrations
            .delete(&user.id, IntegrationProvider::Google)
            .await
            .is_ok());
        assert!(ctx
            .repos
            .user_integrations
            .delete(&user.id, IntegrationProvider::Google)
            .await
            .is_err());

        // Find after delete
        let user_integrations = ctx
            .repos
            .user_integrations
            .find(&user.id)
            .await
            .expect("To find user integrations");
        assert_eq!(user_integrations.len(), 1);
        assert_eq!(user_integrations[0].account_id, account.id);
        assert_eq!(user_integrations[0].provider, IntegrationProvider::Outlook);
    }
}
