use nettu_scheduler_domain::{Schedule, ScheduleRule};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleDTO {
    pub id: String,
    pub user_id: String,
    pub rules: Vec<ScheduleRule>,
    pub timezone: String,
}

impl ScheduleDTO {
    pub fn new(schedule: Schedule) -> Self {
        Self {
            id: schedule.id,
            user_id: schedule.user_id,
            rules: schedule.rules,
            timezone: schedule.timezone.to_string(),
        }
    }
}
