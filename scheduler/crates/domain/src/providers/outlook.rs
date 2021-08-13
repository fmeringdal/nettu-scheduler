use chrono::TimeZone;
use chrono_tz::{Tz, UTC};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OutlookCalendarAccessRole {
    Writer,
    Reader,
}
// https://docs.microsoft.com/en-us/graph/api/resources/datetimetimezone?view=graph-rest-1.0
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutlookCalendarEventTime {
    /// A single point of time in a combined date and time representation ({date}T{time}; for example, 2017-08-29T04:00:00.0000000).
    pub date_time: String,
    pub time_zone: String,
}

impl OutlookCalendarEventTime {
    pub fn get_timestamp_millis(&self) -> i64 {
        self.time_zone
            .parse::<Tz>()
            .unwrap_or(UTC)
            // This is so weird formatting, but it works
            .datetime_from_str(
                &self.date_time[..self.date_time.find('.').unwrap()],
                "%FT%T",
            )
            .map_err(|err| {
                println!("Outlook parse error : {:?}", err);
                println!("Value: {:?}", self);
                err
            })
            .unwrap_or_else(|_| UTC.timestamp(0, 0))
            .timestamp_millis()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OutlookOnlineMeetingProvider {
    #[serde(rename = "teamsForBusiness")]
    BusinessTeams,
    #[serde(rename = "skypeForConsumer")]
    ConsumerSkype,
    #[serde(rename = "skypeForBusiness")]
    BusinessSkype,
    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OutlookCalendarEventShowAs {
    Free,
    Tentative,
    Busy,
    Oof,
    WorkingElsewhere,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutlookCalendarEventOnlineMeeting {
    join_url: String,
    conference_id: String,
    toll_number: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutlookCalendarEvent {
    pub id: String,
    pub start: OutlookCalendarEventTime,
    pub end: OutlookCalendarEventTime,
    subject: String,
    is_online_meeting: bool,
    online_meeting_provider: Option<OutlookOnlineMeetingProvider>,
    online_meeting: Option<OutlookCalendarEventOnlineMeeting>,
    pub show_as: OutlookCalendarEventShowAs,
    //     recurrence: Option<String>,
    body: OutlookCalendarEventBody,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OutlookCalendarEventBodyContentType {
    #[serde(rename = "html")]
    HTML,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutlookCalendarEventBody {
    pub content_type: OutlookCalendarEventBodyContentType,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutlookCalendar {
    pub id: String,
    name: String,
    color: String,
    change_key: String,
    can_share: bool,
    can_view_private_items: bool,
    hex_color: String,
    pub can_edit: bool,
    allowed_online_meeting_providers: Vec<OutlookOnlineMeetingProvider>,
    default_online_meeting_provider: OutlookOnlineMeetingProvider,
    is_tallying_responses: bool,
    is_removable: bool,
    owner: OutlookCalendarOwner,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OutlookCalendarOwner {
    name: String,
    address: String,
}
