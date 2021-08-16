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
    buffer_after: i64,
    buffer_before: i64,
    closest_booking_time: i64,
    furthest_booking_time: Option<i64>,
}

impl From<ServiceUserRaw> for ServiceResource {
    fn from(e: ServiceUserRaw) -> Self {
        let availability = if let Some(calendar) = e.available_calendar_uid {
            TimePlan::Calendar(calendar.into())
        } else if let Some(schedule) = e.available_schedule_uid {
            TimePlan::Schedule(schedule.into())
        } else {
            TimePlan::Empty
        };

        ServiceResource {
            user_id: e.user_uid.into(),
            service_id: e.service_uid.into(),
            availability,
            buffer_after: e.buffer_after,
            buffer_before: e.buffer_before,
            closest_booking_time: e.closest_booking_time,
            furthest_booking_time: e.furthest_booking_time,
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
            INSERT INTO service_users(service_uid, user_uid, available_calendar_uid, available_schedule_uid, buffer_after, buffer_before, closest_booking_time, furthest_booking_time)
            VALUES($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            user.service_id.inner_ref(),
            user.user_id.inner_ref(),
            available_calendar_id,
            available_schedule_id,
            user.buffer_after,
            user.buffer_before,
            user.closest_booking_time,
            user.furthest_booking_time,
        )
        .execute(&self.pool)
        .await?;

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
                buffer_after = $5,
                buffer_before = $6,
                closest_booking_time = $7,
                furthest_booking_time = $8
            WHERE service_uid = $1 AND user_uid = $2
            "#,
            user.service_id.inner_ref(),
            user.user_id.inner_ref(),
            available_calendar_id,
            available_schedule_id,
            user.buffer_after,
            user.buffer_before,
            user.closest_booking_time,
            user.furthest_booking_time,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find(&self, service_id: &ID, user_id: &ID) -> Option<ServiceResource> {
        // https://github.com/launchbadge/sqlx/issues/367
        let service_user: ServiceUserRaw = match sqlx::query_as(
            r#"
            SELECT su.*, array_agg(c.calendar_uid) AS busy FROM service_users as su
            LEFT JOIN service_user_busy_calendars AS c
            ON su.service_uid = c.service_uid AND su.user_uid = c.user_uid
            WHERE su.service_uid = $1 AND su.user_uid = $2
            GROUP BY su.service_uid, su.user_uid
            "#,
        )
        .bind(service_id.inner_ref())
        .bind(user_id.inner_ref())
        .fetch_one(&self.pool)
        .await
        {
            Ok(s) => s,
            Err(_e) => return None,
        };
        Some(service_user.into())
    }

    async fn find_by_user(&self, user_id: &ID) -> Vec<ServiceResource> {
        // https://github.com/launchbadge/sqlx/issues/367
        let service_users: Vec<ServiceUserRaw> = match sqlx::query_as(
            r#"
            SELECT su.*, array_agg(c.calendar_uid) AS busy FROM service_users as su
            LEFT JOIN service_user_busy_calendars AS c
            ON su.service_uid = c.service_uid AND su.user_uid = c.user_uid
            WHERE su.user_uid = $1
            GROUP BY su.service_uid, su.user_uid
            "#,
        )
        .bind(user_id.inner_ref())
        .fetch_all(&self.pool)
        .await
        {
            Ok(s) => s,
            Err(_e) => return vec![],
        };
        service_users.into_iter().map(|u| u.into()).collect()
    }

    async fn delete(&self, service_id: &ID, user_id: &ID) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM service_user_busy_calendars AS busy
            WHERE busy.service_uid = $1 AND
            busy.user_uid = $2
            "#,
            service_id.inner_ref(),
            user_id.inner_ref()
        )
        .execute(&self.pool)
        .await?;

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
