use crate::api::Context;
use crate::event::repos::IEventRepo;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize)]
pub struct EventPathParams {
    event_id: String,
}

pub async fn delete_event_controller(
    params: web::Path<EventPathParams>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let req = DeleteEventReq {
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
        Some(event) => {
            ctx.event_repo.delete(&event.id).await;
            Ok(())
        }
        None => Err(DeleteEventErrors::NotFoundError {}),
    }
}
