use super::IReservationRepo;
use nettu_scheduler_domain::ID;
use sqlx::{types::Uuid, FromRow, PgPool};
use tracing::error;

pub struct PostgresReservationRepo {
    pool: PgPool,
}

impl PostgresReservationRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct ReservationRaw {
    count: i64,
    timestamp: i64,
    service_uid: Uuid,
}

#[async_trait::async_trait]
impl IReservationRepo for PostgresReservationRepo {
    async fn increment(&self, service_id: &ID, timestamp: i64) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO service_reservations(service_uid, timestamp)
            VALUES($1, $2)
            ON CONFLICT(service_uid, timestamp) DO UPDATE SET count = service_reservations.count + 1
            "#,
            service_id.inner_ref(),
            timestamp
        )
        .execute(&self.pool)
        .await
        .map_err(|err| {
            error!(
                "Unable to increment reservation count for service id: {} at timestamp {}. DB returned error: {:?}",
                service_id, timestamp, err
            );
            err
        })?;

        Ok(())
    }

    async fn decrement(&self, service_id: &ID, timestamp: i64) -> anyhow::Result<()> {
        sqlx::query_as!(
            ReservationRaw,
            r#"
            UPDATE service_reservations as r
            SET count = count - 1
            WHERE r.service_uid = $1 AND r.timestamp = $2
            "#,
            service_id.inner_ref(),
            timestamp,
        )
        .execute(&self.pool)
        .await
        .map_err(|err| {
            error!(
                "Unable to decrement reservation count for service id: {} at timestamp {}. DB returned error: {:?}",
                service_id, timestamp, err
            );
            err
        })?;
        Ok(())
    }

    async fn count(&self, service_id: &ID, timestamp: i64) -> anyhow::Result<usize> {
        let reservation: Option<ReservationRaw> = sqlx::query_as!(
            ReservationRaw,
            r#"
            SELECT * FROM service_reservations as r
            WHERE r.service_uid = $1 AND
            r.timestamp = $2
            "#,
            service_id.inner_ref(),
            timestamp,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| {
            error!(
                "Unable to retrieve reservation count for service id: {} at timestamp {}. DB returned error: {:?}",
                service_id, timestamp, err
            );
            err
        })?;

        let count = reservation.map(|r| r.count).unwrap_or(0);
        Ok(count as usize)
    }
}
