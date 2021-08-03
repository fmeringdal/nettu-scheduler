use super::IEventRemindersExpansionJobsRepo;

use nettu_scheduler_domain::{EventRemindersExpansionJob};
use sqlx::{types::Uuid, FromRow, PgPool};

pub struct PostgresEventReminderExpansionJobsRepo {
    pool: PgPool,
}

impl PostgresEventReminderExpansionJobsRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct JobRaw {
    event_uid: Uuid,
    timestamp: i64,
    version: i64,
}

impl Into<EventRemindersExpansionJob> for JobRaw {
    fn into(self) -> EventRemindersExpansionJob {
        EventRemindersExpansionJob {
            event_id: self.event_uid.into(),
            timestamp: self.timestamp,
            version: self.version,
        }
    }
}

#[async_trait::async_trait]
impl IEventRemindersExpansionJobsRepo for PostgresEventReminderExpansionJobsRepo {
    async fn bulk_insert(&self, jobs: &[EventRemindersExpansionJob]) -> anyhow::Result<()> {
        for job in jobs {
            sqlx::query!(
                r#"
            INSERT INTO calendar_event_reminder_expansion_jobs 
            (event_uid, timestamp, version)
            VALUES($1, $2, $3)
            "#,
                job.event_id.inner_ref(),
                job.timestamp,
                job.version
            )
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    async fn delete_all_before(&self, before: i64) -> Vec<EventRemindersExpansionJob> {
        sqlx::query_as!(
            JobRaw,
            r#"
            DELETE FROM calendar_event_reminder_expansion_jobs AS j
            WHERE j.timestamp <= $1
            RETURNING *
            "#,
            before,
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or(vec![])
        .into_iter()
        .map(|job| job.into())
        .collect()
    }
}
