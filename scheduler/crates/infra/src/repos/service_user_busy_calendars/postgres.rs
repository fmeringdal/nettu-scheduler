use super::{BusyCalendarIdentifier, ExternalBusyCalendarIdentifier, IServiceUserBusyCalendarRepo};

use nettu_scheduler_domain::{BusyCalendar, ID};
use sqlx::{FromRow, PgPool};

pub struct PostgresServiceUseBusyCalendarRepo {
    pool: PgPool,
}

impl PostgresServiceUseBusyCalendarRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct BusyCalendarRaw {
    provider: String,
    calendar_id: String,
}

impl Into<BusyCalendar> for BusyCalendarRaw {
    fn into(self) -> BusyCalendar {
        match &self.provider[..] {
            "google" => BusyCalendar::Google(self.calendar_id),
            "outlook" => BusyCalendar::Outlook(self.calendar_id),
            "nettu" => BusyCalendar::Nettu(self.calendar_id.parse().unwrap()),
            _ => unreachable!("Invalid provider"),
        }
    }
}

#[async_trait::async_trait]
impl IServiceUserBusyCalendarRepo for PostgresServiceUseBusyCalendarRepo {
    async fn exists(&self, input: BusyCalendarIdentifier) -> anyhow::Result<bool> {
        sqlx::query!(
            r#"
            SELECT FROM service_user_busy_calendars WHERE 
            service_uid = $1 AND 
            user_uid = $2 AND
            calendar_uid = $3
            "#,
            input.service_id.inner_ref(),
            input.user_id.inner_ref(),
            input.calendar_id.inner_ref(),
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(true)
    }

    async fn exists_ext(&self, input: ExternalBusyCalendarIdentifier) -> anyhow::Result<bool> {
        sqlx::query!(
            r#"
            SELECT FROM service_user_external_busy_calendars WHERE 
            service_uid = $1 AND 
            user_uid = $2 AND
            ext_calendar_id = $3
            "#,
            input.service_id.inner_ref(),
            input.user_id.inner_ref(),
            input.ext_calendar_id,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(true)
    }

    async fn insert(&self, input: BusyCalendarIdentifier) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO service_user_busy_calendars(service_uid, user_uid, calendar_uid)
            VALUES($1, $2, $3)
            "#,
            input.service_id.inner_ref(),
            input.user_id.inner_ref(),
            input.calendar_id.inner_ref(),
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn insert_ext(&self, input: ExternalBusyCalendarIdentifier) -> anyhow::Result<()> {
        let provider: String = input.provider.into();
        sqlx::query!(
                r#"
            INSERT INTO service_user_external_busy_calendars(service_uid, user_uid, ext_calendar_id, provider)
            VALUES($1, $2, $3, $4)
            "#,
                input.service_id.inner_ref(),
                input.user_id.inner_ref(),
                &input.ext_calendar_id,
                provider as _
            )
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn delete(&self, input: BusyCalendarIdentifier) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM service_user_busy_calendars AS busy
            WHERE busy.service_uid = $1 AND
            busy.user_uid = $2 AND
            busy.calendar_uid = $3
            "#,
            input.service_id.inner_ref(),
            input.user_id.inner_ref(),
            input.calendar_id.inner_ref(),
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_ext(&self, input: ExternalBusyCalendarIdentifier) -> anyhow::Result<()> {
        let provider: String = input.provider.into();
        sqlx::query!(
            r#"
            DELETE FROM service_user_external_busy_calendars AS busy
            WHERE busy.service_uid = $1 AND
            busy.user_uid = $2 AND
            busy.ext_calendar_id = $3 AND
            busy.provider = $4
            "#,
            input.service_id.inner_ref(),
            input.user_id.inner_ref(),
            input.ext_calendar_id,
            provider as _,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find(&self, service_id: &ID, user_id: &ID) -> anyhow::Result<Vec<BusyCalendar>> {
        let busy_calendars: Vec<BusyCalendarRaw> = sqlx::query_as(
            r#"
            SELECT ext_c.provider, ext_c.ext_calendar_id as calendar_id
            FROM service_user_external_busy_calendars AS ext_c 
            WHERE ext_c.service_uid = $1 AND ext_c.user_uid = $2
            UNION ALL
            SELECT 'nettu' as provider, bc.calendar_uid::text as calendar_id
            FROM service_user_busy_calendars AS bc 
            WHERE bc.service_uid = $1 AND bc.user_uid = $2
            "#,
        )
        .bind(service_id.inner_ref())
        .bind(user_id.inner_ref())
        .fetch_all(&self.pool)
        .await?;

        Ok(busy_calendars.into_iter().map(|bc| bc.into()).collect())
    }
}
