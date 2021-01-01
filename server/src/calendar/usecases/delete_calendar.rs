use crate::event::repos::IEventRepo;
use crate::{api::Context, shared::auth::protect_route};
use crate::{calendar::repos::ICalendarRepo, shared::auth::AuthContext};
use actix_web::{web, HttpResponse};

use serde::Deserialize;

use std::sync::Arc;

#[derive(Deserialize)]
pub struct DeleteCalendarReq {
    calendar_id: String,
}

pub async fn delete_calendar_controller(
    http_req: web::HttpRequest,
    req: web::Path<DeleteCalendarReq>,
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
    let ctx = DeleteCalendarUseCaseCtx {
        calendar_repo: Arc::clone(&ctx.repos.calendar_repo),
        event_repo: Arc::clone(&ctx.repos.event_repo),
    };
    let req = DeleteCalendarUseCaseReq {
        user_id: user.id,
        calendar_id: req.calendar_id.clone(),
    };
    let res = delete_calendar_usecase(req, ctx).await;
    match res {
        Ok(_) => HttpResponse::Ok().body("Calendar deleted"),
        Err(e) => match e {
            DeleteCalendarErrors::NotFoundError => HttpResponse::NotFound().finish(),
            DeleteCalendarErrors::UnableToDelete => HttpResponse::InternalServerError().finish(),
        },
    }
}

pub enum DeleteCalendarErrors {
    NotFoundError,
    UnableToDelete,
}

pub struct DeleteCalendarUseCaseCtx {
    pub calendar_repo: Arc<dyn ICalendarRepo>,
    pub event_repo: Arc<dyn IEventRepo>,
}

pub struct DeleteCalendarUseCaseReq {
    calendar_id: String,
    user_id: String,
}

async fn delete_calendar_usecase(
    req: DeleteCalendarUseCaseReq,
    ctx: DeleteCalendarUseCaseCtx,
) -> Result<(), DeleteCalendarErrors> {
    let calendar = ctx.calendar_repo.find(&req.calendar_id).await;
    match calendar {
        Some(calendar) if calendar.user_id == req.user_id => {
            ctx.calendar_repo.delete(&calendar.id).await;
            let repo_res = ctx.event_repo.delete_by_calendar(&calendar.id).await;
            if repo_res.is_err() {
                return Err(DeleteCalendarErrors::UnableToDelete);
            }

            Ok(())
        }
        _ => Err(DeleteCalendarErrors::NotFoundError {}),
    }
}
