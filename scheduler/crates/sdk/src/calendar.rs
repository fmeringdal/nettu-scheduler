use crate::{APIResponse, BaseClient, ID};
use nettu_scheduler_api_structs::*;
use reqwest::StatusCode;
use std::sync::Arc;

#[derive(Clone)]
pub struct CalendarClient {
    base: Arc<BaseClient>,
}

pub struct CreateCalendarInput {
    pub user_id: ID,
    pub timezone: String,
    pub week_start: isize,
}

pub struct GetCalendarInput {
    pub calendar_id: ID,
}

pub struct GetCalendarEventsInput {
    pub calendar_id: ID,
    pub start_ts: i64,
    pub end_ts: i64,
}

pub struct DeleteCalendarInput {
    pub calendar_id: ID,
}

pub struct UpdateCalendarSettingsInput {
    pub calendar_id: ID,
    pub week_start: Option<isize>,
    pub timezone: Option<String>,
}

impl CalendarClient {
    pub(crate) fn new(base: Arc<BaseClient>) -> Self {
        Self { base }
    }

    pub async fn update_settings(
        &self,
        input: UpdateCalendarSettingsInput,
    ) -> APIResponse<update_calendar_settings::APIResponse> {
        let body = update_calendar_settings::RequestBody {
            timezone: input.timezone.clone(),
            week_start: input.week_start,
        };
        self.base
            .put(
                body,
                format!("user/calendar/{}/settings", input.calendar_id),
                StatusCode::OK,
            )
            .await
    }

    pub async fn delete(
        &self,
        input: DeleteCalendarInput,
    ) -> APIResponse<delete_calendar::APIResponse> {
        self.base
            .delete(
                format!("user/calendar/{}", input.calendar_id),
                StatusCode::OK,
            )
            .await
    }

    pub async fn get(&self, input: GetCalendarInput) -> APIResponse<get_calendar::APIResponse> {
        self.base
            .get(
                format!("user/calendar/{}", input.calendar_id),
                StatusCode::OK,
            )
            .await
    }

    pub async fn get_events(
        &self,
        input: GetCalendarEventsInput,
    ) -> APIResponse<get_calendar_events::APIResponse> {
        self.base
            .get(
                format!(
                    "user/calendar/{}/events?startTs={}&endTs={}",
                    input.calendar_id, input.start_ts, input.end_ts
                ),
                StatusCode::OK,
            )
            .await
    }

    pub async fn create(
        &self,
        input: CreateCalendarInput,
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
