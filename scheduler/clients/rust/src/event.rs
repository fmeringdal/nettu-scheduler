use crate::{shared::MetadataFindInput, APIResponse, BaseClient};
use crate::{CalendarEventReminder, RRuleOptions, ID};
use nettu_scheduler_api_structs::*;
use nettu_scheduler_domain::Metadata;
use reqwest::StatusCode;
use serde::Serialize;
use std::sync::Arc;

#[derive(Clone)]
pub struct CalendarEventClient {
    base: Arc<BaseClient>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEventInput {
    pub user_id: ID,
    pub calendar_id: ID,
    pub start_ts: i64,
    pub duration: i64,
    #[serde(default)]
    pub busy: Option<bool>,
    #[serde(default)]
    pub recurrence: Option<RRuleOptions>,
    #[serde(default)]
    pub reminders: Vec<CalendarEventReminder>,
    #[serde(default)]
    pub service_id: Option<ID>,
    #[serde(default)]
    pub metadata: Option<Metadata>,
}

pub struct GetEventsInstancesInput {
    pub event_id: ID,
    pub start_ts: i64,
    pub end_ts: i64,
}

pub struct UpdateEventInput {
    pub event_id: ID,
    pub start_ts: Option<i64>,
    pub duration: Option<i64>,
    pub busy: Option<bool>,
    pub reminders: Option<Vec<CalendarEventReminder>>,
    pub rrule_options: Option<RRuleOptions>,
    pub service_id: Option<ID>,
    pub exdates: Option<Vec<i64>>,
    pub metadata: Option<Metadata>,
}

impl CalendarEventClient {
    pub(crate) fn new(base: Arc<BaseClient>) -> Self {
        Self { base }
    }

    pub async fn delete(&self, event_id: ID) -> APIResponse<delete_event::APIResponse> {
        self.base
            .delete(format!("user/events/{}", event_id), StatusCode::OK)
            .await
    }

    pub async fn get(&self, event_id: ID) -> APIResponse<get_event::APIResponse> {
        self.base
            .get(format!("user/events/{}", event_id), StatusCode::OK)
            .await
    }

    pub async fn get_instances(
        &self,
        input: GetEventsInstancesInput,
    ) -> APIResponse<get_event_instances::APIResponse> {
        self.base
            .get(
                format!(
                    "user/events/{}/instances?startTs={}&endTs={}",
                    input.event_id, input.start_ts, input.end_ts
                ),
                StatusCode::OK,
            )
            .await
    }

    pub async fn create(&self, input: CreateEventInput) -> APIResponse<create_event::APIResponse> {
        let user_id = input.user_id.clone();
        let body = create_event::RequestBody {
            calendar_id: input.calendar_id,
            start_ts: input.start_ts,
            duration: input.duration,
            busy: input.busy,
            recurrence: input.recurrence,
            reminders: input.reminders,
            service_id: input.service_id,
            metadata: input.metadata,
        };

        self.base
            .post(
                body,
                format!("user/{}/events", user_id),
                StatusCode::CREATED,
            )
            .await
    }

    pub async fn get_by_meta(
        &self,
        input: MetadataFindInput,
    ) -> APIResponse<get_events_by_meta::APIResponse> {
        self.base
            .get(
                format!("events/meta?{}", input.to_query_string()),
                StatusCode::OK,
            )
            .await
    }

    pub async fn update(&self, input: UpdateEventInput) -> APIResponse<update_event::APIResponse> {
        let event_id = input.event_id.clone();
        let body = update_event::RequestBody {
            busy: input.busy,
            duration: input.duration,
            exdates: input.exdates,
            recurrence: input.rrule_options,
            reminders: input.reminders,
            service_id: input.service_id,
            start_ts: input.start_ts,
            metadata: input.metadata,
        };
        self.base
            .put(body, format!("user/events/{}", event_id), StatusCode::OK)
            .await
    }
}
