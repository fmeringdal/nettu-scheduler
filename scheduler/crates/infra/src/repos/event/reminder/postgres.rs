use super::IReminderRepo;

use nettu_scheduler_domain::{Reminder, ID};
use sqlx::{types::Uuid, FromRow, PgPool};

pub struct PostgresReminderRepo {
    pool: PgPool,
}

impl PostgresReminderRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct ReminderVersionRaw {
    event_uid: Uuid,
    version: i64,
}

#[derive(Debug, FromRow)]
struct ReminderRaw {
    event_uid: Uuid,
    account_uid: Uuid,
    remind_at: i64,
    version: i64,
    identifier: String,
}

impl Into<Reminder> for ReminderRaw {
    fn into(self) -> Reminder {
        Reminder {
            event_id: self.event_uid.into(),
            account_id: self.account_uid.into(),
            remind_at: self.remind_at,
            version: self.version,
            identifier: self.identifier,
        }
    }
}

#[async_trait::async_trait]
impl IReminderRepo for PostgresReminderRepo {
    async fn bulk_insert(&self, reminders: &[Reminder]) -> anyhow::Result<()> {
        for reminder in reminders {
            sqlx::query!(
                r#"
            INSERT INTO reminders 
            (event_uid, account_uid, remind_at, version, identifier)
            VALUES($1, $2, $3, $4, $5)
            "#,
                reminder.event_id.inner_ref(),
                reminder.account_id.inner_ref(),
                reminder.remind_at,
                reminder.version,
                reminder.identifier,
            )
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    async fn delete_all_before(&self, before: i64) -> Vec<Reminder> {
        sqlx::query_as!(
            ReminderRaw,
            r#"
            DELETE FROM reminders AS r
            WHERE r.remind_at <= $1
            RETURNING *
            "#,
            before,
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or(vec![])
        .into_iter()
        .map(|reminder| reminder.into())
        .collect()
    }

    async fn init_version(&self, event_id: &ID) -> anyhow::Result<i64> {
        let r_version = sqlx::query_as!(
            ReminderVersionRaw,
            r#"
            INSERT INTO event_reminder_versions
                (event_uid, version)
            VALUES
                ($1, $2)
            RETURNING *
            "#,
            event_id.inner_ref(),
            0
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(r_version.version)
    }

    async fn inc_version(&self, event_id: &ID) -> anyhow::Result<i64> {
        let r_version = sqlx::query_as!(
            ReminderVersionRaw,
            r#"
            UPDATE event_reminder_versions
                SET version = version + 1
            WHERE event_uid = $1 
            RETURNING *
            "#,
            event_id.inner_ref(),
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(r_version.version)
    }
}
