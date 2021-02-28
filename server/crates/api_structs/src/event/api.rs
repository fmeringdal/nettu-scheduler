use crate::dtos::CalendarEventDTO;
use nettu_scheduler_core::EventInstance;
use nettu_scheduler_core::{CalendarEventReminder, RRuleOptions};
use serde::{Deserialize, Serialize};

pub mod create_event_exception {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub event_id: String,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        pub exception_ts: i64,
    }
}

pub mod create_event {
    use super::*;

    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        pub calendar_id: String,
        pub start_ts: i64,
        pub duration: i64,
        pub busy: Option<bool>,
        pub rrule_options: Option<RRuleOptions>,
        pub reminder: Option<CalendarEventReminder>,
        pub services: Option<Vec<String>>,
    }
}

pub mod delete_event {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub event_id: String,
    }
}

pub mod get_event_instances {

    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub event_id: String,
    }
    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct QueryParams {
        pub start_ts: i64,
        pub end_ts: i64,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct APIResponse {
        pub event: CalendarEventDTO,
        pub instances: Vec<EventInstance>,
    }
}

pub mod get_event {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub event_id: String,
    }
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
        pub event_id: String,
    }
}
