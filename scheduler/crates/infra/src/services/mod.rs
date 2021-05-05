pub mod google_calendar;

pub struct FreeBusyProviderQuery {
    calendar_ids: Vec<String>,
    start: i64,
    end: i64,
}

// TODOS:
// 1. Maybe ready to get into the meat with settings synced calendars and service resource calendars ?
// 1.1. Add external calendar to service resource, validate that user has at least freebusy access to that repo
// 1.2. Add sync calendars to nettu calendars, and add subscribers to use cases?
// 2. User Freebusy queries :)
// 3. Add option to filter on users on freebusy query, can be very useful

// #[async_trait::async_trait]
// pub trait CalendarProvider {
//     type CalendarEvent: From<CalendarEvent>;
//     type Calendar;

//     async fn freebusy(&self, query: FreeBusyProviderQuery) -> CompatibleInstances;
//     async fn create_event(&self, event: &CalendarEvent) -> Result<Self::CalendarEvent, ()>;
//     async fn delete_event(&self, event: &CalendarEvent) -> Result<Self::CalendarEvent, ()>;
//     async fn list(&self) -> Vec<Self::Calendar>;
// }
