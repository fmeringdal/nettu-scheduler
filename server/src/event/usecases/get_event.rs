use crate::{
    api::Context,
    event::{domain::event::CalendarEvent, repos::IEventRepo},
};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize)]
pub struct EventPathParams {
    event_id: String,
}

pub async fn get_event_controller(
    params: web::Path<EventPathParams>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let req = GetEventReq {
        event_id: params.event_id.clone(),
    };
    let ctx = GetEventUseCaseCtx {
        event_repo: ctx.repos.event_repo.clone(),
    };
    let res = get_event_usecase(req, ctx).await;
    return match res {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(_) => HttpResponse::NotFound().finish(),
    };
}

#[derive(Serialize, Deserialize)]
pub struct GetEventReq {
    pub event_id: String,
}

pub struct GetEventUseCaseCtx {
    pub event_repo: Arc<dyn IEventRepo>,
}

pub enum GetEventErrors {
    NotFoundError,
}
async fn get_event_usecase(
    req: GetEventReq,
    ctx: GetEventUseCaseCtx,
) -> Result<CalendarEvent, GetEventErrors> {
    let e = ctx.event_repo.find(&req.event_id).await;
    match e {
        Some(event) => Ok(event),
        None => Err(GetEventErrors::NotFoundError {}),
    }
}
