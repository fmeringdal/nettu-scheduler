use crate::{api::Context, event::repos::IEventRepo, shared::auth::protect_route, user::domain::User};
use crate::{event::domain::event::CalendarEvent, shared::auth::AuthContext};
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize)]
pub struct CreateEventExceptionPathParams {
    event_id: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEventExceptionBody {
    exception_ts: i64,
}

pub async fn create_event_exception_controller(
    http_req: HttpRequest,
    path_params: web::Path<CreateEventExceptionPathParams>,
    body: web::Json<CreateEventExceptionBody>,
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

    let ctx = CreateEventExceptionUseCaseCtx {
        event_repo: ctx.repos.event_repo.clone(),
    };
    let req = CreateEventExceptionReq {
        event_id: path_params.event_id.clone(),
        exception_ts: body.exception_ts.clone(),
    };

    let res = create_event_exception_usecase(req, user, ctx).await;
    match res {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => match e {
            CreateCalendarEventExceptionErrors::NotFoundError => HttpResponse::NotFound().finish(),
            CreateCalendarEventExceptionErrors::StorageError => {
                HttpResponse::InternalServerError().finish()
            }
        },
    }
}

pub struct CreateEventExceptionReq {
    event_id: String,
    exception_ts: i64,
}

pub struct CreateEventExceptionUseCaseCtx {
    pub event_repo: Arc<dyn IEventRepo>,
}

#[derive(Debug)]
pub enum CreateCalendarEventExceptionErrors {
    NotFoundError,
    StorageError,
}

async fn create_event_exception_usecase(
    req: CreateEventExceptionReq,
    user: User,
    ctx: CreateEventExceptionUseCaseCtx,
) -> Result<CalendarEvent, CreateCalendarEventExceptionErrors> {
    let mut event = match ctx.event_repo.find(&req.event_id).await {
        Some(event) if event.user_id == user.id => event,
        _ => return Err(CreateCalendarEventExceptionErrors::NotFoundError),
    };

    event.exdates.push(req.exception_ts);

    let repo_res = ctx.event_repo.save(&event).await;
    if repo_res.is_err() {
        return Err(CreateCalendarEventExceptionErrors::StorageError);
    }

    Ok(event)
}
