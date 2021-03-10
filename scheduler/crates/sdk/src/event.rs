use crate::{shared::MetadataFindInput, APIResponse, BaseClient};
use crate::{CalendarEventReminder, RRuleOptions, ID};
use nettu_scheduler_api_structs::*;
use nettu_scheduler_domain::Metadata;
use reqwest::StatusCode;
use std::sync::Arc;

#[derive(Clone)]
pub struct CalendarEventClient {
    base: Arc<BaseClient>,
}

pub type CreateEventInput = create_event::RequestBody;

pub struct GetEventInput {
    pub event_id: ID,
}

pub struct GetEventsInstancesInput {
    pub event_id: ID,
    pub start_ts: i64,
    pub end_ts: i64,
}

pub struct DeleteEventInput {
    pub event_id: ID,
}

pub struct UpdateEventInput {
    pub event_id: ID,
    pub start_ts: Option<i64>,
    pub duration: Option<i64>,
    pub busy: Option<bool>,
    pub reminder: Option<CalendarEventReminder>,
    pub rrule_options: Option<RRuleOptions>,
    pub services: Option<Vec<String>>,
    pub exdates: Option<Vec<i64>>,
    pub metadata: Option<Metadata>,
}

impl CalendarEventClient {
    pub(crate) fn new(base: Arc<BaseClient>) -> Self {
        Self { base }
    }

    pub async fn delete(&self, input: DeleteEventInput) -> APIResponse<delete_event::APIResponse> {
        self.base
            .delete(format!("user/events/{}", input.event_id), StatusCode::OK)
            .await
    }

    pub async fn get(&self, input: GetEventInput) -> APIResponse<get_event::APIResponse> {
        self.base
            .get(format!("user/events/{}", input.event_id), StatusCode::OK)
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

    pub async fn create(
        &self,
        user_id: ID,
        input: CreateEventInput,
    ) -> APIResponse<create_event::APIResponse> {
        self.base
            .post(
                input,
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
            rrule_options: input.rrule_options,
            reminder: input.reminder,
            services: input.services,
            start_ts: input.start_ts,
            metadata: input.metadata,
        };
        self.base
            .put(body, format!("user/events/{}", event_id), StatusCode::OK)
            .await
    }
}
