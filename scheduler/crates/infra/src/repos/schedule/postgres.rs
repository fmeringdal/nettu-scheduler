use super::IScheduleRepo;
use crate::repos::shared::query_structs::MetadataFindQuery;
use nettu_scheduler_domain::{Schedule, ID};
use serde_json::Value;
use sqlx::{
    types::{Json, Uuid},
    Done, FromRow, PgPool,
};

pub struct PostgresScheduleRepo {
    pool: PgPool,
}

impl PostgresScheduleRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct ScheduleRaw {
    schedule_uid: Uuid,
    user_uid: Uuid,
    account_uid: Uuid,
    rules: Value,
    timezone: String,
    metadata: Value,
}

impl From<ScheduleRaw> for Schedule {
    fn from(e: ScheduleRaw) -> Self {
        Self {
            id: e.schedule_uid.into(),
            user_id: e.user_uid.into(),
            account_id: e.account_uid.into(),
            rules: serde_json::from_value(e.rules).unwrap_or_default(),
            timezone: e
                .timezone
                .parse()
                .unwrap_or_else(|_| "UTC".parse().unwrap()),
            metadata: serde_json::from_value(e.metadata).unwrap(),
        }
    }
}

#[async_trait::async_trait]
impl IScheduleRepo for PostgresScheduleRepo {
    async fn insert(&self, schedule: &Schedule) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO schedules(schedule_uid, user_uid, rules, timezone, metadata)
            VALUES($1, $2, $3, $4, $5)
            "#,
            schedule.id.inner_ref(),
            schedule.user_id.inner_ref(),
            Json(&schedule.rules) as _,
            schedule.timezone.to_string(),
            Json(&schedule.metadata) as _,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn save(&self, schedule: &Schedule) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            UPDATE schedules
            SET rules = $2,
            timezone = $3,
            metadata = $4
            WHERE schedule_uid = $1
            "#,
            schedule.id.inner_ref(),
            Json(&schedule.rules) as _,
            schedule.timezone.to_string(),
            Json(&schedule.metadata) as _,
        )
        .execute(&self.pool)
        .await?
        .rows_affected();
        Ok(())
    }

    async fn find(&self, schedule_id: &ID) -> Option<Schedule> {
        let schedule: ScheduleRaw = match sqlx::query_as!(
            ScheduleRaw,
            r#"
            SELECT s.*, u.account_uid FROM schedules AS s
            INNER JOIN users AS u
                ON u.user_uid = s.user_uid
            WHERE s.schedule_uid = $1
            "#,
            schedule_id.inner_ref(),
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(s) => s,
            Err(_) => return None,
        };
        Some(schedule.into())
    }

    async fn find_many(&self, schedule_ids: &[ID]) -> Vec<Schedule> {
        let ids = schedule_ids
            .iter()
            .map(|id| *id.inner_ref())
            .collect::<Vec<_>>();
        sqlx::query_as!(
            ScheduleRaw,
            r#"
            SELECT s.*, u.account_uid FROM schedules AS s
            INNER JOIN users AS u
                ON u.user_uid = s.user_uid
            WHERE s.schedule_uid = ANY($1)
            "#,
            &ids,
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|e| e.into())
        .collect()
    }

    async fn find_by_user(&self, user_id: &ID) -> Vec<Schedule> {
        let schedules: Vec<ScheduleRaw> = sqlx::query_as!(
            ScheduleRaw,
            r#"
            SELECT s.*, u.account_uid FROM schedules AS s
            INNER JOIN users AS u
                ON u.user_uid = s.user_uid
            WHERE s.user_uid = $1
            "#,
            user_id.inner_ref(),
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        schedules.into_iter().map(|s| s.into()).collect()
    }

    async fn delete(&self, schedule_id: &ID) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM schedules AS s
            WHERE s.schedule_uid = $1
            RETURNING *
            "#,
            schedule_id.inner_ref(),
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_metadata(&self, query: MetadataFindQuery) -> Vec<Schedule> {
        let schedules: Vec<ScheduleRaw> = sqlx::query_as!(
            ScheduleRaw,
            r#"
            SELECT s.*, u.account_uid FROM schedules AS s
            INNER JOIN users AS u
                ON u.user_uid = s.user_uid
            WHERE u.account_uid = $1 AND s.metadata @> $2
            LIMIT $3
            OFFSET $4
            "#,
            query.account_id.inner_ref(),
            Json(&query.metadata) as _,
            query.limit as i64,
            query.skip as i64,
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        schedules.into_iter().map(|s| s.into()).collect()
    }
}
