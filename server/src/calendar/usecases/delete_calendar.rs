use crate::{api::Context, shared::auth::protect_route};
use crate::calendar::repos::ICalendarRepo;
use crate::event::repos::IEventRepo;
use actix_web::{HttpRequest, HttpResponse, web};

use serde::Deserialize;

use std::sync::Arc;

#[derive(Deserialize)]
pub struct DeleteCalendarReq {
    calendar_id: String,
}

pub async fn delete_calendar_controller(
    http_req: HttpRequest,
    req: web::Path<DeleteCalendarReq>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let user = match protect_route(&http_req) {
        Ok(u) => u,
        Err(res) => return res
    };
    let ctx = DeleteCalendarUseCaseCtx {
        calendar_repo: Arc::clone(&ctx.repos.calendar_repo),
        event_repo: Arc::clone(&ctx.repos.event_repo),
    };
    let req = DeleteCalendarUseCaseReq {
        user_id: user.id.clone(),
        calendar_id: req.calendar_id.clone()
    };
    let res = delete_calendar_usecase(req, ctx).await;
    return match res {
        Ok(_) => HttpResponse::Ok().body("Calendar deleted"),
        Err(_) => HttpResponse::NotFound().finish(),
    };
}

pub enum DeleteCalendarErrors {
    NotFoundError,
}

pub struct DeleteCalendarUseCaseCtx {
    pub calendar_repo: Arc<dyn ICalendarRepo>,
    pub event_repo: Arc<dyn IEventRepo>,
}

pub struct DeleteCalendarUseCaseReq {
    calendar_id: String,
    user_id: String
}

async fn delete_calendar_usecase(
    req: DeleteCalendarUseCaseReq,
    ctx: DeleteCalendarUseCaseCtx,
) -> Result<(), DeleteCalendarErrors> {
    let calendar = ctx.calendar_repo.find(&req.calendar_id).await;
    match calendar {
        Some(calendar) if calendar.user_id.eq(&req.user_id) => {
            ctx.calendar_repo.delete(&calendar.id).await;
            ctx.event_repo.delete_by_calendar(&calendar.id).await;
            Ok(())
        }
        _ => Err(DeleteCalendarErrors::NotFoundError {}),
    }
}
