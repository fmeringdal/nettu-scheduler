use super::IUserIntegrationRepo;
use nettu_scheduler_domain::{UserIntegration, UserIntegrationProvider, ID};
use serde::Deserialize;
use sqlx::{types::Uuid, Done, FromRow, PgPool};

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

impl Into<UserIntegration> for UserIntegrationRaw {
    fn into(self) -> UserIntegration {
        UserIntegration {
            user_id: self.user_uid.into(),
            account_id: self.account_uid.into(),
            refresh_token: self.refresh_token,
            access_token: self.access_token,
            access_token_expires_ts: self.access_token_expires_ts,
            provider: self.provider.into(),
        }
    }
}

#[async_trait::async_trait]
impl IUserIntegrationRepo for PostgresUserIntegrationRepo {
    async fn insert(&self, integration: &UserIntegration) -> anyhow::Result<()> {
        let provider: String = integration.provider.clone().into();
        sqlx::query!(
            r#"
            INSERT INTO user_integrations(user_uid, provider, refresh_token, access_token, access_token_expires_ts)
            VALUES($1, $2, $3, $4, $5)
            "#,
            integration.user_id.inner_ref(),
            provider,
            integration.refresh_token,
            integration.access_token,
            integration.access_token_expires_ts
        )
        .execute(&self.pool)
        .await?;
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
        .await?;

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

    async fn delete(&self, user_id: &ID, provider: UserIntegrationProvider) -> anyhow::Result<()> {
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
