pub mod google_calendar;
pub mod outlook_calendar;

#[derive(Debug)]
pub struct FreeBusyProviderQuery {
    pub calendar_ids: Vec<String>,
    pub start: i64,
    pub end: i64,
}
