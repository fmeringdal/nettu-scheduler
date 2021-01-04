use crate::calendar::domain::calendar::Calendar;
use crate::{
    api::Context,
    shared::auth::{protect_account_route, protect_route},
    user::repos::IUserRepo,
};
use crate::{calendar::repos::ICalendarRepo, user::domain::User};
use actix_web::{web, HttpResponse};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize)]
pub struct AdminControllerPathParams {
    user_id: String,
}

pub async fn create_calendar_admin_controller(
    http_req: web::HttpRequest,
    path_params: web::Json<AdminControllerPathParams>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let account = match protect_account_route(&http_req, &ctx).await {
        Ok(u) => u,
        Err(res) => return res,
    };

    let user_id = User::create_id(&account.id, &path_params.user_id);
    let res = create_calendar_usecase(
        UsecaseReq { user_id },
        UsecaseCtx {
            calendar_repo: Arc::clone(&ctx.repos.calendar_repo),
            user_repo: Arc::clone(&ctx.repos.user_repo),
        },
    )
    .await;

    match res {
        Ok(json) => HttpResponse::Created().json(json),
        Err(_) => HttpResponse::UnprocessableEntity().finish(),
    }
}

pub async fn create_calendar_controller(
    http_req: web::HttpRequest,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let user = match protect_route(&http_req, &ctx).await {
        Ok(u) => u,
        Err(res) => return res,
    };
    let res = create_calendar_usecase(
        UsecaseReq { user_id: user.id },
        UsecaseCtx {
            calendar_repo: Arc::clone(&ctx.repos.calendar_repo),
            user_repo: Arc::clone(&ctx.repos.user_repo),
        },
    )
    .await;

    match res {
        Ok(json) => HttpResponse::Created().json(json),
        Err(_) => HttpResponse::UnprocessableEntity().finish(),
    }
}

struct UsecaseReq {
    pub user_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UsecaseRes {
    pub calendar_id: String,
}

enum UsecaseErrors {
    UserNotFoundError,
    StorageError,
}

struct UsecaseCtx {
    pub calendar_repo: Arc<dyn ICalendarRepo>,
    pub user_repo: Arc<dyn IUserRepo>,
}

async fn create_calendar_usecase(
    req: UsecaseReq,
    ctx: UsecaseCtx,
) -> Result<UsecaseRes, UsecaseErrors> {
    let user = ctx.user_repo.find(&req.user_id).await;
    if user.is_none() {
        return Err(UsecaseErrors::UserNotFoundError);
    }

    let calendar = Calendar {
        id: ObjectId::new().to_string(),
        user_id: req.user_id,
    };
    let res = ctx.calendar_repo.insert(&calendar).await;
    match res {
        Ok(_) => Ok(UsecaseRes {
            calendar_id: calendar.id.clone(),
        }),
        Err(_) => Err(UsecaseErrors::StorageError),
    }
}
