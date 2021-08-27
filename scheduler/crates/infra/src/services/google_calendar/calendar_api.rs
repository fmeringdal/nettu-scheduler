use chrono::{DateTime, TimeZone, Utc};
use nettu_scheduler_domain::providers::google::*;
use nettu_scheduler_domain::CalendarEvent;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::error;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleCalendarEventDateTime {
    date_time: GoogleDateTime,
    time_zone: String,
}

impl GoogleCalendarEventDateTime {
    pub fn new(date_time_millis: i64) -> Self {
        Self {
            date_time: GoogleDateTime::from_timestamp_millis(date_time_millis),
            time_zone: String::from("UTC"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleCalendarEvent {
    pub id: String,
    pub start: GoogleCalendarEventDateTime,
    pub end: GoogleCalendarEventDateTime,
    pub summary: String,
    pub description: String,
    #[serde(default)]
    pub transparency: Option<String>,
    #[serde(default)]
    pub recurrence: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleCalendarEventAttributes {
    pub start: GoogleCalendarEventDateTime,
    pub end: GoogleCalendarEventDateTime,
    pub summary: String,
    pub description: String,
    pub transparency: String,
    pub recurrence: Vec<String>,
}

impl From<CalendarEvent> for GoogleCalendarEventAttributes {
    fn from(e: CalendarEvent) -> Self {
        let empty = "".to_string();
        let summary = e
            .metadata
            .inner
            .get("google.summary")
            .unwrap_or(&empty)
            .clone();
        let description = e
            .metadata
            .inner
            .get("google.description")
            .unwrap_or(&empty)
            .clone();
        let transparency = if e.busy {
            "opaque".to_string()
        } else {
            "transparent".to_string()
        };
        Self {
            description,
            summary,
            start: GoogleCalendarEventDateTime::new(e.start_ts),
            // Recurrence sync not supported yet, so e.end_ts will not be correct if used
            end: GoogleCalendarEventDateTime::new(e.start_ts + e.duration),
            // Recurrence sync not supported yet
            recurrence: Vec::new(),
            // Whether it blocks calendar time or not
            transparency,
        }
    }
}

pub struct GoogleCalendarRestApi {
    client: Client,
    access_token: String,
}

impl GoogleCalendarRestApi {
    pub fn new(access_token: String) -> Self {
        let client = Client::new();

        Self {
            client,
            access_token,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleDateTime(String);

impl GoogleDateTime {
    pub fn from_timestamp_millis(timestamp: i64) -> Self {
        let datetime_str = Utc.timestamp_millis(timestamp).to_rfc3339();
        Self(datetime_str)
    }

    pub fn get_timestamp_millis(&self) -> i64 {
        DateTime::parse_from_rfc3339(&self.0)
            .expect("Inner string to always be valid RFC3339 string")
            .timestamp_millis()
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FreeBusyCalendarResponse {
    pub busy: Vec<FreeBusyTimeSpanResponse>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FreeBusyTimeSpanResponse {
    pub start: GoogleDateTime,
    pub end: GoogleDateTime,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FreeBusyResponse {
    kind: String,
    time_min: GoogleDateTime,
    time_max: GoogleDateTime,
    pub calendars: HashMap<String, FreeBusyCalendarResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FreeBusyCalendar {
    pub id: String,
}

impl FreeBusyCalendar {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FreeBusyRequest {
    pub time_min: GoogleDateTime,
    pub time_max: GoogleDateTime,
    pub time_zone: String,
    pub items: Vec<FreeBusyCalendar>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListCalendarsResponse {
    kind: String,
    etag: GoogleDateTime,
    pub items: Vec<GoogleCalendarListEntry>,
}

const GOOGLE_API_BASE_URL: &str = "https://www.googleapis.com/calendar/v3";

impl GoogleCalendarRestApi {
    async fn put<T: for<'de> Deserialize<'de>>(
        &self,
        body: &impl Serialize,
        path: String,
    ) -> anyhow::Result<T> {
        match self
            .client
            .put(&format!("{}/{}", GOOGLE_API_BASE_URL, path))
            .header("authorization", format!("Bearer {}", self.access_token))
            .json(body)
            .send()
            .await
        {
            Ok(res) => res.json::<T>().await.map_err(|e| {
                error!(
                    "[Unexpected Response] Google Calendar API PUT error. Error message: {:?}",
                    e
                );
                anyhow::Error::new(e)
            }),
            Err(e) => {
                error!(
                    "[Network Error] Google Calendar API PUT error. Error message: {:?}",
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
            .post(&format!("{}/{}", GOOGLE_API_BASE_URL, path))
            .header("authorization", format!("Bearer {}", self.access_token))
            .json(body)
            .send()
            .await
        {
            Ok(res) => res.json::<T>().await.map_err(|e| {
                error!(
                    "[Unexpected Response] Google Calendar API POST error. Error message: {:?}",
                    e
                );
                anyhow::Error::new(e)
            }),
            Err(e) => {
                error!(
                    "[Network Error] Google Calendar API POST error. Error message: {:?}",
                    e
                );
                Err(anyhow::Error::new(e))
            }
        }
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, path: String) -> anyhow::Result<T> {
        match self
            .client
            .get(&format!("{}/{}", GOOGLE_API_BASE_URL, path))
            .header("authorization", format!("Bearer {}", self.access_token))
            .send()
            .await
        {
            Ok(res) => res.json::<T>().await.map_err(|e| {
                error!(
                    "[Unexpected Response] Google Calendar API GET error. Error message: {:?}",
                    e
                );
                anyhow::Error::new(e)
            }),
            Err(e) => {
                error!(
                    "[Network Error] Google Calendar API GET error. Error message: {:?}",
                    e
                );
                Err(anyhow::Error::new(e))
            }
        }
    }

    async fn delete<T: for<'de> Deserialize<'de>>(&self, path: String) -> anyhow::Result<T> {
        match self
            .client
            .delete(&format!("{}/{}", GOOGLE_API_BASE_URL, path))
            .header("authorization", format!("Bearer {}", self.access_token))
            .send()
            .await
        {
            Ok(res) => res.json::<T>().await.map_err(|e| {
                error!(
                    "[Unexpected Response] Google Calendar API DELETE error. Error message: {:?}",
                    e
                );
                anyhow::Error::new(e)
            }),
            Err(e) => {
                error!(
                    "[Network Error] Google Calendar API DELETE error. Error message: {:?}",
                    e
                );
                Err(anyhow::Error::new(e))
            }
        }
    }

    pub async fn freebusy(&self, body: &FreeBusyRequest) -> Result<FreeBusyResponse, ()> {
        self.post(body, "freeBusy".into()).await
            .map_err(|e| {
                error!("Failed to get freebusy from google calendar with request: {:?}. Error message: {:?}", body, e);
            })
    }

    pub async fn insert(
        &self,
        calendar_id: String,
        body: &GoogleCalendarEventAttributes,
    ) -> Result<GoogleCalendarEvent, ()> {
        self.post(body, format!("calendars/{}/events", calendar_id))
            .await
            .map_err(|e| {
                error!("Failed to insert google calendar event to google calendar id: {} with body: {:?}. Error message: {:?}", calendar_id, body, e);
            })
    }

    pub async fn update(
        &self,
        calendar_id: String,
        event_id: String,
        body: &GoogleCalendarEventAttributes,
    ) -> Result<GoogleCalendarEvent, ()> {
        self.put(
            body,
            format!("calendars/{}/events/{}", calendar_id, event_id),
        )
        .await
            .map_err(|e| {
                error!("Failed to update google calendar event in google calendar id: {} and google event id: {} and with body: {:?}. Error message: {:?}", calendar_id, event_id, body, e);
            })
    }

    pub async fn remove(&self, calendar_id: String, event_id: String) -> Result<(), ()> {
        self.delete(format!("calendars/{}/events/{}", calendar_id, event_id))
            .await
                .map_err(|e| {
                error!("Failed to delete google calendar event with google calendar id: {} and google event id: {}. Error message: {:?}", calendar_id, event_id, e);
            })
    }

    pub async fn list(
        &self,
        min_access_role: GoogleCalendarAccessRole,
    ) -> Result<ListCalendarsResponse, ()> {
        self.get(format!(
            "users/me/calendarList?minAccessRole={:?}",
            min_access_role
        ))
        .await
        .map_err(|e| {
            error!(
                "Failed to list google calendars with access role: {:?}. Error message: {:?}",
                min_access_role, e
            );
        })
    }
}
