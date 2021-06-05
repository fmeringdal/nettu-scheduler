use std::str::FromStr;

use super::IAccountRepo;
use nettu_scheduler_domain::{Account, PEMKey, ID};
use sqlx::{
    types::{Json, Uuid},
    Done, FromRow, PgPool,
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
    settings: serde_json::Value,
}

impl Into<Account> for AccountRaw {
    fn into(self) -> Account {
        Account {
            id: Default::default(),
            secret_api_key: self.secret_api_key,
            public_jwt_key: self.public_jwt_key.map(|key| PEMKey::new(key).unwrap()),
            settings: serde_json::from_value(self.settings).unwrap(),
        }
    }
}

// impl PostgresAccountRepo {
//     pub fn new() -> Self {
//         Self {
//             : db.collection("accounts"),
//         }
//     }
// }

#[async_trait::async_trait]
impl IAccountRepo for PostgresAccountRepo {
    async fn insert(&self, account: &Account) -> anyhow::Result<()> {
        // let id = Uuid::from_str(account.id.to_string().as_str())?;
        let id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO accounts(account_uid, secret_api_key, public_jwt_key, settings)
            VALUES($1, $2, $3, $4)
            "#,
            id,
            account.secret_api_key,
            account.public_jwt_key.clone().map(|key| key.inner()),
            Json(&account.settings) as _
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn save(&self, account: &Account) -> anyhow::Result<()> {
        let id = Uuid::from_str(account.id.to_string().as_str())?;
        sqlx::query!(
            r#"
            UPDATE accounts
            SET secret_api_key = $2,
            public_jwt_key = $3,
            settings = $4
            WHERE account_uid = $1
            "#,
            id,
            account.secret_api_key,
            account.public_jwt_key.clone().map(|key| key.inner()),
            Json(&account.settings) as _
        )
        .execute(&self.pool)
        .await?
        .rows_affected();
        Ok(())
    }

    async fn find(&self, account_id: &ID) -> Option<Account> {
        // let id = Uuid::from_str(account_id.to_string().as_str()).unwrap();
        let id = Uuid::new_v4();
        let account: AccountRaw = match sqlx::query_as!(
            AccountRaw,
            r#"
            SELECT * FROM accounts
            WHERE account_uid = $1
            "#,
            id,
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
        // let ids = accounts_ids
        //     .iter()
        //     .map(|account_id| Uuid::from_str(&account_id.to_string()).unwrap())
        //     .collect::<Vec<_>>();
        let ids = vec![Uuid::from_str("510ae4e5-3fa5-4ab4-8f87-fe33c2c9205c").unwrap()];
        println!("ids: {:?}", ids);
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
        let id = Uuid::new_v4();
        let account: AccountRaw = match sqlx::query_as!(
            AccountRaw,
            "
            DELETE FROM accounts
            WHERE account_uid = $1
            RETURNING *
            ",
            id
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
