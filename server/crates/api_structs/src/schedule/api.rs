use serde::{Deserialize, Serialize};

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
}

pub mod delete_schedule {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub schedule_id: String,
    }
}

pub mod get_schedule {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub schedule_id: String,
    }
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
}
