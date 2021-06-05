use super::IEventRepo;
use crate::repos::shared::{query_structs::MetadataFindQuery};
use nettu_scheduler_domain::{CalendarEvent, Metadata, ID};
use sqlx::{
    types::{Json, Uuid},
    FromRow, PgPool,
};

pub struct PostgresEventRepo {
    pool: PgPool,
}

impl PostgresEventRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct EventRaw {
    event_uid: Uuid,
    calendar_uid: Uuid,
    user_uid: Uuid,
    account_uid: Uuid,
    start_ts: i64,
    duration: i64,
    busy: bool,
    end_ts: i64,
    created: i64,
    updated: i64,
    recurrence: Option<serde_json::Value>,
    exdates: Vec<i64>,
    reminder: Option<serde_json::Value>,
    is_service: bool,
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

impl Into<CalendarEvent> for EventRaw {
    fn into(self) -> CalendarEvent {
        CalendarEvent {
            id: self.event_uid.into(),
            user_id: self.user_uid.into(),
            account_id: self.account_uid.into(),
            calendar_id: self.calendar_uid.into(),
            start_ts: self.start_ts,
            duration: self.duration,
            busy: self.busy,
            end_ts: self.end_ts,
            created: self.created,
            updated: self.updated,
            recurrence: self
                .recurrence
                .map(|recurrence| serde_json::from_value(recurrence).unwrap()),
            exdates: self.exdates,
            reminder: self
                .reminder
                .map(|reminder| serde_json::from_value(reminder).unwrap()),
            is_service: self.is_service,
            metadata: extract_metadata(self.metadata),
        }
    }
}

#[async_trait::async_trait]
impl IEventRepo for PostgresEventRepo {
    async fn insert(&self, e: &CalendarEvent) -> anyhow::Result<()> {
        let metadata = to_metadata(e.metadata.clone());
        sqlx::query!(
            r#"
            INSERT INTO calendar_events(
                event_uid,
                calendar_uid, 
                user_uid, 
                account_uid, 
                start_ts,
                duration,
                end_ts,
                busy,
                created,
                updated,
                recurrence,
                exdates,
                reminder,
                is_service,
                metadata
            )
            VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
            e.id.inner_ref(),
            e.calendar_id.inner_ref(),
            e.user_id.inner_ref(),
            e.account_id.inner_ref(),
            e.start_ts,
            e.duration,
            e.end_ts,
            e.busy,
            e.created,
            e.updated,
            Json(&e.recurrence) as _,
            &e.exdates,
            Json(&e.reminder) as _,
            e.is_service,
            &metadata
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn save(&self, e: &CalendarEvent) -> anyhow::Result<()> {
        let metadata = to_metadata(e.metadata.clone());
        sqlx::query!(
            r#"
            UPDATE calendar_events SET 
                calendar_uid = $2, 
                user_uid = $3, 
                account_uid = $4, 
                start_ts = $5,
                duration = $6,
                end_ts = $7,
                busy = $8,
                created = $9,
                updated = $10,
                recurrence = $11,
                exdates = $12,
                reminder = $13,
                is_service = $14,
                metadata = $15
            WHERE event_uid = $1
            "#,
            e.id.inner_ref(),
            e.calendar_id.inner_ref(),
            e.user_id.inner_ref(),
            e.account_id.inner_ref(),
            e.start_ts,
            e.duration,
            e.end_ts,
            e.busy,
            e.created,
            e.updated,
            Json(&e.recurrence) as _,
            &e.exdates,
            Json(&e.reminder) as _,
            e.is_service,
            &metadata
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find(&self, event_id: &ID) -> Option<CalendarEvent> {
        let event: EventRaw = match sqlx::query_as!(
            EventRaw,
            r#"
            SELECT * FROM calendar_events AS e
            WHERE e.event_uid = $1
            "#,
            event_id.inner_ref(),
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(event) => event,
            Err(_) => return None,
        };
        Some(event.into())
    }

    async fn find_many(&self, event_ids: &[ID]) -> anyhow::Result<Vec<CalendarEvent>> {
        let ids = event_ids
            .iter()
            .map(|id| id.inner_ref().clone())
            .collect::<Vec<_>>();
        let events: Vec<EventRaw> = sqlx::query_as!(
            EventRaw,
            r#"
            SELECT * FROM calendar_events AS e
            WHERE e.event_uid = ANY($1)
            "#,
            &ids,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(events.into_iter().map(|e| e.into()).collect())
    }

    async fn find_by_calendar(
        &self,
        calendar_id: &ID,
        timespan: Option<&nettu_scheduler_domain::TimeSpan>,
    ) -> anyhow::Result<Vec<CalendarEvent>> {
        let events: Vec<EventRaw> = match timespan {
            Some(timespan) => {
                sqlx::query_as!(
                    EventRaw,
                    r#"
                    SELECT * FROM calendar_events AS e
                    WHERE e.calendar_uid = $1 AND 
                    e.start_ts <= $2 AND e.end_ts >= $3
                    "#,
                    calendar_id.inner_ref(),
                    timespan.end(),
                    timespan.start()
                )
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as!(
                    EventRaw,
                    r#"
                    SELECT * FROM calendar_events AS e
                    WHERE e.calendar_uid = $1
                    "#,
                    calendar_id.inner_ref(),
                )
                .fetch_all(&self.pool)
                .await?
            }
        };
        Ok(events.into_iter().map(|e| e.into()).collect())
    }

    async fn delete(&self, event_id: &ID) -> Option<CalendarEvent> {
        match sqlx::query_as!(
            EventRaw,
            r#"
            DELETE FROM calendar_events AS c
            WHERE c.event_uid = $1
            RETURNING *
            "#,
            event_id.inner_ref(),
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(e) => Some(e.into()),
            Err(_) => None,
        }
    }

    async fn find_by_metadata(&self, query: MetadataFindQuery) -> Vec<CalendarEvent> {
        let key = format!("{}_{}", query.metadata.key, query.metadata.value);

        let events: Vec<EventRaw> = sqlx::query_as!(
            EventRaw,
            r#"
            SELECT * FROM calendar_events AS e
            WHERE e.account_uid = $1 AND metadata @> ARRAY[$2]
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

        events.into_iter().map(|e| e.into()).collect()
    }
}
