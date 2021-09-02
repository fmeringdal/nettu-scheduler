use nettu_scheduler_domain::{Metadata, Schedule, ScheduleRule, Tz, ID};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleDTO {
    pub id: ID,
    pub user_id: ID,
    pub rules: Vec<ScheduleRule>,
    pub timezone: Tz,
    pub metadata: Metadata,
}

impl ScheduleDTO {
    pub fn new(schedule: Schedule) -> Self {
        Self {
            id: schedule.id.clone(),
            user_id: schedule.user_id.clone(),
            rules: schedule.rules,
            timezone: schedule.timezone,
            metadata: schedule.metadata,
        }
    }
}
