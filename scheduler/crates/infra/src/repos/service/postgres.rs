use super::IServiceRepo;
use crate::repos::shared::query_structs::MetadataFindQuery;
use nettu_scheduler_domain::{Metadata, Service, ID};
use sqlx::{types::Uuid, FromRow, PgPool};

pub struct PostgresServiceRepo {
    pool: PgPool,
}

impl PostgresServiceRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct ServiceRaw {
    service_uid: Uuid,
    account_uid: Uuid,
    metadata: Vec<String>,
}

#[derive(Debug, FromRow)]
struct ServiceWithUsersRaw {
    service_uid: Uuid,
    account_uid: Uuid,
    users: Option<serde_json::Value>,
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

fn to_metadata(metadata: &Metadata) -> Vec<String> {
    metadata
        .into_iter()
        .map(|row| format!("{}_{}", row.0, row.1))
        .collect()
}

impl Into<Service> for ServiceRaw {
    fn into(self) -> Service {
        Service {
            id: Default::default(),
            account_id: Default::default(),
            metadata: extract_metadata(self.metadata),
        }
    }
}

impl Into<Service> for ServiceWithUsersRaw {
    fn into(self) -> Service {
        Service {
            id: Default::default(),
            account_id: Default::default(),
            metadata: extract_metadata(self.metadata),
        }
    }
}

#[async_trait::async_trait]
impl IServiceRepo for PostgresServiceRepo {
    async fn insert(&self, service: &Service) -> anyhow::Result<()> {
        let id = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO services(service_uid, account_uid, metadata)
            VALUES($1, $2, $3)
            "#,
            id,
            id2,
            &to_metadata(&service.metadata)
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn save(&self, service: &Service) -> anyhow::Result<()> {
        let id = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        sqlx::query!(
            r#"
            UPDATE services SET 
                metadata = $2 
            WHERE service_uid = $1 
            "#,
            id,
            &to_metadata(&service.metadata)
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find(&self, service_id: &ID) -> Option<Service> {
        let id = Uuid::new_v4();
        let schedule: ServiceWithUsersRaw = match sqlx::query_as(
            r#"
            SELECT s.*, jsonb_agg((u.*)) AS users FROM services AS s 
            LEFT JOIN (
                SELECT su.*, array_agg(c.calendar_uid) AS busy FROM service_users AS su 
                LEFT JOIN service_user_busy_calendars as c
                ON su.service_uid = c.service_uid AND su.user_uid = c.user_uid
                GROUP BY su.service_uid, su.user_uid
            ) as u
            ON u.service_uid = s.service_uid 
            WHERE s.service_uid = $1
            GROUP BY s.service_uid
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        {
            Ok(s) => s,
            Err(_) => return None,
        };
        Some(schedule.into())
    }

    async fn delete(&self, service_id: &ID) -> anyhow::Result<()> {
        let id = Uuid::new_v4();
        sqlx::query!(
            r#"
            DELETE FROM services AS s
            WHERE s.service_uid = $1 
            "#,
            id,
        )
        .execute(&self.pool)
        .await;
        Ok(())
    }

    async fn find_by_metadata(&self, query: MetadataFindQuery) -> Vec<Service> {
        let account_id = Uuid::new_v4();
        let key = format!("{}_{}", query.metadata.key, query.metadata.value);

        let services: Vec<ServiceRaw> = sqlx::query_as!(
            ServiceRaw,
            r#"
            SELECT * FROM services AS s
            WHERE s.account_uid = $1 AND metadata @> ARRAY[$2]
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

        services.into_iter().map(|s| s.into()).collect()
    }
}
