pub mod google_calendar;
mod outlook_calendar;

#[derive(Debug)]
pub struct FreeBusyProviderQuery {
    pub calendar_ids: Vec<String>,
    pub start: i64,
    pub end: i64,
}
