use super::IAccountRepo;
use nettu_scheduler_domain::{Account, PEMKey, ID};
use serde_json::Value;
use sqlx::{
    types::{Json, Uuid},
    FromRow, PgPool,
};

pub struct PostgresAccountRepo {
    pool: PgPool,
}

impl PostgresAccountRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
pub struct AccountRaw {
    account_uid: Uuid,
    secret_api_key: String,
    public_jwt_key: Option<String>,
    settings: Value,
}

impl From<AccountRaw> for Account {
    fn from(e: AccountRaw) -> Self {
        Self {
            id: e.account_uid.into(),
            secret_api_key: e.secret_api_key,
            public_jwt_key: e.public_jwt_key.map(|key| PEMKey::new(key).unwrap()),
            settings: serde_json::from_value(e.settings).unwrap(),
        }
    }
}

#[async_trait::async_trait]
impl IAccountRepo for PostgresAccountRepo {
    async fn insert(&self, account: &Account) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO accounts(account_uid, secret_api_key, public_jwt_key, settings)
            VALUES($1, $2, $3, $4)
            "#,
            account.id.inner_ref(),
            account.secret_api_key,
            account.public_jwt_key.clone().map(|key| key.inner()),
            Json(&account.settings) as _
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn save(&self, account: &Account) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            UPDATE accounts
            SET secret_api_key = $2,
            public_jwt_key = $3,
            settings = $4
            WHERE account_uid = $1
            "#,
            account.id.inner_ref(),
            account.secret_api_key,
            account.public_jwt_key.clone().map(|key| key.inner()),
            Json(&account.settings) as _
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find(&self, account_id: &ID) -> Option<Account> {
        let account: AccountRaw = match sqlx::query_as!(
            AccountRaw,
            r#"
            SELECT * FROM accounts
            WHERE account_uid = $1
            "#,
            account_id.inner_ref(),
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(a) => a,
            Err(_) => return None,
        };
        Some(account.into())
    }

    async fn find_many(&self, accounts_ids: &[ID]) -> anyhow::Result<Vec<Account>> {
        let ids = accounts_ids
            .iter()
            .map(|id| *id.inner_ref())
            .collect::<Vec<_>>();
        let accounts_raw: Vec<AccountRaw> = sqlx::query_as!(
            AccountRaw,
            "
            SELECT * FROM accounts
            WHERE account_uid = ANY($1)
            ",
            &ids
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(accounts_raw.into_iter().map(|acc| acc.into()).collect())
    }

    async fn delete(&self, account_id: &ID) -> Option<Account> {
        let account: AccountRaw = match sqlx::query_as!(
            AccountRaw,
            "
            DELETE FROM accounts
            WHERE account_uid = $1
            RETURNING *
            ",
            account_id.inner_ref()
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(acc) => acc,
            Err(_) => return None,
        };
        Some(account.into())
    }

    async fn find_by_apikey(&self, api_key: &str) -> Option<Account> {
        let account: AccountRaw = match sqlx::query_as!(
            AccountRaw,
            "
            SELECT * FROM accounts
            WHERE secret_api_key = $1
            ",
            api_key
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(acc) => acc,
            Err(_) => return None,
        };
        Some(account.into())
    }
}
