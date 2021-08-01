use super::ICalendarSyncedRepo;
use nettu_scheduler_domain::{SyncedCalendar, ID};
use sqlx::{types::Uuid, FromRow, PgPool};

pub struct PostgresCalendarSyncedRepo {
    pool: PgPool,
}

impl PostgresCalendarSyncedRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct SyncedCalendarRaw {
    calendar_uid: Uuid,
    user_uid: Uuid,
    ext_calendar_id: String,
    provider: String,
}

impl Into<SyncedCalendar> for SyncedCalendarRaw {
    fn into(self) -> SyncedCalendar {
        SyncedCalendar {
            user_id: self.user_uid.into(),
            calendar_id: self.calendar_uid.into(),
            ext_calendar_id: self.ext_calendar_id,
            provider: self.provider.into(),
        }
    }
}

#[async_trait::async_trait]
impl ICalendarSyncedRepo for PostgresCalendarSyncedRepo {
    async fn insert(&self, c: &SyncedCalendar) -> anyhow::Result<()> {
        let provider: String = c.provider.clone().into();
        sqlx::query!(
            r#"
            INSERT INTO calendar_ext_synced_calendars (
                calendar_uid, 
                user_uid, 
                ext_calendar_id, 
                provider
            )
            VALUES($1, $2, $3, $4)
            "#,
            c.calendar_id.inner_ref(),
            c.user_id.inner_ref(),
            c.ext_calendar_id,
            provider as _
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete(&self, c: &SyncedCalendar) -> anyhow::Result<()> {
        let provider: String = c.provider.clone().into();
        sqlx::query!(
            r#"
            DELETE FROM calendar_ext_synced_calendars
            WHERE calendar_uid = $1 AND
                user_uid = $2 AND
                ext_calendar_id = $3 AND
                provider = $4
            "#,
            c.calendar_id.inner_ref(),
            c.user_id.inner_ref(),
            c.ext_calendar_id,
            provider as _
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_by_calendar(&self, calendar_id: &ID) -> anyhow::Result<Vec<SyncedCalendar>> {
        let synced_calendars: Vec<SyncedCalendarRaw> = sqlx::query_as!(
            SyncedCalendarRaw,
            r#"
            SELECT * FROM calendar_ext_synced_calendars AS c
            WHERE c.calendar_uid = $1
            "#,
            calendar_id.inner_ref(),
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(synced_calendars.into_iter().map(|c| c.into()).collect())
    }
}
