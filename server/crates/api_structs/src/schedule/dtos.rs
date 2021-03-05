use nettu_scheduler_domain::{Schedule, ScheduleRule, ID};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleDTO {
    pub id: ID,
    pub user_id: ID,
    pub rules: Vec<ScheduleRule>,
    pub timezone: String,
}

impl ScheduleDTO {
    pub fn new(schedule: Schedule) -> Self {
        Self {
            id: schedule.id.clone(),
            user_id: schedule.user_id.clone(),
            rules: schedule.rules,
            timezone: schedule.timezone.to_string(),
        }
    }
}
