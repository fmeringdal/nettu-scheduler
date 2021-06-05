use super::IServiceUserRepo;

use nettu_scheduler_domain::{ServiceResource, TimePlan, ID};
use serde::Deserialize;
use sqlx::{types::Uuid, FromRow, PgPool};

pub struct PostgresServiceUserRepo {
    pool: PgPool,
}

impl PostgresServiceUserRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow, Deserialize)]
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
            TimePlan::Calendar(calendar.into())
        } else if let Some(schedule) = self.available_schedule_uid {
            TimePlan::Schedule(schedule.into())
        } else {
            TimePlan::Empty
        };

        ServiceResource {
            user_id: self.user_uid.into(),
            service_id: self.service_uid.into(),
            availability,
            buffer: self.buffer,
            busy: self
                .busy
                .unwrap_or_default()
                .into_iter()
                .map(|uid| uid.into())
                .collect(),
            closest_booking_time: self.closest_booking_time,
            furthest_booking_time: self.furthest_booking_time,
        }
    }
}

#[async_trait::async_trait]
impl IServiceUserRepo for PostgresServiceUserRepo {
    async fn insert(&self, user: &ServiceResource) -> anyhow::Result<()> {
        let (available_calendar_id, available_schedule_id) = match &user.availability {
            TimePlan::Calendar(id) => (Some(id.inner_ref()), None),
            TimePlan::Schedule(id) => (None, Some(id.inner_ref())),
            _ => (None, None),
        };

        sqlx::query!(
            r#"
            INSERT INTO service_users(service_uid, user_uid, available_calendar_uid, available_schedule_uid, buffer, closest_booking_time, furthest_booking_time)
            VALUES($1, $2, $3, $4, $5, $6, $7)
            "#,
            user.service_id.inner_ref(),
            user.user_id.inner_ref(),
            available_calendar_id,
            available_schedule_id,
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
                user.service_id.inner_ref(),
                user.user_id.inner_ref(),
                busy.inner_ref()
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    async fn save(&self, user: &ServiceResource) -> anyhow::Result<()> {
        let (available_calendar_id, available_schedule_id) = match &user.availability {
            TimePlan::Calendar(id) => (Some(id.inner_ref()), None),
            TimePlan::Schedule(id) => (None, Some(id.inner_ref())),
            _ => (None, None),
        };
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
            user.service_id.inner_ref(),
            user.user_id.inner_ref(),
            available_calendar_id,
            available_schedule_id,
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
                user.service_id.inner_ref(),
                user.user_id.inner_ref(),
                busy.inner_ref()
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    async fn find(&self, service_id: &ID, user_id: &ID) -> Option<ServiceResource> {
        let schedule: ServiceUserRaw = match sqlx::query_as!(
            ServiceUserRaw,
            r#"
            SELECT su.*, array_agg(c.calendar_uid) AS busy FROM service_users as su 
            LEFT JOIN service_user_busy_calendars AS c
            ON su.service_uid = c.service_uid AND su.user_uid = c.user_uid
            WHERE su.service_uid = $1 AND su.user_uid = $2
            GROUP BY su.service_uid, su.user_uid
            "#,
            service_id.inner_ref(),
            user_id.inner_ref()
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
        sqlx::query!(
            r#"
            DELETE FROM service_users AS s
            WHERE s.service_uid = $1 AND
            s.user_uid = $2
            "#,
            service_id.inner_ref(),
            user_id.inner_ref()
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
