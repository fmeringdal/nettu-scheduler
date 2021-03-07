use crate::{APIResponse, BaseClient};
use nettu_scheduler_api_structs::*;
use reqwest::StatusCode;
use serde::Serialize;
use std::sync::Arc;

#[derive(Clone)]
pub struct CalendarClient {
    base: Arc<BaseClient>,
}

#[derive(Serialize)]
pub struct CreateCalendarInput {
    pub user_id: String,
    pub timezone: String,
    pub week_start: isize,
}

#[derive(Serialize)]
pub struct GetCalendarInput {
    pub user_id: String,
    pub calendar_id: String,
}

impl CalendarClient {
    pub(crate) fn new(base: Arc<BaseClient>) -> Self {
        Self { base }
    }

    pub async fn get(&self, input: &GetCalendarInput) -> APIResponse<get_calendar::APIResponse> {
        self.base
            .get(
                format!("user/{}/calendar/{}", input.user_id, input.calendar_id),
                StatusCode::OK,
            )
            .await
    }

    pub async fn create(
        &self,
        input: &CreateCalendarInput,
    ) -> APIResponse<create_calendar::APIResponse> {
        let body = create_calendar::RequestBody {
            timezone: input.timezone.clone(),
            week_start: input.week_start,
        };
        self.base
            .post(
                body,
                format!("user/{}/calendar", input.user_id),
                StatusCode::CREATED,
            )
            .await
    }
}
