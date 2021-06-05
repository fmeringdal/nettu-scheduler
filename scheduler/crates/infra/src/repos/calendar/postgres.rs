use super::ICalendarRepo;
use crate::repos::shared::{query_structs::MetadataFindQuery, repo::DeleteResult};
use nettu_scheduler_domain::{Calendar, Metadata, ID};
use sqlx::{
    types::{Json, Uuid},
    Done, FromRow, PgPool,
};

pub struct PostgresCalendarRepo {
    pool: PgPool,
}

impl PostgresCalendarRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct CalendarRaw {
    calendar_uid: Uuid,
    user_uid: Uuid,
    account_uid: Uuid,
    settings: serde_json::Value,
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

impl Into<Calendar> for CalendarRaw {
    fn into(self) -> Calendar {
        Calendar {
            id: self.calendar_uid.into(),
            user_id: self.user_uid.into(),
            account_id: self.account_uid.into(),
            settings: serde_json::from_value(self.settings).unwrap(),
            metadata: extract_metadata(self.metadata),
        }
    }
}

#[async_trait::async_trait]
impl ICalendarRepo for PostgresCalendarRepo {
    async fn insert(&self, calendar: &Calendar) -> anyhow::Result<()> {
        let metadata = to_metadata(calendar.metadata.clone());
        sqlx::query!(
            r#"
            INSERT INTO calendars(calendar_uid, user_uid, account_uid, settings, metadata)
            VALUES($1, $2, $3, $4, $5)
            "#,
            calendar.id.inner_ref(),
            calendar.user_id.inner_ref(),
            calendar.account_id.inner_ref(),
            Json(&calendar.settings) as _,
            &metadata
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn save(&self, calendar: &Calendar) -> anyhow::Result<()> {
        let metadata = to_metadata(calendar.metadata.clone());
        sqlx::query!(
            r#"
            UPDATE calendars
            SET user_uid = $2,
            account_uid = $3,
            settings = $4,
            metadata = $5
            WHERE calendar_uid = $1
            "#,
            calendar.id.inner_ref(),
            calendar.user_id.inner_ref(),
            calendar.account_id.inner_ref(),
            Json(&calendar.settings) as _,
            &metadata
        )
        .execute(&self.pool)
        .await?
        .rows_affected();
        Ok(())
    }

    async fn find(&self, calendar_id: &ID) -> Option<Calendar> {
        let calendar: CalendarRaw = match sqlx::query_as!(
            CalendarRaw,
            r#"
            SELECT * FROM calendars AS c
            WHERE c.calendar_uid = $1
            "#,
            calendar_id.inner_ref(),
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(cal) => cal,
            Err(_) => return None,
        };
        Some(calendar.into())
    }

    async fn find_by_user(&self, user_id: &ID) -> Vec<Calendar> {
        let calendars: Vec<CalendarRaw> = sqlx::query_as!(
            CalendarRaw,
            r#"
            SELECT * FROM calendars AS c
            WHERE c.user_uid = $1
            "#,
            user_id.inner_ref(),
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or(vec![]);

        calendars.into_iter().map(|c| c.into()).collect()
    }

    async fn delete(&self, calendar_id: &ID) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM calendars AS c
            WHERE c.calendar_uid = $1
            "#,
            calendar_id.inner_ref(),
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|e| anyhow::Error::new(e))
    }

    async fn find_by_metadata(&self, query: MetadataFindQuery) -> Vec<Calendar> {
        let key = format!("{}_{}", query.metadata.key, query.metadata.value);

        let calendars: Vec<CalendarRaw> = sqlx::query_as!(
            CalendarRaw,
            r#"
            SELECT * FROM calendars AS c
            WHERE c.account_uid = $1 AND metadata @> ARRAY[$2]
            LIMIT $3
            OFFSET $4
            "#,
            query.account_id.inner_ref(),
            key,
            query.limit as i64,
            query.skip as i64,
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or(vec![]);

        calendars.into_iter().map(|c| c.into()).collect()
    }
}
