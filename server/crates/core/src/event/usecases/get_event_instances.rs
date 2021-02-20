use crate::{calendar::domain::CalendarView, event::domain::event::CalendarEvent};
use crate::{
    context::Context,
    event::domain::event_instance::EventInstance,
    shared::usecase::{execute, UseCase},
};
use serde::Serialize;

pub struct GetEventInstancesReqView {
    start_ts: i64,
    end_ts: i64,
}

pub struct GetEventInstancesUseCase {
    pub user_id: String,
    pub event_id: String,
    pub view: GetEventInstancesReqView,
}

#[derive(Debug)]
pub enum UseCaseErrors {
    NotFoundError,
    InvalidTimespanError,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UseCaseResponse {
    pub event: CalendarEvent,
    pub instances: Vec<EventInstance>,
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetEventInstancesUseCase {
    type Response = UseCaseResponse;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let e = ctx.repos.event_repo.find(&self.event_id).await;
        match e {
            Some(event) if self.user_id == event.user_id => {
                let calendar = match ctx.repos.calendar_repo.find(&event.calendar_id).await {
                    Some(cal) => cal,
                    None => return Err(UseCaseErrors::NotFoundError {}),
                };

                let view = CalendarView::create(self.view.start_ts, self.view.end_ts);
                if view.is_err() {
                    return Err(UseCaseErrors::InvalidTimespanError);
                }
                let instances = event.expand(Some(&view.unwrap()), &calendar.settings);
                Ok(UseCaseResponse { event, instances })
            }
            _ => Err(UseCaseErrors::NotFoundError {}),
        }
    }
}
