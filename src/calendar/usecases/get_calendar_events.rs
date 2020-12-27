use crate::calendar::domain::{calendar::Calendar, calendar_view::CalendarView};
use crate::calendar::repo::ICalendarRepo;
use crate::event::domain::event::CalendarEvent;
use crate::event::domain::event_instance::EventInstance;
use crate::event::repo::IEventRepo;
use crate::shared::usecase::UseCase;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct GetCalendarEventsReq {
    pub calendar_id: String,
    pub start_ts: i64,
    pub end_ts: i64,
}

pub struct GetCalendarEventsUseCase {
    pub event_repo: Arc<dyn IEventRepo>,
    pub calendar_repo: Arc<dyn ICalendarRepo>,
}

#[derive(Serialize)]
pub struct GetCalendarEventsResponse {
    calendar: Calendar,
    events: Vec<EventWithInstances>,
}

#[derive(Serialize)]
struct EventWithInstances {
    pub event: CalendarEvent,
    pub instances: Vec<EventInstance>,
}

#[async_trait(?Send)]
impl UseCase<GetCalendarEventsReq, Result<GetCalendarEventsResponse, GetCalendarEventsErrors>>
    for GetCalendarEventsUseCase
{
    async fn execute(
        &self,
        req: GetCalendarEventsReq,
    ) -> Result<GetCalendarEventsResponse, GetCalendarEventsErrors> {
        let calendar = self.calendar_repo.find(&req.calendar_id).await;

        let view = CalendarView::create(req.start_ts, req.end_ts);
        if view.is_err() {
            return Err(GetCalendarEventsErrors::InvalidTimespanError);
        }
        let view = view.unwrap();

        match calendar {
            Some(calendar) => {
                let events = self
                    .event_repo
                    .find_by_calendar(&calendar.id, Some(&view))
                    .await
                    .unwrap()
                    .into_iter()
                    .map(|event| {
                        let instances = event.expand(Some(&view));
                        EventWithInstances { event, instances }
                    })
                    // Also it is possible that there are no instances in the expanded event, should remove them
                    .filter(|data| !data.instances.is_empty())
                    .collect();

                Ok(GetCalendarEventsResponse { calendar, events })
            }
            None => Err(GetCalendarEventsErrors::NotFoundError {}),
        }
    }
}

// ERRORS

#[derive(Debug)]
pub enum GetCalendarEventsErrors {
    NotFoundError,
    InvalidTimespanError,
}

impl std::fmt::Display for GetCalendarEventsErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            GetCalendarEventsErrors::NotFoundError => write!(f, "The calendar was not found"),
            GetCalendarEventsErrors::InvalidTimespanError => {
                write!(f, "The provided timesspan was invalid.")
            }
        }
    }
}
