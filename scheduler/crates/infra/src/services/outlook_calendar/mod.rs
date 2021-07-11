use crate::NettuContext;
pub mod auth_provider;
mod calendar_api;

use self::calendar_api::{FreeBusyRequest, ListCalendarsResponse, OutlookCalendarEventAttributes};
use super::FreeBusyProviderQuery;
use calendar_api::OutlookCalendarRestApi;
use nettu_scheduler_domain::{
    providers::outlook::{OutlookCalendarAccessRole, OutlookCalendarEvent},
    CalendarEvent, CompatibleInstances, User,
};

// https://docs.microsoft.com/en-us/graph/api/resources/event?view=graph-rest-1.0

pub struct OutlookCalendarProvider {
    api: OutlookCalendarRestApi,
}

impl OutlookCalendarProvider {
    pub async fn new(user: &mut User, ctx: &NettuContext) -> Result<Self, ()> {
        let access_token = match auth_provider::get_access_token(user, ctx).await {
            Some(token) => token,
            None => return Err(()),
        };
        Ok(Self {
            api: OutlookCalendarRestApi::new(access_token),
        })
    }

    pub async fn freebusy(&self, query: FreeBusyProviderQuery) -> CompatibleInstances {
        let body = FreeBusyRequest {
            time_min: query.start,
            time_max: query.end,
            time_zone: "UTC".to_string(),
            calendars: query.calendar_ids,
        };
        self.api.freebusy(&body).await.unwrap_or_default()
    }

    pub async fn create_event(
        &self,
        calendar_id: String,
        event: CalendarEvent,
    ) -> Result<OutlookCalendarEvent, ()> {
        let google_calendar_event: OutlookCalendarEventAttributes = event.into();
        self.api.insert(calendar_id, &google_calendar_event).await
    }

    pub async fn update_event(
        &self,
        calendar_id: String,
        event_id: String,
        event: CalendarEvent,
    ) -> Result<OutlookCalendarEvent, ()> {
        let google_calendar_event: OutlookCalendarEventAttributes = event.into();
        self.api
            .update(calendar_id, event_id, &google_calendar_event)
            .await
    }

    pub async fn delete_event(&self, calendar_id: String, event_id: String) -> Result<(), ()> {
        self.api.remove(calendar_id, event_id).await
    }

    pub async fn list(
        &self,
        min_access_role: OutlookCalendarAccessRole,
    ) -> Result<ListCalendarsResponse, ()> {
        let mut calendars = self.api.list().await?;
        calendars.value.retain(|cal| match min_access_role {
            OutlookCalendarAccessRole::Reader => true,
            OutlookCalendarAccessRole::Writer => cal.can_edit,
        });

        Ok(calendars)
    }
}
