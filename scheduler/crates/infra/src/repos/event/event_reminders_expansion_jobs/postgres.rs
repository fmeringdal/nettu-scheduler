use super::IEventRemindersGenerationJobsRepo;
use nettu_scheduler_domain::EventRemindersExpansionJob;
use sqlx::{types::Uuid, FromRow, PgPool};
use tracing::error;

pub struct PostgresEventReminderGenerationJobsRepo {
    pool: PgPool,
}

impl PostgresEventReminderGenerationJobsRepo {
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

impl From<JobRaw> for EventRemindersExpansionJob {
    fn from(e: JobRaw) -> Self {
        Self {
            event_id: e.event_uid.into(),
            timestamp: e.timestamp,
            version: e.version,
        }
    }
}

#[async_trait::async_trait]
impl IEventRemindersGenerationJobsRepo for PostgresEventReminderGenerationJobsRepo {
    async fn bulk_insert(&self, jobs: &[EventRemindersExpansionJob]) -> anyhow::Result<()> {
        for job in jobs {
            sqlx::query!(
                r#"
            INSERT INTO calendar_event_reminder_generation_jobs
            (event_uid, timestamp, version)
            VALUES($1, $2, $3)
            "#,
                job.event_id.inner_ref(),
                job.timestamp,
                job.version as _
            )
            .execute(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "Unable to insert calendar event reminder expansion job: {:?}. DB returned error: {:?}",
                    job, e
                );
                e
            })?;
        }
        Ok(())
    }

    async fn delete_all_before(&self, before: i64) -> Vec<EventRemindersExpansionJob> {
        sqlx::query_as!(
            JobRaw,
            r#"
            DELETE FROM calendar_event_reminder_generation_jobs AS j
            WHERE j.timestamp <= $1
            RETURNING *
            "#,
            before,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!(
                "Unable to delete calendar event reminder expansion job before timestamp: {}. DB returned error: {:?}",
                before, e
            );
            e
        })
        .unwrap_or_default()
        .into_iter()
        .map(|job| job.into())
        .collect()
    }
}
