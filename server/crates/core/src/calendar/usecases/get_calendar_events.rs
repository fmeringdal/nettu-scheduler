use crate::event::domain::event_instance::EventInstance;
use crate::{
    calendar::domain::{Calendar, CalendarView},
    context::Context,
    event::domain::event::CalendarEvent,
    shared::usecase::{execute, UseCase},
};
use serde::Serialize;

pub struct GetCalendarEventsUseCase {
    pub calendar_id: String,
    pub user_id: String,
    pub start_ts: i64,
    pub end_ts: i64,
}

pub struct UseCaseResponse {
    calendar: Calendar,
    events: Vec<EventWithInstances>,
}

#[derive(Serialize)]
pub struct EventWithInstances {
    pub event: CalendarEvent,
    pub instances: Vec<EventInstance>,
}

#[derive(Debug)]
pub enum UseCaseErrors {
    NotFoundError,
    InvalidTimespanError,
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetCalendarEventsUseCase {
    type Response = UseCaseResponse;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let calendar = ctx.repos.calendar_repo.find(&self.calendar_id).await;

        let view = CalendarView::create(self.start_ts, self.end_ts);
        if view.is_err() {
            return Err(UseCaseErrors::InvalidTimespanError);
        }
        let view = view.unwrap();

        match calendar {
            Some(calendar) if calendar.user_id == self.user_id => {
                let events = ctx
                    .repos
                    .event_repo
                    .find_by_calendar(&calendar.id, Some(&view))
                    .await
                    .unwrap()
                    .into_iter()
                    .map(|event| {
                        let instances = event.expand(Some(&view), &calendar.settings);
                        EventWithInstances { event, instances }
                    })
                    // Also it is possible that there are no instances in the expanded event, should remove them
                    .filter(|data| !data.instances.is_empty())
                    .collect();

                Ok(UseCaseResponse { calendar, events })
            }
            _ => Err(UseCaseErrors::NotFoundError),
        }
    }
}
