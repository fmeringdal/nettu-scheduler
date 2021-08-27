use super::IAccountIntegrationRepo;
use nettu_scheduler_domain::{AccountIntegration, IntegrationProvider, ID};
use sqlx::{types::Uuid, FromRow, PgPool};
use tracing::error;

pub struct PostgresAccountIntegrationRepo {
    pool: PgPool,
}

impl PostgresAccountIntegrationRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
pub struct AccountIntegrationRaw {
    account_uid: Uuid,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    provider: String,
}

impl From<AccountIntegrationRaw> for AccountIntegration {
    fn from(e: AccountIntegrationRaw) -> Self {
        Self {
            account_id: e.account_uid.into(),
            client_id: e.client_id,
            client_secret: e.client_secret,
            redirect_uri: e.redirect_uri,
            provider: e.provider.into(),
        }
    }
}

#[async_trait::async_trait]
impl IAccountIntegrationRepo for PostgresAccountIntegrationRepo {
    async fn insert(&self, integration: &AccountIntegration) -> anyhow::Result<()> {
        let provider: String = integration.provider.clone().into();
        sqlx::query!(
            r#"
            INSERT INTO account_integrations(account_uid, client_id, client_secret, redirect_uri, provider)
            VALUES($1, $2, $3, $4, $5)
            "#,
            integration.account_id.inner_ref(),
            integration.client_id,
            integration.client_secret,
            integration.redirect_uri,
            provider as _
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!(
                "Unable to insert account integration: {:?}. DB returned error: {:?}",
                integration, e
            );
            e
        })?;
        Ok(())
    }

    async fn find(&self, account_id: &ID) -> anyhow::Result<Vec<AccountIntegration>> {
        let integrations: Vec<AccountIntegrationRaw> = sqlx::query_as!(
            AccountIntegrationRaw,
            r#"
            SELECT * FROM account_integrations
            WHERE account_uid = $1
            "#,
            account_id.inner_ref(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!(
                "Unable to query account integrations for account: {:?}. DB returned error: {:?}",
                account_id, e
            );
            e
        })?;
        Ok(integrations.into_iter().map(|i| i.into()).collect())
    }

    async fn delete(&self, account_id: &ID, provider: IntegrationProvider) -> anyhow::Result<()> {
        let provider: String = provider.into();
        match sqlx::query!(
            "
            DELETE FROM account_integrations
            WHERE account_uid = $1 AND
            provider = $2
            ",
            account_id.inner_ref(),
            provider
        )
        .execute(&self.pool)
        .await
        {
            Ok(res) if res.rows_affected() == 1 => Ok(()),
            Ok(_) => Err(anyhow::Error::msg("Unable to delete account integration")),
            Err(e) => {
                error!(
                    "Unable to delete account integration for account: {:?} and provider: {:?}. DB returned error: {:?}",
                    account_id, provider, e
                );

                Err(anyhow::Error::msg("Unable to delete account integration"))
            }
        }
    }
}
