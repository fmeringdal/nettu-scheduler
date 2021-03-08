use crate::{APIResponse, BaseClient};
use nettu_scheduler_api_structs::*;
use reqwest::StatusCode;
use std::sync::Arc;

#[derive(Clone)]
pub struct CalendarEventClient {
    base: Arc<BaseClient>,
}

pub type CreateEventInput = create_event::RequestBody;

pub struct GetEventInput {
    pub event_id: String,
}

pub struct GetEventsInstancesInput {
    pub event_id: String,
    pub start_ts: i64,
    pub end_ts: i64,
}

pub struct DeleteEventInput {
    pub event_id: String,
}

impl CalendarEventClient {
    pub(crate) fn new(base: Arc<BaseClient>) -> Self {
        Self { base }
    }

    pub async fn delete(&self, input: &DeleteEventInput) -> APIResponse<delete_event::APIResponse> {
        self.base
            .delete(format!("user/events/{}", input.event_id), StatusCode::OK)
            .await
    }

    pub async fn get(&self, input: &GetEventInput) -> APIResponse<get_event::APIResponse> {
        self.base
            .get(format!("user/events/{}", input.event_id), StatusCode::OK)
            .await
    }

    pub async fn get_instances(
        &self,
        input: &GetEventsInstancesInput,
    ) -> APIResponse<get_event_instances::APIResponse> {
        self.base
            .get(
                format!(
                    "user/events/{}?startTs={}&endTs={}",
                    input.event_id, input.start_ts, input.end_ts
                ),
                StatusCode::OK,
            )
            .await
    }

    pub async fn create(
        &self,
        user_id: String,
        input: &CreateEventInput,
    ) -> APIResponse<create_event::APIResponse> {
        self.base
            .post(
                input,
                format!("user/{}/events", user_id),
                StatusCode::CREATED,
            )
            .await
    }
}
