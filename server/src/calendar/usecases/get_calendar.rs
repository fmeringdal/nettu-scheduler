use crate::calendar::repos::ICalendarRepo;
use crate::{api::Context, calendar::domain::calendar::Calendar};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub async fn get_calendar_controller(
    req: web::Path<GetCalendarReq>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let ctx = GetCalendarUseCaseCtx {
        calendar_repo: ctx.repos.calendar_repo.clone(),
    };

    let res = get_calendar_usecase(req.0, ctx).await;
    match res {
        Ok(cal) => HttpResponse::Ok().json(cal),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

#[derive(Serialize, Deserialize)]
pub struct GetCalendarReq {
    pub calendar_id: String,
}

pub struct GetCalendarUseCaseCtx {
    pub calendar_repo: Arc<dyn ICalendarRepo>,
}

pub enum GetCalendarErrors {
    NotFoundError,
}

async fn get_calendar_usecase(
    req: GetCalendarReq,
    ctx: GetCalendarUseCaseCtx,
) -> Result<Calendar, GetCalendarErrors> {
    let cal = ctx.calendar_repo.find(&req.calendar_id).await;
    match cal {
        Some(cal) => Ok(cal),
        None => Err(GetCalendarErrors::NotFoundError {}),
    }
}
