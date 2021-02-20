use crate::{context::Context, event::domain::event::CalendarEvent, shared::usecase::UseCase};

pub struct GetEventUseCase {
    pub event_id: String,
    pub user_id: String,
}

#[derive(Debug)]
pub enum UseCaseErrors {
    NotFoundError,
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetEventUseCase {
    type Response = CalendarEvent;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let e = ctx.repos.event_repo.find(&self.event_id).await;
        match e {
            Some(event) if event.user_id == self.user_id => Ok(event),
            _ => Err(UseCaseErrors::NotFoundError),
        }
    }
}
