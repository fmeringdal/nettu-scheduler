use chrono::{TimeZone, Utc};
use futures::future::join_all;
use nettu_scheduler_domain::{
    providers::outlook::{
        OutlookCalendar, OutlookCalendarEvent, OutlookCalendarEventBody,
        OutlookCalendarEventBodyContentType, OutlookCalendarEventOnlineMeeting,
        OutlookCalendarEventShowAs, OutlookCalendarEventTime, OutlookOnlineMeetingProvider,
    },
    CalendarEvent, CompatibleInstances, EventInstance,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::warn;

const API_BASE_URL: &str = "https://graph.microsoft.com/v1.0/";

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
                date_time: format!("{}", Utc.timestamp_millis(self.start_ts).format("%+")),
            },
            end: OutlookCalendarEventTime {
                time_zone: "UTC".to_string(),
                date_time: format!("{}", Utc.timestamp_millis(self.end_ts).format("%+")),
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
pub struct ListCalendarsResponse {
    pub value: Vec<OutlookCalendar>,
}

pub struct OutlookCalendarRestApi {
    client: Client,
    access_token: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FreeBusyRequest {
    pub time_min: i64,
    pub time_max: i64,
    pub time_zone: String,
    pub calendars: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarViewResponse {
    pub value: Vec<OutlookCalendarEvent>,
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

    pub async fn freebusy(&self, body: &FreeBusyRequest) -> Result<CompatibleInstances, ()> {
        let cal_futures = body
            .calendars
            .iter()
            .map(|calendar_id| {
                self.get::<CalendarViewResponse>(format!(
                    "me/calendars/{}/calendarView?startDateTime={}&endDateTime={}",
                    calendar_id,
                    format!("{}", Utc.timestamp_millis(body.time_min).format("%+")),
                    format!("{}", Utc.timestamp_millis(body.time_max).format("%+"))
                ))
            })
            .collect::<Vec<_>>();
        let calendar_views = join_all(cal_futures)
            .await
            .into_iter()
            .filter_map(|res| res.ok())
            .map(|view| {
                view.value
                    .into_iter()
                    .filter(|e| match e.show_as {
                        OutlookCalendarEventShowAs::Busy => true,
                        _ => false,
                    })
                    .map(|e| EventInstance {
                        busy: true,
                        start_ts: e.start.get_timestamp_millis(),
                        end_ts: e.end.get_timestamp_millis(),
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect::<Vec<_>>();
        Ok(CompatibleInstances::new(calendar_views))
    }
}
