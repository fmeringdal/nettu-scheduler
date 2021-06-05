use super::IServiceUserRepo;
use crate::repos::shared::{query_structs::MetadataFindQuery, repo::DeleteResult};
use nettu_scheduler_domain::{Schedule, ServiceResource, TimePlan, ID};
use sqlx::{
    types::{Json, Uuid},
    Done, FromRow, PgPool,
};

pub struct PostgresServiceUserRepo {
    pool: PgPool,
}

impl PostgresServiceUserRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
pub struct ServiceUserRaw {
    service_uid: Uuid,
    user_uid: Uuid,
    available_calendar_uid: Option<Uuid>,
    available_schedule_uid: Option<Uuid>,
    busy: Option<Vec<Uuid>>,
    buffer: i64,
    closest_booking_time: i64,
    furthest_booking_time: Option<i64>,
}

impl Into<ServiceResource> for ServiceUserRaw {
    fn into(self) -> ServiceResource {
        let availability = if let Some(calendar) = self.available_calendar_uid {
            TimePlan::Calendar(Default::default())
        } else if let Some(schedule) = self.available_schedule_uid {
            TimePlan::Schedule(Default::default())
        } else {
            TimePlan::Empty
        };

        ServiceResource {
            id: Default::default(),
            user_id: Default::default(),
            service_id: Default::default(),
            availability,
            buffer: self.buffer,
            busy: vec![Default::default()],
            closest_booking_time: self.closest_booking_time,
            furthest_booking_time: self.furthest_booking_time,
        }
    }
}

#[async_trait::async_trait]
impl IServiceUserRepo for PostgresServiceUserRepo {
    async fn insert(&self, user: &ServiceResource) -> anyhow::Result<()> {
        let id = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();
        let id4 = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO service_users(service_uid, user_uid, available_calendar_uid, available_schedule_uid, buffer, closest_booking_time, furthest_booking_time)
            VALUES($1, $2, $3, $4, $5, $6, $7)
            "#,
            id,
            id2,
            id3,
            id4,
            user.buffer,
            user.closest_booking_time,
            user.furthest_booking_time
        )
        .execute(&self.pool)
        .await?;

        for busy in &user.busy {
            sqlx::query!(
                r#"
            INSERT INTO service_user_busy_calendars(service_uid, user_uid, calendar_uid)
            VALUES($1, $2, $3)
            "#,
                id,
                id2,
                id3,
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    async fn save(&self, user: &ServiceResource) -> anyhow::Result<()> {
        let id = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();
        let id4 = Uuid::new_v4();
        sqlx::query!(
            r#"
            UPDATE service_users SET 
                available_calendar_uid = $3, 
                available_schedule_uid = $4, 
                buffer = $5, 
                closest_booking_time = $6, 
                furthest_booking_time = $7
            WHERE service_uid = $1 AND user_uid = $2
            "#,
            id,
            id2,
            id3,
            id4,
            user.buffer,
            user.closest_booking_time,
            user.furthest_booking_time
        )
        .execute(&self.pool)
        .await?;

        for busy in &user.busy {
            sqlx::query!(
                r#"
            INSERT INTO service_user_busy_calendars(service_uid, user_uid, calendar_uid)
            VALUES($1, $2, $3)
            ON CONFLICT DO NOTHING
            "#,
                id,
                id2,
                id3,
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    async fn find(&self, service_id: &ID, user_id: &ID) -> Option<ServiceResource> {
        let id = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let schedule: ServiceUserRaw = match sqlx::query_as!(
            ServiceUserRaw,
            r#"
            SELECT su.*, array_agg(c.calendar_uid) AS busy FROM service_users as su 
            LEFT JOIN service_user_busy_calendars AS c
            ON su.service_uid = c.service_uid AND su.user_uid = c.user_uid
            WHERE su.service_uid = $1 AND su.user_uid = $2
            GROUP BY su.service_uid, su.user_uid
            "#,
            id,
            id2
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(s) => s,
            Err(_) => return None,
        };
        Some(schedule.into())
    }

    async fn delete(&self, service_id: &ID, user_id: &ID) -> anyhow::Result<()> {
        let id = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        sqlx::query!(
            r#"
            DELETE FROM service_users AS s
            WHERE s.service_uid = $1 AND
            s.user_uid = $2
            "#,
            id,
            id2
        )
        .execute(&self.pool)
        .await;
        Ok(())
    }
}
