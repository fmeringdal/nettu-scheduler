use nettu_scheduler_domain::{
    CalendarEvent, CalendarEventReminder, EventInstance, RRuleOptions, ID,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEventDTO {
    pub id: ID,
    pub start_ts: i64,
    pub duration: i64,
    pub busy: bool,
    pub end_ts: i64,
    pub recurrence: Option<RRuleOptions>,
    pub exdates: Vec<i64>,
    pub calendar_id: ID,
    pub user_id: ID,
    pub reminder: Option<CalendarEventReminder>,
}

impl CalendarEventDTO {
    pub fn new(event: CalendarEvent) -> Self {
        Self {
            id: event.id.clone(),
            start_ts: event.start_ts,
            duration: event.duration,
            end_ts: event.end_ts,
            busy: event.busy,
            recurrence: event.recurrence,
            exdates: event.exdates,
            calendar_id: event.calendar_id.clone(),
            user_id: event.user_id.clone(),
            reminder: event.reminder,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventWithInstancesDTO {
    pub event: CalendarEventDTO,
    pub instances: Vec<EventInstance>,
}
