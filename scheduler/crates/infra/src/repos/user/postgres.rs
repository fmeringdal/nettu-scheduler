use super::{IUserRepo, UserWithIntegrations};
use crate::repos::user_integrations::UserIntegrationRaw;
use crate::repos::{extract_metadata, shared::query_structs::MetadataFindQuery, to_metadata};
use nettu_scheduler_domain::{User, UserIntegration, UserIntegrationProvider, ID};
use serde::{Deserialize, Serialize};
use sqlx::{
    types::{Json, Uuid},
    FromRow, PgPool,
};

pub struct PostgresUserRepo {
    pool: PgPool,
}

impl PostgresUserRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct UserRaw {
    user_uid: Uuid,
    account_uid: Uuid,
    metadata: Vec<String>,
}

#[derive(Debug, FromRow)]
struct UserWithIntegrationsRaw {
    user_uid: Uuid,
    account_uid: Uuid,
    integrations: Option<serde_json::Value>,
    metadata: Vec<String>,
}

impl Into<UserWithIntegrations> for UserWithIntegrationsRaw {
    fn into(self) -> UserWithIntegrations {
        let integrations: Vec<UserIntegrationRaw> = match self.integrations {
            Some(json) => serde_json::from_value(json).unwrap(),
            None => vec![],
        };
        UserWithIntegrations {
            id: self.user_uid.into(),
            account_id: self.account_uid.into(),
            integrations: integrations.into_iter().map(|i| i.into()).collect(),
            metadata: extract_metadata(self.metadata),
        }
    }
}

impl Into<User> for UserRaw {
    fn into(self) -> User {
        User {
            id: self.user_uid.into(),
            account_id: self.account_uid.into(),
            metadata: extract_metadata(self.metadata),
        }
    }
}

#[async_trait::async_trait]
impl IUserRepo for PostgresUserRepo {
    async fn insert(&self, user: &User) -> anyhow::Result<()> {
        let metadata = to_metadata(user.metadata.clone());
        sqlx::query!(
            r#"
            INSERT INTO users(user_uid, account_uid, metadata)
            VALUES($1, $2, $3)
            "#,
            user.id.inner_ref(),
            user.account_id.inner_ref(),
            &metadata
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn save(&self, user: &User) -> anyhow::Result<()> {
        let metadata = to_metadata(user.metadata.clone());
        sqlx::query!(
            r#"
            UPDATE users
            SET account_uid = $2,
            metadata = $3
            WHERE user_uid = $1
            "#,
            user.id.inner_ref(),
            user.account_id.inner_ref(),
            &metadata
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, user_id: &ID) -> Option<User> {
        match sqlx::query_as!(
            UserRaw,
            r#"
            DELETE FROM users AS u
            WHERE u.user_uid = $1
            RETURNING *
            "#,
            user_id.inner_ref(),
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(user) => Some(user.into()),
            Err(_) => None,
        }
    }

    async fn find(&self, user_id: &ID) -> Option<UserWithIntegrations> {
        let user: UserWithIntegrationsRaw = match sqlx::query_as!(
            UserWithIntegrationsRaw,
            r#"
            SELECT u.*, json_agg((ui.*)) AS integrations  FROM users AS u
            LEFT JOIN user_integrations AS ui ON ui.user_uid = u.user_uid
            WHERE u.user_uid = $1
            GROUP BY u.user_uid
            "#,
            user_id.inner_ref(),
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(s) => s,
            Err(_) => return None,
        };
        Some(user.into())
    }

    async fn find_many(&self, user_ids: &[ID]) -> Vec<UserWithIntegrations> {
        let user_ids = user_ids
            .iter()
            .map(|id| id.inner_ref().clone())
            .collect::<Vec<_>>();

        let users: Vec<UserWithIntegrationsRaw> = sqlx::query_as!(
            UserWithIntegrationsRaw,
            r#"
            SELECT u.*, json_agg((ui.*)) AS integrations  FROM users AS u
            LEFT JOIN user_integrations AS ui ON ui.user_uid = u.user_uid
            WHERE u.user_uid = ANY($1)
            GROUP BY u.user_uid
            "#,
            &user_ids
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or(vec![]);

        users.into_iter().map(|u| u.into()).collect()
    }

    async fn find_by_account_id(
        &self,
        user_id: &ID,
        account_id: &ID,
    ) -> Option<UserWithIntegrations> {
        let user: UserWithIntegrationsRaw = match sqlx::query_as!(
            UserWithIntegrationsRaw,
            r#"
            SELECT u.*, json_agg((ui.*)) AS integrations  FROM users AS u
            LEFT JOIN user_integrations AS ui ON ui.user_uid = u.user_uid
            WHERE u.user_uid = $1 AND
            u.account_uid = $2
            GROUP BY u.user_uid
            "#,
            user_id.inner_ref(),
            account_id.inner_ref()
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(s) => s,
            Err(_) => return None,
        };
        Some(user.into())
    }

    async fn find_by_metadata(&self, query: MetadataFindQuery) -> Vec<UserWithIntegrations> {
        let key = format!("{}_{}", query.metadata.key, query.metadata.value);

        let users: Vec<UserWithIntegrationsRaw> = sqlx::query_as!(
            UserWithIntegrationsRaw,
            r#"
            SELECT u.*, json_agg((ui.*)) AS integrations  FROM users AS u
            LEFT JOIN user_integrations AS ui ON ui.user_uid = u.user_uid
            WHERE u.account_uid = $1 AND metadata @> ARRAY[$2]
            GROUP BY u.user_uid
            LIMIT $3
            OFFSET $4
            "#,
            query.account_id.inner_ref(),
            key,
            query.limit as i64,
            query.skip as i64,
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or(vec![]);

        users.into_iter().map(|u| u.into()).collect()
    }
}
