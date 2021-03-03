use serde::{Deserialize, Serialize};

use crate::dtos::{CalendarDTO, EventWithInstancesDTO};
use nettu_scheduler_domain::{Calendar, EventInstance};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarResponse {
    calendar: CalendarDTO,
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

    #[derive(Deserialize)]
    pub struct PathParams {
        pub user_id: String,
    }

    pub type APIResponse = CalendarResponse;
}

pub mod delete_calendar {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub calendar_id: String,
    }

    pub type APIResponse = CalendarResponse;
}

pub mod get_calendar_events {
    use nettu_scheduler_domain::EventWithInstances;

    use crate::dtos::CalendarEventDTO;

    use super::*;

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct PathParams {
        pub calendar_id: String,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct QueryParams {
        pub start_ts: i64,
        pub end_ts: i64,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
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
    #[serde(rename_all = "camelCase")]
    pub struct PathParams {
        pub calendar_id: String,
    }

    pub type APIResponse = CalendarResponse;
}

pub mod get_user_freebusy {
    use std::collections::VecDeque;

    use super::*;

    #[derive(Debug, Deserialize)]
    pub struct PathParams {
        pub external_user_id: String,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct QueryParams {
        pub start_ts: i64,
        pub end_ts: i64,
        pub calendar_ids: Option<String>,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct APIResponse {
        pub busy: VecDeque<EventInstance>,
        pub user_id: String,
    }
}

pub mod update_calendar_settings {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub calendar_id: String,
    }

    #[derive(Deserialize)]
    pub struct RequestBody {
        pub wkst: Option<isize>,
        pub timezone: Option<String>,
    }

    pub type APIResponse = CalendarResponse;
}
