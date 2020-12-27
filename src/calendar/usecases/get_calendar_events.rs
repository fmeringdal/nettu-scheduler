use crate::calendar::domain::calendar::Calendar;
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

pub enum GetCalendarEventsErrors {
    NotFoundError,
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
        match calendar {
            Some(calendar) => {
                let events = self
                    .event_repo
                    .find_by_calendar(&calendar.id)
                    .await
                    .unwrap()
                    .into_iter()
                    .map(|event| {
                        let instances = event.expand();
                        EventWithInstances { event, instances }
                    })
                    .collect();

                Ok(GetCalendarEventsResponse { calendar, events })
            }
            None => Err(GetCalendarEventsErrors::NotFoundError {}),
        }
    }
}
