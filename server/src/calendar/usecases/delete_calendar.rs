use crate::api::Context;
use crate::calendar::repo::ICalendarRepo;
use crate::event::repo::IEventRepo;
use actix_web::{web, HttpResponse};

use serde::Deserialize;

use std::sync::Arc;

#[derive(Deserialize)]
pub struct DeleteCalendarReq {
    calendar_id: String,
}

pub async fn delete_calendar_controller(
    req: web::Path<DeleteCalendarReq>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let ctx = DeleteCalendarUseCaseCtx {
        calendar_repo: Arc::clone(&ctx.repos.calendar_repo),
        event_repo: Arc::clone(&ctx.repos.event_repo),
    };
    let res = delete_calendar_usecase(req.0, ctx).await;
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

async fn delete_calendar_usecase(
    req: DeleteCalendarReq,
    ctx: DeleteCalendarUseCaseCtx,
) -> Result<(), DeleteCalendarErrors> {
    let calendar = ctx.calendar_repo.find(&req.calendar_id).await;
    match calendar {
        Some(calendar) => {
            ctx.calendar_repo.delete(&calendar.id).await;
            ctx.event_repo.delete_by_calendar(&calendar.id).await;
            Ok(())
        }
        None => Err(DeleteCalendarErrors::NotFoundError {}),
    }
}
