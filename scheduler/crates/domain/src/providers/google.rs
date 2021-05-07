use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GoogleCalendarAccessRole {
    Owner,
    Writer,
    Reader,
    FreeBusyReader,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleCalendarListEntry {
    pub id: String,
    pub access_role: GoogleCalendarAccessRole,
    pub summary: String,
    pub summary_override: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub time_zone: Option<String>,
    pub color_id: Option<String>,
    pub background_color: Option<String>,
    pub foreground_color: Option<String>,
    pub hidden: Option<bool>,
    pub selected: Option<bool>,
    pub primary: Option<bool>,
    pub deleted: Option<bool>,
}
