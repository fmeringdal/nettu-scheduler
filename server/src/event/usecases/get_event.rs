use crate::{
    api::Context,
    event::{domain::event::CalendarEvent, repos::IEventRepo},
    shared::auth::{protect_route, AuthContext},
};
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize)]
pub struct EventPathParams {
    event_id: String,
}

pub async fn get_event_controller(
    http_req: HttpRequest,
    params: web::Path<EventPathParams>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let user = match protect_route(
        &http_req,
        &AuthContext {
            account_repo: Arc::clone(&ctx.repos.account_repo),
            user_repo: Arc::clone(&ctx.repos.user_repo),
        },
    )
    .await
    {
        Ok(u) => u,
        Err(res) => return res,
    };

    let req = GetEventReq {
        event_id: params.event_id.clone(),
        user_id: user.id.clone(),
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
    pub user_id: String,
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
        Some(event) if event.user_id == req.user_id => Ok(event),
        _ => Err(GetEventErrors::NotFoundError {}),
    }
}
