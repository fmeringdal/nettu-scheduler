use super::IReservationRepo;
use nettu_scheduler_domain::{ServiceReservation, ID};
use sqlx::{types::Uuid, FromRow, PgPool};

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
    reservation_uid: Uuid,
    timestamp: i64,
    service_uid: Uuid,
}

impl Into<ServiceReservation> for ReservationRaw {
    fn into(self) -> ServiceReservation {
        ServiceReservation {
            id: self.reservation_uid.into(),
            service_id: self.service_uid.into(),
            timestamp: self.timestamp,
        }
    }
}

#[async_trait::async_trait]
impl IReservationRepo for PostgresReservationRepo {
    async fn insert(&self, reservation: &ServiceReservation) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO service_reservations(reservation_uid, service_uid, timestamp)
            VALUES($1, $2, $3)
            "#,
            reservation.id.inner_ref(),
            reservation.service_id.inner_ref(),
            reservation.timestamp
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_in_range(
        &self,
        service_id: &ID,
        min_ts: i64,
        max_ts: i64,
    ) -> Vec<ServiceReservation> {
        let reservations: Vec<ReservationRaw> = match sqlx::query_as!(
            ReservationRaw,
            r#"
            SELECT * FROM service_reservations as r
            WHERE r.service_uid = $1 AND
            r.timestamp < $2 AND r.timestamp > $3
            "#,
            service_id.inner_ref(),
            max_ts,
            min_ts
        )
        .fetch_all(&self.pool)
        .await
        {
            Ok(res) => res,
            Err(_) => vec![],
        };
        reservations.into_iter().map(|r| r.into()).collect()
    }
}
