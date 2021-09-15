use crate::dtos::{CalendarDTO, EventWithInstancesDTO};
use nettu_scheduler_domain::{Calendar, EventInstance, Tz, Weekday, ID};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct CalendarResponse {
    pub calendar: CalendarDTO,
}

impl CalendarResponse {
    pub fn new(calendar: Calendar) -> Self {
        Self {
            calendar: CalendarDTO::new(calendar),
        }
    }
}

pub mod create_calendar {
    use super::*;
    use nettu_scheduler_domain::{Metadata, Weekday};

    #[derive(Deserialize)]
    pub struct PathParams {
        pub user_id: ID,
    }

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        pub timezone: Tz,
        #[serde(default = "default_weekday")]
        pub week_start: Weekday,
        pub metadata: Option<Metadata>,
    }

    pub type APIResponse = CalendarResponse;
}

fn default_weekday() -> Weekday {
    Weekday::Mon
}

pub mod add_sync_calendar {
    use super::*;
    use nettu_scheduler_domain::IntegrationProvider;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub user_id: ID,
    }

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        pub provider: IntegrationProvider,
        pub calendar_id: ID,
        pub ext_calendar_id: String,
    }

    pub type APIResponse = String;
}

pub mod remove_sync_calendar {
    use super::*;
    use nettu_scheduler_domain::IntegrationProvider;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub user_id: ID,
    }

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        pub provider: IntegrationProvider,
        pub calendar_id: ID,
        pub ext_calendar_id: String,
    }

    pub type APIResponse = String;
}

pub mod delete_calendar {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub calendar_id: ID,
    }

    pub type APIResponse = CalendarResponse;
}

pub mod get_calendar_events {
    use nettu_scheduler_domain::EventWithInstances;

    use crate::dtos::CalendarEventDTO;

    use super::*;

    #[derive(Debug, Deserialize)]
    pub struct PathParams {
        pub calendar_id: ID,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct QueryParams {
        pub start_ts: i64,
        pub end_ts: i64,
    }

    #[derive(Serialize, Deserialize)]
    pub struct APIResponse {
        pub calendar: CalendarDTO,
        pub events: Vec<EventWithInstancesDTO>,
    }

    impl APIResponse {
        pub fn new(calendar: Calendar, events: Vec<EventWithInstances>) -> Self {
            Self {
                calendar: CalendarDTO::new(calendar),
                events: events
                    .into_iter()
                    .map(|e| EventWithInstancesDTO {
                        event: CalendarEventDTO::new(e.event),
                        instances: e.instances,
                    })
                    .collect(),
            }
        }
    }
}

pub mod get_calendar {
    use super::*;

    #[derive(Serialize, Deserialize)]
    pub struct PathParams {
        pub calendar_id: ID,
    }

    pub type APIResponse = CalendarResponse;
}

pub mod get_calendars_by_meta {
    use super::*;

    #[derive(Deserialize)]
    pub struct QueryParams {
        pub key: String,
        pub value: String,
        #[serde(default)]
        pub skip: Option<usize>,
        pub limit: Option<usize>,
    }

    #[derive(Deserialize, Serialize)]
    pub struct APIResponse {
        pub calendars: Vec<CalendarDTO>,
    }

    impl APIResponse {
        pub fn new(calendars: Vec<Calendar>) -> Self {
            Self {
                calendars: calendars.into_iter().map(CalendarDTO::new).collect(),
            }
        }
    }
}

pub mod get_google_calendars {
    use nettu_scheduler_domain::providers::google::{
        GoogleCalendarAccessRole, GoogleCalendarListEntry,
    };

    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub user_id: ID,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct QueryParams {
        pub min_access_role: GoogleCalendarAccessRole,
    }

    #[derive(Deserialize, Serialize)]
    pub struct APIResponse {
        pub calendars: Vec<GoogleCalendarListEntry>,
    }

    impl APIResponse {
        pub fn new(calendars: Vec<GoogleCalendarListEntry>) -> Self {
            Self { calendars }
        }
    }
}

pub mod get_outlook_calendars {
    use nettu_scheduler_domain::providers::outlook::{OutlookCalendar, OutlookCalendarAccessRole};

    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub user_id: ID,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct QueryParams {
        pub min_access_role: OutlookCalendarAccessRole,
    }

    #[derive(Deserialize, Serialize)]
    pub struct APIResponse {
        pub calendars: Vec<OutlookCalendar>,
    }

    impl APIResponse {
        pub fn new(calendars: Vec<OutlookCalendar>) -> Self {
            Self { calendars }
        }
    }
}

pub mod get_user_freebusy {
    use super::*;
    use std::collections::VecDeque;

    #[derive(Debug, Deserialize)]
    pub struct PathParams {
        pub user_id: ID,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct QueryParams {
        pub start_ts: i64,
        pub end_ts: i64,
        #[serde(default)]
        pub calendar_ids: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct APIResponse {
        pub busy: VecDeque<EventInstance>,
        pub user_id: String,
    }
}

pub mod update_calendar {
    use super::*;
    use nettu_scheduler_domain::{Metadata, Weekday};

    #[derive(Deserialize)]
    pub struct PathParams {
        pub calendar_id: ID,
    }

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CalendarSettings {
        #[serde(default)]
        pub week_start: Option<Weekday>,
        pub timezone: Option<Tz>,
    }

    #[derive(Deserialize, Serialize)]
    pub struct RequestBody {
        pub settings: CalendarSettings,
        #[serde(default)]
        pub metadata: Option<Metadata>,
    }

    pub type APIResponse = CalendarResponse;
}
