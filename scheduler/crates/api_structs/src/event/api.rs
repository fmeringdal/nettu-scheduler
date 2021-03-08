use crate::dtos::CalendarEventDTO;
use nettu_scheduler_domain::{CalendarEvent, EventInstance};
use nettu_scheduler_domain::{CalendarEventReminder, RRuleOptions, ID};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEventResponse {
    pub event: CalendarEventDTO,
}

impl CalendarEventResponse {
    pub fn new(event: CalendarEvent) -> Self {
        Self {
            event: CalendarEventDTO::new(event),
        }
    }
}

pub mod create_event_exception {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub event_id: ID,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        pub exception_ts: i64,
    }

    pub type APIResponse = CalendarEventResponse;
}

pub mod create_event {
    use super::*;

    #[derive(Serialize, Deserialize)]
    pub struct PathParams {
        pub user_id: ID,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        pub calendar_id: ID,
        pub start_ts: i64,
        pub duration: i64,
        pub busy: Option<bool>,
        pub rrule_options: Option<RRuleOptions>,
        pub reminder: Option<CalendarEventReminder>,
        pub services: Option<Vec<String>>,
    }

    pub type APIResponse = CalendarEventResponse;
}

pub mod delete_event {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub event_id: ID,
    }

    pub type APIResponse = CalendarEventResponse;
}

pub mod get_event_instances {

    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub event_id: ID,
    }
    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct QueryParams {
        pub start_ts: i64,
        pub end_ts: i64,
    }

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct APIResponse {
        pub event: CalendarEventDTO,
        pub instances: Vec<EventInstance>,
    }

    impl APIResponse {
        pub fn new(event: CalendarEvent, instances: Vec<EventInstance>) -> Self {
            Self {
                event: CalendarEventDTO::new(event),
                instances,
            }
        }
    }
}

pub mod get_event {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub event_id: ID,
    }

    pub type APIResponse = CalendarEventResponse;
}

pub mod update_event {
    use super::*;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        pub start_ts: Option<i64>,
        pub duration: Option<i64>,
        pub busy: Option<bool>,
        pub rrule_options: Option<RRuleOptions>,
        pub services: Option<Vec<String>>,
    }

    #[derive(Deserialize)]
    pub struct PathParams {
        pub event_id: ID,
    }

    pub type APIResponse = CalendarEventResponse;
}

pub mod send_account_event_reminders {
    use super::*;

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AccountEventRemindersDTO {
        events: Vec<CalendarEventDTO>,
    }

    impl AccountEventRemindersDTO {
        pub fn new(events: Vec<CalendarEvent>) -> Self {
            Self {
                events: events
                    .into_iter()
                    .map(|e| CalendarEventDTO::new(e))
                    .collect(),
            }
        }
    }
}
