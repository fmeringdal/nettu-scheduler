use serde::{Deserialize, Serialize};

use super::domain::{Schedule, ScheduleRule};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleDTO {
    id: String,
    rules: Vec<ScheduleRule>,
    timezone: String,
}

impl ScheduleDTO {
    pub fn new(schedule: &Schedule) -> Self {
        Self {
            id: schedule.id.clone(),
            rules: schedule.rules.clone(),
            timezone: schedule.timezone.to_string(),
        }
    }
}
