use nettu_scheduler_api_structs::*;
use nettu_scheduler_domain::{Metadata, ID};
use reqwest::StatusCode;
use std::sync::Arc;

use crate::{
    base::{APIResponse, BaseClient},
    shared::MetadataFindInput,
};

#[derive(Clone)]
pub struct CalendarClient {
    base: Arc<BaseClient>,
}

pub struct CreateCalendarInput {
    pub user_id: ID,
    pub timezone: String,
    pub week_start: isize,
    pub metadata: Option<Metadata>,
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

pub struct UpdateCalendarInput {
    pub calendar_id: ID,
    pub week_start: Option<isize>,
    pub timezone: Option<String>,
    pub metadata: Option<Metadata>,
}

impl CalendarClient {
    pub(crate) fn new(base: Arc<BaseClient>) -> Self {
        Self { base }
    }

    pub async fn update(
        &self,
        input: UpdateCalendarInput,
    ) -> APIResponse<update_calendar::APIResponse> {
        let settings = update_calendar::CalendarSettings {
            timezone: input.timezone.clone(),
            week_start: input.week_start,
        };
        let body = update_calendar::RequestBody {
            settings,
            metadata: input.metadata,
        };
        self.base
            .put(
                body,
                format!("user/calendar/{}", input.calendar_id),
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

    pub async fn get_by_meta(
        &self,
        input: MetadataFindInput,
    ) -> APIResponse<get_calendars_by_meta::APIResponse> {
        self.base
            .get(
                format!("calendar/meta?{}", input.to_query_string()),
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
            metadata: input.metadata,
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
