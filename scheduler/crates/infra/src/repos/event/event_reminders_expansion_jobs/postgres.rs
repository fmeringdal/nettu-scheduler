use super::IEventRemindersExpansionJobsRepo;
use crate::repos::shared::repo::DeleteResult;
use nettu_scheduler_domain::{EventRemindersExpansionJob, ID};
use sqlx::{types::Uuid, Done, FromRow, PgPool};

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
    job_uid: Uuid,
    event_uid: Uuid,
    timestamp: i64,
}

impl Into<EventRemindersExpansionJob> for JobRaw {
    fn into(self) -> EventRemindersExpansionJob {
        EventRemindersExpansionJob {
            id: self.job_uid.into(),
            event_id: self.event_uid.into(),
            timestamp: self.timestamp,
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
            (job_uid, event_uid, timestamp)
            VALUES($1, $2, $3)
            "#,
                job.id.inner_ref(),
                job.event_id.inner_ref(),
                job.timestamp
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

    async fn delete_by_event(&self, event_id: &ID) -> anyhow::Result<DeleteResult> {
        let res = sqlx::query!(
            r#"
            DELETE FROM calendar_event_reminder_expansion_jobs AS j
            WHERE j.event_uid = $1
            "#,
            event_id.inner_ref(),
        )
        .execute(&self.pool)
        .await?;
        Ok(DeleteResult {
            deleted_count: res.rows_affected() as i64,
        })
    }
}