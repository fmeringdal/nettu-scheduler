use nettu_scheduler_domain::{Calendar, CalendarSettings, Metadata, ID};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CalendarDTO {
    pub id: ID,
    pub user_id: ID,
    pub settings: CalendarSettingsDTO,
    pub metadata: Metadata,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CalendarSettingsDTO {
    pub week_start: isize,
    pub timezone: String,
}

impl CalendarDTO {
    pub fn new(calendar: Calendar) -> Self {
        Self {
            id: calendar.id.clone(),
            user_id: calendar.user_id.clone(),
            settings: CalendarSettingsDTO::new(&calendar.settings),
            metadata: calendar.metadata,
        }
    }
}

impl CalendarSettingsDTO {
    pub fn new(settings: &CalendarSettings) -> Self {
        Self {
            week_start: settings.week_start,
            timezone: settings.timezone.to_string(),
        }
    }
}
