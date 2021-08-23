use super::{IEventRepo, MostRecentCreatedServiceEvents};
use crate::repos::shared::query_structs::MetadataFindQuery;
use nettu_scheduler_domain::{CalendarEvent, CalendarEventReminder, RRuleOptions, ID};
use serde_json::Value;
use sqlx::{
    types::{Json, Uuid},
    FromRow, PgPool,
};
use tracing::error;

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

impl From<MostRecentCreatedServiceEventsRaw> for MostRecentCreatedServiceEvents {
    fn from(e: MostRecentCreatedServiceEventsRaw) -> Self {
        Self {
            user_id: e.user_uid.into(),
            created: e.created,
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
    recurrence: Option<Value>,
    exdates: Vec<i64>,
    reminders: Option<Value>,
    service_uid: Option<Uuid>,
    metadata: Value,
}

impl From<EventRaw> for CalendarEvent {
    fn from(e: EventRaw) -> Self {
        let recurrence: Option<RRuleOptions> = match e.recurrence {
            Some(json) => serde_json::from_value(json).unwrap(),
            None => None,
        };
        let reminders: Vec<CalendarEventReminder> = match e.reminders {
            Some(json) => serde_json::from_value(json).unwrap(),
            None => Vec::new(),
        };

        Self {
            id: e.event_uid.into(),
            user_id: e.user_uid.into(),
            account_id: e.account_uid.into(),
            calendar_id: e.calendar_uid.into(),
            start_ts: e.start_ts,
            duration: e.duration,
            busy: e.busy,
            end_ts: e.end_ts,
            created: e.created,
            updated: e.updated,
            recurrence,
            exdates: e.exdates,
            reminders,
            service_id: e.service_uid.map(|id| id.into()),
            metadata: serde_json::from_value(e.metadata).unwrap(),
        }
    }
}

#[async_trait::async_trait]
impl IEventRepo for PostgresEventRepo {
    async fn insert(&self, e: &CalendarEvent) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO calendar_events(
                event_uid,
                calendar_uid,
                start_ts,
                duration,
                end_ts,
                busy,
                created,
                updated,
                recurrence,
                exdates,
                reminders,
                service_uid,
                metadata
            )
            VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
            e.id.inner_ref(),
            e.calendar_id.inner_ref(),
            e.start_ts,
            e.duration,
            e.end_ts,
            e.busy,
            e.created,
            e.updated,
            Json(&e.recurrence) as _,
            &e.exdates,
            Json(&e.reminders) as _,
            e.service_id.as_ref().map(|id| id.inner_ref()),
            Json(&e.metadata) as _,
        )
        .execute(&self.pool)
        .await
        .map_err(|err| {
            println!("Insert calendar event {:?}. Error: {:?}", e, err);
            error!("Insert calendar event {:?}. Error: {:?}", e, err);
            err
        })?;

        Ok(())
    }

