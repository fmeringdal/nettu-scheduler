use super::IServiceRepo;
use crate::repos::{service_user::ServiceUserRaw, shared::query_structs::MetadataFindQuery};
use nettu_scheduler_domain::{Service, ServiceWithUsers, ID};
use serde_json::Value;
use sqlx::{
    types::{Json, Uuid},
    FromRow, PgPool,
};
use tracing::error;

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
    multi_person: Value,
    metadata: Value,
}

#[derive(Debug, FromRow)]
struct ServiceWithUsersRaw {
    service_uid: Uuid,
    account_uid: Uuid,
    users: Option<Value>,
    multi_person: Value,
    metadata: Value,
}

impl From<ServiceRaw> for Service {
    fn from(e: ServiceRaw) -> Self {
        Self {
            id: e.service_uid.into(),
            account_id: e.account_uid.into(),
            multi_person: serde_json::from_value(e.multi_person).unwrap(),
            metadata: serde_json::from_value(e.metadata).unwrap(),
        }
    }
}

impl From<ServiceWithUsersRaw> for ServiceWithUsers {
    fn from(e: ServiceWithUsersRaw) -> Self {
        let users: Vec<ServiceUserRaw> = match e.users {
            Some(json) => serde_json::from_value(json).unwrap_or_default(),
            None => Vec::new(),
        };
        Self {
            id: e.service_uid.into(),
            account_id: e.account_uid.into(),
            users: users.into_iter().map(|u| u.into()).collect(),
            multi_person: serde_json::from_value(e.multi_person).unwrap(),
            metadata: serde_json::from_value(e.metadata).unwrap(),
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
            Json(&service.metadata) as _,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!(
                "Unable to insert service: {:?}. DB returned error: {:?}",
                service, e
            );
            e
        })?;

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
            Json(&service.metadata) as _,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!(
                "Unable to save service: {:?}. DB returned error: {:?}",
                service, e
            );
            e
        })?;

        Ok(())
    }

    async fn find(&self, service_id: &ID) -> Option<Service> {
        let res: Option<ServiceRaw> = sqlx::query_as!(
            ServiceRaw,
            r#"
            SELECT * FROM services AS s
            WHERE s.service_uid = $1
            "#,
            service_id.inner_ref()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!(
                "Find service with id: {:?} failed. DB returned error: {:?}",
                service_id, e
            );
            e
        })
        .ok()?;

        res.map(|service| service.into())
    }

    async fn find_with_users(&self, service_id: &ID) -> Option<ServiceWithUsers> {
        let res: Option<ServiceWithUsersRaw> = sqlx::query_as(
            r#"
            SELECT s.*, jsonb_agg((su.*)) AS users FROM services AS s
            LEFT JOIN service_users AS su
            ON su.service_uid = s.service_uid
            WHERE s.service_uid = $1
            GROUP BY s.service_uid
            "#,
        )
        .bind(service_id.inner_ref())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!(
                "Find service with id: {:?} failed. DB returned error: {:?}",
                service_id, e
            );
            e
        })
        .ok()?;

        res.map(|service| service.into())
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
        .await
        .map(|_| ())
        .map_err(|e| {
            error!(
                "Delete service with id: {:?} failed. DB returned error: {:?}",
                service_id, e
            );
            anyhow::Error::new(e)
        })
    }

    async fn find_by_metadata(&self, query: MetadataFindQuery) -> Vec<Service> {
        let services: Vec<ServiceRaw> = sqlx::query_as!(
            ServiceRaw,
            r#"
            SELECT * FROM services AS s
            WHERE s.account_uid = $1 AND metadata @> $2
            LIMIT $3
            OFFSET $4
            "#,
            query.account_id.inner_ref(),
            Json(&query.metadata) as _,
            query.limit as i64,
            query.skip as i64,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!(
                "Find services by metadata: {:?} failed. DB returned error: {:?}",
                query, e
            );
            e
        })
        .unwrap_or_default();

        services.into_iter().map(|s| s.into()).collect()
    }
}
