use super::ICalendarSyncedRepo;
use nettu_scheduler_domain::{SyncedCalendar, ID};
use sqlx::{types::Uuid, FromRow, PgPool};
use tracing::error;

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

impl From<SyncedCalendarRaw> for SyncedCalendar {
    fn from(e: SyncedCalendarRaw) -> Self {
        Self {
            user_id: e.user_uid.into(),
            calendar_id: e.calendar_uid.into(),
            ext_calendar_id: e.ext_calendar_id,
            provider: e.provider.into(),
        }
    }
}

#[async_trait::async_trait]
impl ICalendarSyncedRepo for PostgresCalendarSyncedRepo {
    async fn insert(&self, c: &SyncedCalendar) -> anyhow::Result<()> {
        let provider: String = c.provider.clone().into();
        sqlx::query!(
            r#"
            INSERT INTO externally_synced_calendars (
                calendar_uid,
                user_uid,
                ext_calendar_id,
                provider
            )
            VALUES($1, $2, $3, $4)
            "#,
            c.calendar_id.as_ref(),
            c.user_id.as_ref(),
            c.ext_calendar_id,
            provider as _
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!(
                "Unable to insert sync calendar: {:?}. DB returned error: {:?}",
                c, e
            );
            e
        })?;

        Ok(())
    }

    async fn delete(&self, c: &SyncedCalendar) -> anyhow::Result<()> {
        let provider: String = c.provider.clone().into();
        let rows = sqlx::query!(
            r#"
            DELETE FROM externally_synced_calendars
            WHERE calendar_uid = $1 AND
                user_uid = $2 AND
                ext_calendar_id = $3 AND
                provider = $4
            "#,
            c.calendar_id.as_ref(),
            c.user_id.as_ref(),
            c.ext_calendar_id,
            provider as _
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!(
                "Delete sync calendar: {:?} failed. DB returned error: {:?}",
                c, e
            );

            e
        })?;
        if rows.rows_affected() == 1 {
            Ok(())
        } else {
            Err(anyhow::Error::msg("Synced calendar not found"))
        }
    }

    async fn find_by_calendar(&self, calendar_id: &ID) -> anyhow::Result<Vec<SyncedCalendar>> {
        let synced_calendars: Vec<SyncedCalendarRaw> = sqlx::query_as!(
            SyncedCalendarRaw,
            r#"
            SELECT * FROM externally_synced_calendars AS c
            WHERE c.calendar_uid = $1
            "#,
            calendar_id.as_ref(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!(
                "Find synced calendars by calendar id: {:?} failed. DB returned error: {:?}",
                calendar_id, e
            );
            e
        })?;

        Ok(synced_calendars.into_iter().map(|c| c.into()).collect())
    }
}
