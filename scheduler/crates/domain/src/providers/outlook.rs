use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OutlookCalendarAccessRole {
    Writer,
    Reader,
}
