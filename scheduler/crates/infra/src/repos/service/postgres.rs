use super::IServiceRepo;
use crate::repos::{
    extract_metadata, service_user::ServiceUserRaw, shared::query_structs::MetadataFindQuery,
    to_metadata,
};
use nettu_scheduler_domain::{Service, ServiceWithUsers, ID};
use sqlx::{
    types::{Json, Uuid},
    FromRow, PgPool,
};

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
    multi_person: serde_json::Value,
    metadata: Vec<String>,
}

#[derive(Debug, FromRow)]
struct ServiceWithUsersRaw {
    service_uid: Uuid,
    account_uid: Uuid,
    users: Option<serde_json::Value>,
    multi_person: serde_json::Value,
    metadata: Vec<String>,
}

impl Into<Service> for ServiceRaw {
    fn into(self) -> Service {
        Service {
            id: self.service_uid.into(),
            account_id: self.account_uid.into(),
            multi_person: serde_json::from_value(self.multi_person).unwrap(),
            metadata: extract_metadata(self.metadata),
        }
    }
}

impl Into<ServiceWithUsers> for ServiceWithUsersRaw {
    fn into(self) -> ServiceWithUsers {
        let users: Vec<ServiceUserRaw> = match self.users {
            Some(json) => serde_json::from_value(json).unwrap_or_default(),
            None => vec![],
        };
        ServiceWithUsers {
            id: self.service_uid.into(),
            account_id: self.account_uid.into(),
            users: users.into_iter().map(|u| u.into()).collect(),
            multi_person: serde_json::from_value(self.multi_person).unwrap(),
            metadata: extract_metadata(self.metadata),
        }
    }
}

#[async_trait::async_trait]
impl IServiceRepo for PostgresServiceRepo {
    async fn insert(&self, service: &Service) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO services(service_uid, account_uid, multi_person, metadata)
            VALUES($1, $2, $3, $4)
            "#,
            service.id.inner_ref(),
            service.account_id.inner_ref(),
            Json(&service.multi_person) as _,
            &to_metadata(service.metadata.clone())
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn save(&self, service: &Service) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            UPDATE services SET 
                multi_person = $2,
                metadata = $3
            WHERE service_uid = $1 
            "#,
            service.id.inner_ref(),
            Json(&service.multi_person) as _,
            &to_metadata(service.metadata.clone())
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find(&self, service_id: &ID) -> Option<Service> {
        let service: ServiceRaw = match sqlx::query_as!(
            ServiceRaw,
            r#"
            SELECT * FROM services AS s 
            WHERE s.service_uid = $1
            "#,
            service_id.inner_ref()
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(s) => s,
            Err(_) => return None,
        };
        Some(service.into())
    }

    async fn find_with_users(&self, service_id: &ID) -> Option<ServiceWithUsers> {
        let service: ServiceWithUsersRaw = match sqlx::query_as(
            r#"
            SELECT s.*, jsonb_agg((su.*)) AS users FROM services AS s 
            LEFT JOIN service_users AS su 
            ON su.service_uid = s.service_uid 
            WHERE s.service_uid = $1
            GROUP BY s.service_uid
            "#,
        )
        .bind(service_id.inner_ref())
        .fetch_one(&self.pool)
        .await
        {
            Ok(s) => s,
            Err(_) => return None,
        };

        Some(service.into())
    }

    async fn delete(&self, service_id: &ID) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM services AS s
            WHERE s.service_uid = $1 
            "#,
            service_id.inner_ref(),
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_metadata(&self, query: MetadataFindQuery) -> Vec<Service> {
        let key = format!("{}_{}", query.metadata.key, query.metadata.value);

        let services: Vec<ServiceRaw> = sqlx::query_as!(
            ServiceRaw,
            r#"
            SELECT * FROM services AS s
            WHERE s.account_uid = $1 AND metadata @> ARRAY[$2]
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

        services.into_iter().map(|s| s.into()).collect()
    }
}
