use super::IScheduleRepo;
use crate::repos::shared::query_structs::MetadataFindQuery;
use nettu_scheduler_domain::{Metadata, Schedule, ID};
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
    rules: serde_json::Value,
    timezone: String,
    metadata: Vec<String>,
}

fn extract_metadata(entries: Vec<String>) -> Metadata {
    entries
        .into_iter()
        .map(|row| {
            let key_value = row.splitn(1, "_").collect::<Vec<_>>();
            (key_value[0].to_string(), key_value[1].to_string())
        })
        .collect()
}

fn to_metadata(metadata: Metadata) -> Vec<String> {
    metadata
        .into_iter()
        .map(|row| format!("{}_{}", row.0, row.1))
        .collect()
}

impl Into<Schedule> for ScheduleRaw {
    fn into(self) -> Schedule {
        Schedule {
            id: self.schedule_uid.into(),
            user_id: self.user_uid.into(),
            account_id: self.account_uid.into(),
            rules: serde_json::from_value(self.rules).unwrap_or_default(),
            timezone: self.timezone.parse().unwrap_or("UTC".parse().unwrap()),
            metadata: extract_metadata(self.metadata),
        }
    }
}

#[async_trait::async_trait]
impl IScheduleRepo for PostgresScheduleRepo {
    async fn insert(&self, schedule: &Schedule) -> anyhow::Result<()> {
        let metadata = to_metadata(schedule.metadata.clone());
        sqlx::query!(
            r#"
            INSERT INTO schedules(schedule_uid, user_uid, account_uid, rules, timezone, metadata)
            VALUES($1, $2, $3, $4, $5, $6)
            "#,
            schedule.id.inner_ref(),
            schedule.user_id.inner_ref(),
            schedule.account_id.inner_ref(),
            Json(&schedule.rules) as _,
            schedule.timezone.to_string(),
            &metadata
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn save(&self, schedule: &Schedule) -> anyhow::Result<()> {
        let metadata = to_metadata(schedule.metadata.clone());
        sqlx::query!(
            r#"
            UPDATE schedules
            SET user_uid = $2,
            account_uid = $3,
            rules = $4,
            timezone = $5,
            metadata = $6
            WHERE schedule_uid = $1
            "#,
            schedule.id.inner_ref(),
            schedule.user_id.inner_ref(),
            schedule.account_id.inner_ref(),
            Json(&schedule.rules) as _,
            schedule.timezone.to_string(),
            &metadata
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
            SELECT * FROM schedules AS s
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
            .map(|id| id.inner_ref().clone())
            .collect::<Vec<_>>();
        sqlx::query_as!(
            ScheduleRaw,
            r#"
            SELECT * FROM schedules AS s
            WHERE s.schedule_uid = ANY($1)
            "#,
            &ids,
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or(vec![])
        .into_iter()
        .map(|e| e.into())
        .collect()
    }

    async fn find_by_user(&self, user_id: &ID) -> Vec<Schedule> {
        let schedules: Vec<ScheduleRaw> = sqlx::query_as!(
            ScheduleRaw,
            r#"
            SELECT * FROM schedules AS s
            WHERE s.user_uid = $1
            "#,
            user_id.inner_ref(),
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or(vec![]);

        schedules.into_iter().map(|s| s.into()).collect()
    }

    async fn delete(&self, _schedule_id: &ID) -> Option<Schedule> {
        let id = Uuid::new_v4();
        match sqlx::query_as!(
            ScheduleRaw,
            r#"
            DELETE FROM schedules AS s
            WHERE s.schedule_uid = $1
            RETURNING *
            "#,
            id,
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(schedule) => Some(schedule.into()),
            Err(_) => None,
        }
    }

    async fn find_by_metadata(&self, query: MetadataFindQuery) -> Vec<Schedule> {
        let account_id = Uuid::new_v4();
        let key = format!("{}_{}", query.metadata.key, query.metadata.value);

        let schedules: Vec<ScheduleRaw> = sqlx::query_as!(
            ScheduleRaw,
            r#"
            SELECT * FROM schedules AS s
            WHERE s.account_uid = $1 AND metadata @> ARRAY[$2]
            LIMIT $3
            OFFSET $4
            "#,
            account_id,
            key,
            query.limit as i64,
            query.skip as i64,
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or(vec![]);

        schedules.into_iter().map(|s| s.into()).collect()
    }
}
