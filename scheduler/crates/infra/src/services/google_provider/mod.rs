// CONSIDER USING THIS LIB: https://docs.rs/google-calendar3
mod auth_provider;
mod calendar_api;

use super::FreeBusyProviderQuery;
use calendar_api::{
    FreeBusyCalendar, FreeBusyRequest, GoogleCalendarEvent, GoogleCalendarEventDateTime,
    GoogleCalendarRestApi, GoogleDateTime,
};
use nettu_scheduler_domain::{CalendarEvent, CompatibleInstances, EventInstance};

// https://developers.google.com/calendar/v3/reference/events

pub struct GoogleCalendarProvider {
    api: GoogleCalendarRestApi,
}

impl GoogleCalendarProvider {
    async fn freebusy(&self, query: FreeBusyProviderQuery) -> CompatibleInstances {
        let body = FreeBusyRequest {
            time_max: GoogleDateTime::from_timestamp_millis(query.start),
            time_min: GoogleDateTime::from_timestamp_millis(query.end),
            time_zone: "UTC".to_string(),
            items: query
                .calendar_ids
                .into_iter()
                .map(FreeBusyCalendar::new)
                .collect(),
        };
        let mut instances = vec![];
        if let Ok(res) = self.api.freebusy(&body).await {
            for (_, calendar_busy) in res.calendars {
                for instance in calendar_busy.busy {
                    let instance = EventInstance {
                        start_ts: instance.start.get_timestamp_millis(),
                        end_ts: instance.end.get_timestamp_millis(),
                        busy: true,
                    };
                    instances.push(instance);
                }
            }
        }
        CompatibleInstances::new(instances)
    }

    async fn create_event(
        &self,
        calendar_id: String,
        event: CalendarEvent,
    ) -> Result<GoogleCalendarEvent, ()> {
        let google_calendar_event: GoogleCalendarEvent = event.into();
        self.api.insert(calendar_id, &google_calendar_event).await
    }

    async fn delete_event(&self, event: &CalendarEvent) -> Result<(), ()> {
        for synced_event in &event.synced_events {
            match synced_event.provider {
                nettu_scheduler_domain::SyncedCalendarProvider::Google => {
                    return self
                        .api
                        .remove(
                            synced_event.calendar_id.clone(),
                            synced_event.event_id.clone(),
                        )
                        .await;
                }
                _ => (),
            }
        }
        Ok(())
    }

    async fn list(&self) -> Vec<()> {
        todo!()
    }
}
