use crate::{calendar::repos::ICalendarRepo, shared::auth::protect_route};
use crate::{api::Context, calendar::domain::calendar::Calendar};
use actix_web::{HttpRequest, HttpResponse, web};
use serde::{Deserialize, Serialize};
use std::sync::Arc;


#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCalendarReq {
    pub calendar_id: String,
}

pub async fn get_calendar_controller(
    http_req: HttpRequest,
    req: web::Path<GetCalendarReq>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let user = match protect_route(&http_req) {
        Ok(u) => u,
        Err(res) => return res
    };
    let ctx = GetCalendarUseCaseCtx {
        calendar_repo: ctx.repos.calendar_repo.clone(),
    };
    let req = GetCalendarUseCaseReq {
        user_id: user.id.clone(),
        calendar_id: req.calendar_id.clone(),
    };

    let res = get_calendar_usecase(req, ctx).await;
    match res {
        Ok(cal) => HttpResponse::Ok().json(cal),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}


pub struct GetCalendarUseCaseReq {
    pub user_id: String,
    pub calendar_id: String,
}

pub struct GetCalendarUseCaseCtx {
    pub calendar_repo: Arc<dyn ICalendarRepo>,
}

pub enum GetCalendarErrors {
    NotFoundError,
}

async fn get_calendar_usecase(
    req: GetCalendarUseCaseReq,
    ctx: GetCalendarUseCaseCtx,
) -> Result<Calendar, GetCalendarErrors> {
    let cal = ctx.calendar_repo.find(&req.calendar_id).await;
    match cal {
        Some(cal) if cal.user_id == req.user_id => Ok(cal),
        _ => Err(GetCalendarErrors::NotFoundError {}),
    }
}
