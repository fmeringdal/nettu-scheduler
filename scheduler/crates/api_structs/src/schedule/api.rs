use nettu_scheduler_domain::{Schedule, ID};
use serde::{Deserialize, Serialize};

use crate::dtos::ScheduleDTO;

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
    use nettu_scheduler_domain::ScheduleRule;

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
    use nettu_scheduler_domain::ScheduleRule;

    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub schedule_id: ID,
    }

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        pub timezone: Option<String>,
        pub rules: Option<Vec<ScheduleRule>>,
    }

    pub type APIResponse = ScheduleResponse;
}
