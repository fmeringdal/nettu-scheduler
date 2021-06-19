use super::{IEventRepo, MostRecentCreatedServiceEvents};
use crate::repos::{shared::query_structs::MetadataFindQuery, user};
use nettu_scheduler_domain::{CalendarEvent, CalendarEventReminder, Metadata, RRuleOptions, ID};
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
struct MostRecentCreatedServiceEventsRaw {
    user_uid: Uuid,
    created: Option<i64>,
}

impl Into<MostRecentCreatedServiceEvents> for MostRecentCreatedServiceEventsRaw {
    fn into(self) -> MostRecentCreatedServiceEvents {
        MostRecentCreatedServiceEvents {
            user_id: self.user_uid.into(),
            created: self.created,
        }
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
    synced_events: Option<serde_json::Value>,
    service_uid: Option<Uuid>,
    metadata: Vec<String>,
}

fn extract_metadata(entries: Vec<String>) -> Metadata {
    entries
        .into_iter()
        .map(|row| {
            let key_value = row.splitn(2, "_").collect::<Vec<_>>();
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
        let recurrence: Option<RRuleOptions> = match self.recurrence {
            Some(json) => serde_json::from_value(json).unwrap(),
            None => None,
        };
        let reminder: Option<CalendarEventReminder> = match self.reminder {
            Some(json) => serde_json::from_value(json).unwrap(),
            None => None,
        };
        let synced_events = match self.synced_events {
            Some(json) => serde_json::from_value(json).unwrap(),
            None => vec![],
        };

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
            recurrence,
            exdates: self.exdates,
            reminder,
            service_id: self.service_uid.map(|id| id.into()),
            synced_events,
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
                service_uid,
                synced_events,
                metadata
            )
            VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
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
            e.service_id.as_ref().map(|id| id.inner_ref()),
            Json(&e.synced_events) as _,
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
                service_uid = $14,
                synced_events = $15,
                metadata = $16
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
            e.service_id.as_ref().map(|id| id.inner_ref()),
            Json(&e.synced_events) as _,
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
            Err(_e) => return None,
        };
        Some(event.into())
    }

    async fn find_most_recently_created_service_events(
        &self,
        service_id: &ID,
        user_ids: &[ID],
    ) -> Vec<MostRecentCreatedServiceEvents> {
        let user_ids = user_ids
            .iter()
            .map(|id| id.inner_ref().clone())
            .collect::<Vec<_>>();
        // https://github.com/launchbadge/sqlx/issues/367
        let events: Vec<MostRecentCreatedServiceEventsRaw> = match sqlx::query_as(
            r#"
            SELECT users.user_uid, events.created FROM users LEFT JOIN (
                SELECT DISTINCT ON (user_uid) user_uid, created
                FROM calendar_events 
                WHERE service_uid = $1
                ORDER BY user_uid, created DESC
            ) AS events ON events.user_uid = users.user_uid
            WHERE users.user_uid = ANY($2)
            "#,
        )
        .bind(service_id.inner_ref())
        .bind(&user_ids)
        .fetch_all(&self.pool)
        .await
        {
            Ok(events) => events,
            Err(_e) => return vec![],
        };
        events.into_iter().map(|e| e.into()).collect()
    }

    async fn find_by_service(
        &self,
        service_id: &ID,
        user_ids: &[ID],
        min_ts: i64,
        max_ts: i64,
    ) -> Vec<CalendarEvent> {
        let user_ids = user_ids
            .iter()
            .map(|id| id.inner_ref().clone())
            .collect::<Vec<_>>();
        let events: Vec<EventRaw> = match sqlx::query_as!(
            EventRaw,
            r#"
            SELECT * FROM calendar_events AS e
            WHERE e.service_uid = $1 AND
            e.user_uid = ANY($2) AND
            e.start_ts <= $3 AND e.end_ts >= $4
            "#,
            service_id.inner_ref(),
            &user_ids,
            max_ts,
            min_ts,
        )
        .fetch_all(&self.pool)
        .await
        {
            Ok(events) => events,
            Err(_e) => return vec![],
        };
        events.into_iter().map(|e| e.into()).collect()
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
