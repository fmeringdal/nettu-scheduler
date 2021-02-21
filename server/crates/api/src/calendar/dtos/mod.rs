use nettu_scheduler_core::{Calendar, CalendarSettings};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarDTO {
    pub id: String,
    pub user_id: String,
    pub settings: CalendarSettingsDTO,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarSettingsDTO {
    pub wkst: isize,
    pub timezone: String,
}

impl CalendarDTO {
    pub fn new(calendar: &Calendar) -> Self {
        Self {
            id: calendar.id.clone(),
            user_id: calendar.user_id.clone(),
            settings: CalendarSettingsDTO::new(&calendar.settings),
        }
    }
}

impl CalendarSettingsDTO {
    pub fn new(settings: &CalendarSettings) -> Self {
        Self {
            wkst: settings.wkst,
            timezone: settings.timezone.to_string(),
        }
    }
}