    async fn save(&self, e: &CalendarEvent) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            UPDATE calendar_events SET
                start_ts = $2,
                duration = $3,
                end_ts = $4,
                busy = $5,
                created = $6,
                updated = $7,
                recurrence = $8,
                exdates = $9,
                reminders = $10,
                service_uid = $11,
                metadata = $12
            WHERE event_uid = $1
            "#,
            e.id.inner_ref(),
            e.start_ts,
            e.duration,
            e.end_ts,
            e.busy,
            e.created,
            e.updated,
            Json(&e.recurrence) as _,
            &e.exdates,
            Json(&e.reminders) as _,
            e.service_id.as_ref().map(|id| id.inner_ref()),
            Json(&e.metadata) as _,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Unable to update calendar event: {:?}", e);
            e
        })?;

        Ok(())
    }

    async fn find(&self, event_id: &ID) -> Option<CalendarEvent> {
        let event: EventRaw = match sqlx::query_as!(
            EventRaw,
            r#"
            SELECT e.*, u.user_uid, account_uid FROM calendar_events AS e
            INNER JOIN calendars AS c
                ON c.calendar_uid = e.calendar_uid
            INNER JOIN users AS u
                ON u.user_uid = c.user_uid
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

    async fn find_many(&self, event_ids: &[ID]) -> anyhow::Result<Vec<CalendarEvent>> {
        let ids = event_ids
            .iter()
            .map(|id| *id.inner_ref())
            .collect::<Vec<_>>();
        let events: Vec<EventRaw> = sqlx::query_as(
            r#"
            SELECT e.*, u.user_uid, account_uid FROM calendar_events AS e
            INNER JOIN calendars AS c
                ON c.calendar_uid = e.calendar_uid
            INNER JOIN users AS u
                ON u.user_uid = c.user_uid
            WHERE e.event_uid = ANY($1)
            "#,
        )
        .bind(&ids)
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
                    SELECT e.*, u.user_uid, account_uid FROM calendar_events AS e
                    INNER JOIN calendars AS c
                        ON c.calendar_uid = e.calendar_uid
                    INNER JOIN users AS u
                        ON u.user_uid = c.user_uid
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
                    SELECT e.*, u.user_uid, account_uid FROM calendar_events AS e
                    INNER JOIN calendars AS c
                        ON c.calendar_uid = e.calendar_uid
                    INNER JOIN users AS u
                        ON u.user_uid = c.user_uid
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

    async fn find_most_recently_created_service_events(
        &self,
        service_id: &ID,
        user_ids: &[ID],
    ) -> Vec<MostRecentCreatedServiceEvents> {
        let user_ids = user_ids
            .iter()
            .map(|id| *id.inner_ref())
            .collect::<Vec<_>>();
        // https://github.com/launchbadge/sqlx/issues/367
        let events: Vec<MostRecentCreatedServiceEventsRaw> = match sqlx::query_as(
            r#"
            SELECT users.user_uid, events.created FROM users LEFT JOIN (
                SELECT DISTINCT ON (user_uid) user_uid, e.created
                FROM calendar_events AS e
                INNER JOIN calendars AS c
                    ON c.calendar_uid = e.calendar_uid
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
            Err(e) => {
                error!(
                    "Unable to find_most_recently_created_service_events for service id: {}.  Error : {:?}",
                    service_id,
                    e
                );
                return Vec::new();
            }
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
            .map(|id| *id.inner_ref())
            .collect::<Vec<_>>();
        let events: Vec<EventRaw> = match sqlx::query_as!(
            EventRaw,
            r#"
            SELECT e.*, u.user_uid, account_uid FROM calendar_events AS e
            INNER JOIN calendars AS c
                ON c.calendar_uid = e.calendar_uid
            INNER JOIN users AS u
                ON u.user_uid = c.user_uid
            WHERE e.service_uid = $1 AND
            u.user_uid = ANY($2) AND
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
            Err(_e) => return Vec::new(),
        };
        events.into_iter().map(|e| e.into()).collect()
    }

    async fn find_user_service_events(
        &self,
        user_id: &ID,
        busy: bool,
        min_ts: i64,
        max_ts: i64,
    ) -> Vec<CalendarEvent> {
        let events: Vec<EventRaw> = match sqlx::query_as!(
            EventRaw,
            r#"
            SELECT e.*, u.user_uid, account_uid FROM calendar_events AS e
            INNER JOIN calendars AS c
                ON c.calendar_uid = e.calendar_uid
            INNER JOIN users AS u
                ON u.user_uid = c.user_uid
            WHERE u.user_uid = $1 AND
            e.busy = $2 AND
            e.service_uid IS NOT NULL AND
            e.start_ts <= $3 AND e.end_ts >= $4
            "#,
            user_id.inner_ref(),
            busy,
            max_ts,
            min_ts,
        )
        .fetch_all(&self.pool)
        .await
        {
            Ok(events) => events,
            Err(_e) => return Vec::new(),
        };
        events.into_iter().map(|e| e.into()).collect()
    }

    async fn delete(&self, event_id: &ID) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM calendar_events AS c
            WHERE c.event_uid = $1
            RETURNING *
            "#,
            event_id.inner_ref(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Unable to delete calendar event: {:?}", e);
            e
        })?;
        Ok(())
    }

    async fn delete_by_service(&self, service_id: &ID) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM calendar_events AS c
            WHERE c.service_uid = $1
            "#,
            service_id.inner_ref(),
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_metadata(&self, query: MetadataFindQuery) -> Vec<CalendarEvent> {
        let events: Vec<EventRaw> = sqlx::query_as!(
            EventRaw,
            r#"
            SELECT e.*, u.user_uid, account_uid FROM calendar_events AS e
            INNER JOIN calendars AS c
                ON c.calendar_uid = e.calendar_uid
            INNER JOIN users AS u
                ON u.user_uid = c.user_uid
            WHERE u.account_uid = $1 AND e.metadata @> $2
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

        events.into_iter().map(|e| e.into()).collect()
    }
}
