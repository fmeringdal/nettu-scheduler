use chrono::{SecondsFormat, TimeZone, Utc};
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
use tracing::error;

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

impl From<CalendarEvent> for OutlookCalendarEventAttributes {
    fn from(e: CalendarEvent) -> Self {
        let show_as = if e.busy {
            OutlookCalendarEventShowAs::Busy
        } else {
            OutlookCalendarEventShowAs::Free
        };

        let empty = "".to_string();
        let subject = e
            .metadata
            .inner
            .get("outlook.subject")
            .unwrap_or(&empty)
            .clone();
        let content = e
            .metadata
            .inner
            .get("outlook.content")
            .unwrap_or(&empty)
            .clone();
        OutlookCalendarEventAttributes {
            start: OutlookCalendarEventTime {
                time_zone: "UTC".to_string(),
                date_time: format!("{}", Utc.timestamp_millis(e.start_ts).format("%+")),
            },
            end: OutlookCalendarEventTime {
                time_zone: "UTC".to_string(),
                date_time: format!("{}", Utc.timestamp_millis(e.end_ts).format("%+")),
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

pub type ListCalendarsResponse = Vec<OutlookCalendar>;

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
    ) -> anyhow::Result<T> {
        match self
            .client
            .put(&format!("{}/{}", API_BASE_URL, path))
            .header("authorization", format!("Bearer {}", self.access_token))
            .json(body)
            .send()
            .await
        {
            Ok(res) => res.json::<T>().await.map_err(|e| {
                error!(
                    "[Unexpected Response] Outlook Calendar API PUT error. Error message: {:?}",
                    e
                );
                anyhow::Error::new(e)
            }),
            Err(e) => {
                error!(
                    "[Network Error] Outlook Calendar API PUT error. Error message: {:?}",
                    e
                );
                Err(anyhow::Error::new(e))
            }
        }
    }

    async fn post<T: for<'de> Deserialize<'de>>(
        &self,
        body: &impl Serialize,
        path: String,
    ) -> anyhow::Result<T> {
        match self
            .client
            .post(&format!("{}/{}", API_BASE_URL, path))
            .header("authorization", format!("Bearer {}", self.access_token))
            .json(body)
            .send()
            .await
        {
            Ok(res) => res.json::<T>().await.map_err(|e| {
                error!(
                    "[Unexpected Response] Outlook Calendar API POST error. Error message: {:?}",
                    e
                );
                anyhow::Error::new(e)
            }),
            Err(e) => {
                error!(
                    "[Network Error] Outlook Calendar API POST error. Error message: {:?}",
                    e
                );
                Err(anyhow::Error::new(e))
            }
        }
    }

    async fn delete<T: for<'de> Deserialize<'de>>(&self, path: String) -> anyhow::Result<T> {
        match self
            .client
            .delete(&format!("{}/{}", API_BASE_URL, path))
            .header("authorization", format!("Bearer {}", self.access_token))
            .send()
            .await
        {
            Ok(res) => res.json::<T>().await.map_err(|e| {
                error!(
                    "[Unexpected Response] Outlook Calendar API DELETE error. Error message: {:?}",
                    e
                );
                anyhow::Error::new(e)
            }),
            Err(e) => {
                error!(
                    "[Network Error] Outlook Calendar API DELETE error. Error message: {:?}",
                    e
                );
                Err(anyhow::Error::new(e))
            }
        }
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, path: String) -> anyhow::Result<T> {
        match self
            .client
            .get(&format!("{}/{}", API_BASE_URL, path))
            .header("authorization", format!("Bearer {}", self.access_token))
            .send()
            .await
        {
            Ok(res) => res.json::<T>().await.map_err(|e| {
                error!(
                    "[Unexpected Response] Outlook Calendar API GET error. Error message: {:?}",
                    e
                );
                anyhow::Error::new(e)
            }),
            Err(e) => {
                error!(
                    "[Network Error] Outlook Calendar API GET error. Error message: {:?}",
                    e
                );
                Err(anyhow::Error::new(e))
            }
        }
    }

    pub async fn list(&self) -> Result<ListCalendarsResponse, ()> {
        self.get("me/calendars".to_string()).await.map_err(|e| {
            error!("Failed to list outlook calendars. Error message: {:?}", e);
        })
    }

    pub async fn remove(&self, calendar_id: String, event_id: String) -> Result<(), ()> {
        self.delete(format!("me/calendars/{}/events/{}", calendar_id, event_id))
            .await
            .map_err(|e| {
                error!("Failed to delete outlook calendar event with outlook calendar id: {} and outlook event id: {}. Error message: {:?}", calendar_id, event_id, e);
            })
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
            .map_err(|e| {
                error!("Failed to update outlook calendar event in outlook calendar id: {} and outlook event id: {} and with body: {:?}. Error message: {:?}", calendar_id, event_id, body, e);
            })
    }

    pub async fn insert(
        &self,
        calendar_id: String,
        body: &OutlookCalendarEventAttributes,
    ) -> Result<OutlookCalendarEvent, ()> {
        self.post(body, format!("me/calendars/{}/events", calendar_id))
            .await
            .map_err(|e| {
                error!("Failed to insert outlook calendar event to outlook calendar id: {} with body: {:?}. Error message: {:?}", calendar_id, body, e);
            })
    }

    pub async fn freebusy(&self, body: &FreeBusyRequest) -> Result<CompatibleInstances, ()> {
        let cal_futures = body
            .calendars
            .iter()
            .map(|calendar_id| {
                self.get::<CalendarViewResponse>(format!(
                    "me/calendars/{}/calendarView?startDateTime={}&endDateTime={}",
                    calendar_id,
                    // format!("{}", Utc.timestamp_millis(body.time_min).format("%+")),
                    // format!("{}", Utc.timestamp_millis(body.time_max).format("%+"))
                    Utc.timestamp_millis(body.time_min)
                        .to_rfc3339_opts(SecondsFormat::Secs, true),
                    Utc.timestamp_millis(body.time_max)
                        .to_rfc3339_opts(SecondsFormat::Secs, true),
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
                    .filter(|e| matches!(e.show_as, OutlookCalendarEventShowAs::Busy))
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
