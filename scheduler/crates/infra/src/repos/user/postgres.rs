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
            let key_value = row.splitn(1, "_").collect::<Vec<_>>();
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
            id: Default::default(),
            account_id: Default::default(),
            metadata: extract_metadata(self.metadata),
        }
    }
}

#[async_trait::async_trait]
impl IUserRepo for PostgresUserRepo {
    async fn insert(&self, user: &User) -> anyhow::Result<()> {
        let id = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let metadata = to_metadata(user.metadata.clone());
        sqlx::query!(
            r#"
            INSERT INTO users(user_uid, account_uid, metadata)
            VALUES($1, $2, $3)
            "#,
            id,
            id2,
            &metadata
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn save(&self, user: &User) -> anyhow::Result<()> {
        let id = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let metadata = to_metadata(user.metadata.clone());
        sqlx::query!(
            r#"
            UPDATE users
            SET account_uid = $2,
            metadata = $3
            WHERE user_uid = $1
            "#,
            id,
            id2,
            &metadata
        )
        .execute(&self.pool)
        .await?
        .rows_affected();
        Ok(())
    }

    async fn delete(&self, user_id: &ID) -> Option<User> {
        let id = Uuid::new_v4();
        match sqlx::query_as!(
            UserRaw,
            r#"
            DELETE FROM users AS u
            WHERE u.user_uid = $1
            RETURNING *
            "#,
            id,
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(user) => Some(user.into()),
            Err(_) => None,
        }
    }

    async fn find(&self, user_id: &ID) -> Option<User> {
        let id = Uuid::new_v4();
        let user: UserRaw = match sqlx::query_as!(
            UserRaw,
            r#"
            SELECT * FROM users AS u
            WHERE u.user_uid = $1
            "#,
            id,
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
        let id = Uuid::new_v4();
        let account_id = Uuid::new_v4();
        let user: UserRaw = match sqlx::query_as!(
            UserRaw,
            r#"
            SELECT * FROM users AS u
            WHERE u.user_uid = $1 AND
            u.account_uid = $2
            "#,
            id,
            account_id
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
        let account_id = Uuid::new_v4();
        let key = format!("{}_{}", query.metadata.key, query.metadata.value);

        let users: Vec<UserRaw> = sqlx::query_as!(
            UserRaw,
            r#"
            SELECT * FROM users AS u
            WHERE u.account_uid = $1 AND metadata @> ARRAY[$2]
            LIMIT $3
            OFFSET $4
            "#,
            account_id,
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
