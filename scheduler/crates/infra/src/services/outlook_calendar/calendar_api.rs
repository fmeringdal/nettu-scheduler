use chrono::{TimeZone, Utc};
use nettu_scheduler_domain::CalendarEvent;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::warn;

const API_BASE_URL: &str = "https://graph.microsoft.com/v1.0/";

// https://docs.microsoft.com/en-us/graph/api/resources/datetimetimezone?view=graph-rest-1.0
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutlookCalendarEventTime {
    /// A single point of time in a combined date and time representation ({date}T{time}; for example, 2017-08-29T04:00:00.0000000).
    date_time: String,
    time_zone: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OutlookOnlineMeetingProvider {
    #[serde(rename = "teamsForBusiness")]
    BusinessTeams,
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
    id: String,
    start: OutlookCalendarEventTime,
    end: OutlookCalendarEventTime,
    subject: String,
    is_online_meeting: bool,
    online_meeting_provider: Option<OutlookOnlineMeetingProvider>,
    online_meeting: Option<OutlookCalendarEventOnlineMeeting>,
    show_as: OutlookCalendarEventShowAs,
    //     recurrence: Option<String>,
    body: OutlookCalendarEventBody,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutlookCalendarEventAttributes {
    start: OutlookCalendarEventTime,
    end: OutlookCalendarEventTime,
    subject: String,
    is_online_meeting: bool,
    online_meeting_provider: Option<OutlookOnlineMeetingProvider>,
    online_meeting: Option<OutlookCalendarEventOnlineMeeting>,
    show_as: OutlookCalendarEventShowAs,
    //     recurrence: Option<String>,
    body: OutlookCalendarEventBody,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OutlookCalendarEventBodyContentType {
    #[serde(rename = "HTML")]
    HTML,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutlookCalendarEventBody {
    content_type: OutlookCalendarEventBodyContentType,
    content: String,
}

impl Into<OutlookCalendarEventAttributes> for CalendarEvent {
    fn into(self) -> OutlookCalendarEventAttributes {
        let show_as = if self.busy {
            OutlookCalendarEventShowAs::Busy
        } else {
            OutlookCalendarEventShowAs::Free
        };

        let empty = "".to_string();
        let subject = self
            .metadata
            .get("outlook.subject")
            .unwrap_or(&empty)
            .clone();
        let content = self
            .metadata
            .get("outlook.content")
            .unwrap_or(&empty)
            .clone();
        OutlookCalendarEventAttributes {
            start: OutlookCalendarEventTime {
                time_zone: "UTC".to_string(),
                date_time: format!("{:?}", Utc.timestamp_millis(self.start_ts)),
            },
            end: OutlookCalendarEventTime {
                time_zone: "UTC".to_string(),
                date_time: format!("{:?}", Utc.timestamp_millis(self.end_ts)),
            },
            is_online_meeting: false,
            body: OutlookCalendarEventBody {
                content_type: OutlookCalendarEventBodyContentType::HTML,
                content,
            },
            online_meeting_provider: None,
            online_meeting: None,
            subject,
            show_as,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListCalendarsResponse {
    value: Vec<OutlookCalendar>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OutlookCalendar {
    id: String,
    name: String,
    color: String,
    change_key: String,
    can_share: bool,
    can_view_private_items: bool,
    hex_color: String,
    can_edit: bool,
    allowed_online_meeting_providers: Vec<OutlookOnlineMeetingProvider>,
    default_online_meeting_provider: OutlookOnlineMeetingProvider,
    is_tallying_responses: bool,
    is_removable: bool,
    owner: OutlookCalendarOwner,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OutlookCalendarOwner {
    name: String,
    address: String,
}

pub struct OutlookCalendarRestApi {
    client: Client,
    access_token: String,
}

impl OutlookCalendarRestApi {
    pub fn new(access_token: String) -> Self {
        let client = Client::new();

        Self {
            client,
            access_token,
        }
    }

    async fn put<T: for<'de> Deserialize<'de>>(
        &self,
        body: &impl Serialize,
        path: String,
    ) -> Result<T, ()> {
        match self
            .client
            .put(&format!("{}/{}", API_BASE_URL, path))
            .header("authorization", format!("Bearer {}", self.access_token))
            .json(body)
            .send()
            .await
        {
            Ok(res) => res.json::<T>().await.map_err(|e| {
                warn!("Outlook calendar api PUT deserialize error: {:?}", e);
                ()
            }),
            Err(e) => {
                warn!("Outlook calendar api PUT error: {:?}", e);
                Err(())
            }
        }
    }

    async fn post<T: for<'de> Deserialize<'de>>(
        &self,
        body: &impl Serialize,
        path: String,
    ) -> Result<T, ()> {
        match self
            .client
            .post(&format!("{}/{}", API_BASE_URL, path))
            .header("authorization", format!("Bearer {}", self.access_token))
            .json(body)
            .send()
            .await
        {
            Ok(res) => res.json::<T>().await.map_err(|e| {
                warn!("Outlook calendar api POST deserialize error: {:?}", e);
                ()
            }),
            Err(e) => {
                warn!("Outlook calendar api POST error: {:?}", e);
                Err(())
            }
        }
    }

    async fn delete<T: for<'de> Deserialize<'de>>(&self, path: String) -> Result<T, ()> {
        match self
            .client
            .delete(&format!("{}/{}", API_BASE_URL, path))
            .header("authorization", format!("Bearer {}", self.access_token))
            .send()
            .await
        {
            Ok(res) => res.json::<T>().await.map_err(|e| {
                warn!("Error: {:?}", e);
                ()
            }),
            Err(_) => Err(()),
        }
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, path: String) -> Result<T, ()> {
        match self
            .client
            .get(&format!("{}/{}", API_BASE_URL, path))
            .header("authorization", format!("Bearer {}", self.access_token))
            .send()
            .await
        {
            Ok(res) => res.json::<T>().await.map_err(|e| {
                warn!("Outlook calendar api GET deserialize error: {:?}", e);
                ()
            }),
            Err(e) => {
                warn!("Outlook calendar api get error: {:?}", e);
                Err(())
            }
        }
    }

    // TODO: add access role her
    pub async fn list(&self) -> Result<ListCalendarsResponse, ()> {
        self.get(format!("me/calendars")).await
    }

    pub async fn remove(&self, calendar_id: String, event_id: String) -> Result<(), ()> {
        self.delete(format!("me/calendars/{}/events/{}", calendar_id, event_id))
            .await
    }

    pub async fn update(
        &self,
        calendar_id: String,
        event_id: String,
        body: &OutlookCalendarEventAttributes,
    ) -> Result<OutlookCalendarEvent, ()> {
        self.put(
            body,
            format!("calendars/{}/events/{}", calendar_id, event_id),
        )
        .await
    }

    pub async fn insert(
        &self,
        calendar_id: String,
        body: &OutlookCalendarEventAttributes,
    ) -> Result<OutlookCalendarEvent, ()> {
        self.post(body, format!("me/calendars/{}/events", calendar_id))
            .await
    }

    pub async fn freebusy(&self, body: &FreeBusyRequest) -> Result<FreeBusyResponse, ()> {
        self.post(body, "me/calendar/getSchedule".into()).await
    }
}
