use crate::{calendar::repos::ICalendarRepo, user::domain::User};
use crate::{api::Context, shared::auth::protect_route};
use crate::{calendar::domain::calendar::Calendar, shared::auth::AuthContext};
use actix_web::{web, HttpResponse};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub async fn create_calendar_controller(
    http_req: web::HttpRequest,
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
    let res = create_calendar_usecase(
        CreateCalendarReq { user },
        CreateCalendarUseCaseCtx {
            calendar_repo: Arc::clone(&ctx.repos.calendar_repo),
        },
    )
    .await;

    match res {
        Ok(json) => HttpResponse::Created().json(json),
        Err(_) => HttpResponse::UnprocessableEntity().finish(),
    }
}

struct CreateCalendarReq {
    pub user: User,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateCalendarRes {
    pub calendar_id: String,
}

struct CreateCalendarUseCaseCtx {
    pub calendar_repo: Arc<dyn ICalendarRepo>,
}

async fn create_calendar_usecase(
    req: CreateCalendarReq,
    ctx: CreateCalendarUseCaseCtx,
) -> Result<CreateCalendarRes, anyhow::Error> {
    let calendar = Calendar {
        id: ObjectId::new().to_string(),
        user_id: req.user.id,
    };
    let res = ctx.calendar_repo.insert(&calendar).await;
    match res {
        Ok(_) => Ok(CreateCalendarRes {
            calendar_id: calendar.id.clone(),
        }),
        Err(_) => Err(anyhow::Error::msg("Unable to create calendar")),
    }
}
