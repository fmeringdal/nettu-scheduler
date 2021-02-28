use nettu_scheduler_core::{
    CalendarEvent, CalendarEventReminder, EventInstance, RRuleOptions, User,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEventDTO {
    pub id: String,
    pub start_ts: i64,
    pub duration: i64,
    pub busy: bool,
    pub end_ts: i64,
    pub recurrence: Option<RRuleOptions>,
    pub exdates: Vec<i64>,
    pub calendar_id: String,
    pub user_id: String,
    pub reminder: Option<CalendarEventReminder>,
}

impl CalendarEventDTO {
    pub fn new(event: &CalendarEvent) -> Self {
        Self {
            id: event.id.clone(),
            start_ts: event.start_ts,
            duration: event.duration,
            end_ts: event.end_ts,
            busy: event.busy,
            recurrence: event.recurrence.clone(),
            exdates: event.exdates.clone(),
            calendar_id: event.calendar_id.clone(),
            user_id: User::create_external_id(&event.user_id),
            reminder: event.reminder.clone(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventWithInstancesDTO {
    pub event: CalendarEventDTO,
    pub instances: Vec<EventInstance>,
}
