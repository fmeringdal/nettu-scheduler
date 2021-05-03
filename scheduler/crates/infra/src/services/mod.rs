use nettu_scheduler_domain::{CalendarEvent, CompatibleInstances};
mod google_provider;

pub struct FreeBusyProviderQuery {
    calendar_ids: Vec<String>,
    start: i64,
    end: i64,
}

// #[async_trait::async_trait]
// pub trait CalendarProvider {
//     type CalendarEvent: From<CalendarEvent>;
//     type Calendar;

//     async fn freebusy(&self, query: FreeBusyProviderQuery) -> CompatibleInstances;
//     async fn create_event(&self, event: &CalendarEvent) -> Result<Self::CalendarEvent, ()>;
//     async fn delete_event(&self, event: &CalendarEvent) -> Result<Self::CalendarEvent, ()>;
//     async fn list(&self) -> Vec<Self::Calendar>;
// }
