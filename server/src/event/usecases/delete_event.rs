use crate::{api::Context, shared::auth::protect_route};
use crate::event::repos::IEventRepo;
use actix_web::{HttpRequest, HttpResponse, web};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize)]
pub struct EventPathParams {
    event_id: String,
}

pub async fn delete_event_controller(
    http_req: HttpRequest,
    params: web::Path<EventPathParams>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let user = match protect_route(&http_req) {
        Ok(u) => u,
        Err(res) => return res
    };

    let req = DeleteEventReq {
        user_id: user.id.clone(),
        event_id: params.event_id.clone(),
    };
    let ctx = DeleteEventUseCaseCtx {
        event_repo: ctx.repos.event_repo.clone(),
    };
    let res = delete_event_usecase(req, ctx).await;
    return match res {
        Ok(_) => HttpResponse::Ok().body("Event deleted"),
        Err(_) => HttpResponse::NotFound().finish(),
    };
}

#[derive(Serialize, Deserialize)]
pub struct DeleteEventReq {
    pub user_id: String,
    pub event_id: String,
}

pub struct DeleteEventUseCaseCtx {
    pub event_repo: Arc<dyn IEventRepo>,
}
pub enum DeleteEventErrors {
    NotFoundError,
}

// TODO: use only one db call
async fn delete_event_usecase(
    req: DeleteEventReq,
    ctx: DeleteEventUseCaseCtx,
) -> Result<(), DeleteEventErrors> {
    let e = ctx.event_repo.find(&req.event_id).await;
    match e {
        Some(event) if event.user_id == req.user_id => {
            ctx.event_repo.delete(&event.id).await;
            Ok(())
        }
        _ => Err(DeleteEventErrors::NotFoundError {}),
    }
}
