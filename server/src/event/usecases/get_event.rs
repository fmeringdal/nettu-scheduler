use crate::{
    api::{Context, NettuError},
    event::domain::event::CalendarEvent,
    shared::{
        auth::protect_route,
        usecase::{execute, UseCase},
    },
};
use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PathParams {
    event_id: String,
}

pub async fn get_event_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    ctx: web::Data<Context>,
) -> Result<HttpResponse, NettuError> {
    let (user, policy) = protect_route(&http_req, &ctx).await?;

    let usecase = GetEventUseCase {
        event_id: path_params.event_id.clone(),
        user_id: user.id.clone(),
    };

    execute(usecase, &ctx)
        .await
        .map(|calendar_event| HttpResponse::Ok().json(calendar_event))
        .map_err(|e| match e {
            UseCaseErrors::NotFoundError => NettuError::NotFound(format!(
                "The event with id: {}, was not found",
                path_params.event_id
            )),
        })
}

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
