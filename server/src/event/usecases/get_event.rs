use crate::{
    api::Context,
    event::{domain::event::CalendarEvent, repos::IEventRepo},
    shared::{
        auth::protect_route,
        usecase::{perform, Usecase},
    },
};
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize)]
pub struct PathParams {
    event_id: String,
}

pub async fn get_event_controller(
    http_req: HttpRequest,
    params: web::Path<PathParams>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let user = match protect_route(&http_req, &ctx).await {
        Ok(u) => u,
        Err(res) => return res,
    };

    let usecase = GetEventUseCase {
        event_id: params.event_id.clone(),
        user_id: user.id.clone(),
    };
    let res = perform(usecase, &ctx).await;
    match res {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(_) => HttpResponse::NotFound().finish(),
    }
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
impl Usecase for GetEventUseCase {
    type Response = CalendarEvent;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn perform(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let e = ctx.repos.event_repo.find(&self.event_id).await;
        match e {
            Some(event) if event.user_id == self.user_id => Ok(event),
            _ => Err(UseCaseErrors::NotFoundError),
        }
    }
}
