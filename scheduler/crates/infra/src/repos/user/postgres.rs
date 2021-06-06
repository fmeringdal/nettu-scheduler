use super::IUserRepo;
use crate::repos::shared::query_structs::MetadataFindQuery;
use nettu_scheduler_domain::{Metadata, User, ID};
use sqlx::{types::Uuid, Done, FromRow, PgPool};

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

fn extract_metadata(entries: Vec<String>) -> Metadata {
    entries
        .into_iter()
        .map(|row| {
            let key_value = row.splitn(2, "_").collect::<Vec<_>>();
            (key_value[0].to_string(), key_value[1].to_string())
        })
        .collect()
}

fn to_metadata(metadata: Metadata) -> Vec<String> {
    metadata
        .into_iter()
        .map(|row| format!("{}_{}", row.0, row.1))
        .collect()
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
        .await?
        .rows_affected();
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

    async fn find(&self, user_id: &ID) -> Option<User> {
        let user: UserRaw = match sqlx::query_as!(
            UserRaw,
            r#"
            SELECT * FROM users AS u
            WHERE u.user_uid = $1
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

    async fn find_by_account_id(&self, user_id: &ID, account_id: &ID) -> Option<User> {
        let user: UserRaw = match sqlx::query_as!(
            UserRaw,
            r#"
            SELECT * FROM users AS u
            WHERE u.user_uid = $1 AND
            u.account_uid = $2
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

    async fn find_by_metadata(&self, query: MetadataFindQuery) -> Vec<User> {
        let key = format!("{}_{}", query.metadata.key, query.metadata.value);

        let users: Vec<UserRaw> = sqlx::query_as!(
            UserRaw,
            r#"
            SELECT * FROM users AS u
            WHERE u.account_uid = $1 AND metadata @> ARRAY[$2]
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
