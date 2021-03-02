use nettu_scheduler_domain::{Schedule, ScheduleRule, User};
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
    pub fn new(schedule: &Schedule) -> Self {
        Self {
            id: schedule.id.clone(),
            user_id: User::create_external_id(&schedule.user_id),
            rules: schedule.rules.clone(),
            timezone: schedule.timezone.to_string(),
        }
    }
}
