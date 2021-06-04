use super::IEventRemindersExpansionJobsRepo;
use crate::repos::shared::{query_structs::MetadataFindQuery, repo::DeleteResult};
use nettu_scheduler_domain::{Calendar, CalendarEvent, EventRemindersExpansionJob, Metadata, ID};
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
            id: Default::default(),
            event_id: Default::default(),
            timestamp: self.timestamp,
        }
    }
}

#[async_trait::async_trait]
impl IEventRemindersExpansionJobsRepo for PostgresEventReminderExpansionJobsRepo {
    async fn bulk_insert(&self, jobs: &[EventRemindersExpansionJob]) -> anyhow::Result<()> {
        for job in jobs {
            let id = Uuid::new_v4();
            let event_id = Uuid::new_v4();
            sqlx::query!(
                r#"
            INSERT INTO calendar_event_reminder_expansion_jobs 
            (job_uid, event_uid, timestamp)
            VALUES($1, $2, $3)
            "#,
                id,
                event_id,
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
        let id = Uuid::new_v4();
        let res = sqlx::query!(
            r#"
            DELETE FROM calendar_event_reminder_expansion_jobs AS j
            WHERE j.event_uid = $1
            "#,
            id,
        )
        .execute(&self.pool)
        .await?;
        Ok(DeleteResult {
            deleted_count: res.rows_affected() as i64,
        })
    }
}
