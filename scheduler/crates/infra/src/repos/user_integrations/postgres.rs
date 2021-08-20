use super::IUserIntegrationRepo;
use nettu_scheduler_domain::{IntegrationProvider, UserIntegration, ID};
use serde::Deserialize;
use sqlx::{types::Uuid, FromRow, PgPool};
use tracing::error;

pub struct PostgresUserIntegrationRepo {
    pool: PgPool,
}

impl PostgresUserIntegrationRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow, Deserialize)]
pub struct UserIntegrationRaw {
    user_uid: Uuid,
    account_uid: Uuid,
    refresh_token: String,
    access_token: String,
    access_token_expires_ts: i64,
    provider: String,
}

impl From<UserIntegrationRaw> for UserIntegration {
    fn from(e: UserIntegrationRaw) -> Self {
        Self {
            user_id: e.user_uid.into(),
            account_id: e.account_uid.into(),
            refresh_token: e.refresh_token,
            access_token: e.access_token,
            access_token_expires_ts: e.access_token_expires_ts,
            provider: e.provider.into(),
        }
    }
}

#[async_trait::async_trait]
impl IUserIntegrationRepo for PostgresUserIntegrationRepo {
    async fn insert(&self, integration: &UserIntegration) -> anyhow::Result<()> {
        let provider: String = integration.provider.clone().into();
        sqlx::query!(
            r#"
            INSERT INTO user_integrations(account_uid, user_uid, provider, refresh_token, access_token, access_token_expires_ts)
            VALUES($1, $2, $3, $4, $5, $6)
            "#,
            integration.account_id.inner_ref(),
            integration.user_id.inner_ref(),
            provider as _,
            integration.refresh_token,
            integration.access_token,
            integration.access_token_expires_ts
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Unable to insert user integration : {:?}", e);
            e
        })?;
        Ok(())
    }

    async fn save(&self, integration: &UserIntegration) -> anyhow::Result<()> {
        let provider: String = integration.provider.clone().into();
        sqlx::query!(
            r#"
            UPDATE user_integrations
            SET access_token = $1,
            access_token_expires_ts = $2,
            refresh_token = $3
            WHERE user_uid = $4 AND provider = $5
            "#,
            integration.access_token,
            integration.access_token_expires_ts,
            integration.refresh_token,
            integration.user_id.inner_ref(),
            // https://github.com/launchbadge/sqlx/issues/1004#issuecomment-764964043
            provider as _
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Unable to save user integration : {:?}", e);
            e
        })?;

        Ok(())
    }

    async fn find(&self, user_id: &ID) -> anyhow::Result<Vec<UserIntegration>> {
        let integrations: Vec<UserIntegrationRaw> = sqlx::query_as!(
            UserIntegrationRaw,
            r#"
            SELECT * FROM user_integrations
            WHERE user_uid = $1
            "#,
            user_id.inner_ref(),
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(integrations.into_iter().map(|i| i.into()).collect())
    }

    async fn delete(&self, user_id: &ID, provider: IntegrationProvider) -> anyhow::Result<()> {
        let provider: String = provider.into();
        match sqlx::query!(
            "
            DELETE FROM user_integrations
            WHERE user_uid = $1 AND
            provider = $2
            ",
            user_id.inner_ref(),
            provider
        )
        .execute(&self.pool)
        .await
        {
            Ok(res) if res.rows_affected() == 1 => Ok(()),
            _ => Err(anyhow::Error::msg("Unable to delete user integration")),
        }
    }
}
