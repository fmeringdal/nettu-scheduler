use nettu_scheduler_core::Schedule;
use serde::{Deserialize, Serialize};

use crate::dtos::ScheduleDTO;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleResponse {
    schedule: ScheduleDTO,
}

impl ScheduleResponse {
    pub fn new(schedule: Schedule) -> Self {
        Self {
            schedule: ScheduleDTO::new(&schedule),
        }
    }
}

pub mod create_schedule {
    use super::*;

    #[derive(Deserialize)]
    pub struct AdminPathParams {
        pub user_id: String,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequstBody {
        pub timezone: String,
    }

    pub type APIResponse = ScheduleResponse;
}

pub mod delete_schedule {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub schedule_id: String,
    }

    pub type APIResponse = ScheduleResponse;
}

pub mod get_schedule {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub schedule_id: String,
    }

    pub type APIResponse = ScheduleResponse;
}

pub mod update_schedule {
    use nettu_scheduler_core::ScheduleRule;

    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub schedule_id: String,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        pub timezone: Option<String>,
        pub rules: Option<Vec<ScheduleRule>>,
    }

    pub type APIResponse = ScheduleResponse;
}
