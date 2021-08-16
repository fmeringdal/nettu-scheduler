use crate::dtos::ScheduleDTO;
use nettu_scheduler_domain::{Schedule, ID};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleResponse {
    pub schedule: ScheduleDTO,
}

impl ScheduleResponse {
    pub fn new(schedule: Schedule) -> Self {
        Self {
            schedule: ScheduleDTO::new(schedule),
        }
    }
}

pub mod create_schedule {
    use nettu_scheduler_domain::{Metadata, ScheduleRule};

    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub user_id: ID,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        pub timezone: String,
        pub rules: Option<Vec<ScheduleRule>>,
        pub metadata: Option<Metadata>,
    }

    pub type APIResponse = ScheduleResponse;
}

pub mod delete_schedule {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub schedule_id: ID,
    }

    pub type APIResponse = ScheduleResponse;
}

pub mod get_schedule {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub schedule_id: ID,
    }

    pub type APIResponse = ScheduleResponse;
}

pub mod update_schedule {
    use super::*;
    use nettu_scheduler_domain::{Metadata, ScheduleRule};

    #[derive(Deserialize)]
    pub struct PathParams {
        pub schedule_id: ID,
    }

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        pub timezone: Option<String>,
        pub rules: Option<Vec<ScheduleRule>>,
        pub metadata: Option<Metadata>,
    }

    pub type APIResponse = ScheduleResponse;
}

pub mod get_schedules_by_meta {
    use super::*;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct QueryParams {
        pub key: String,
        pub value: String,
        #[serde(default)]
        pub skip: Option<usize>,
        pub limit: Option<usize>,
    }

    #[derive(Deserialize, Serialize)]
    pub struct APIResponse {
        pub schedules: Vec<ScheduleDTO>,
    }

    impl APIResponse {
        pub fn new(schedules: Vec<Schedule>) -> Self {
            Self {
                schedules: schedules.into_iter().map(ScheduleDTO::new).collect(),
            }
        }
    }
}
