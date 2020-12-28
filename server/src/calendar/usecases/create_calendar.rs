use crate::api::Context;
use crate::calendar::domain::calendar::Calendar;
use crate::calendar::repos::ICalendarRepo;
use actix_web::{web, HttpResponse};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct CreateCalendarReq {
    pub user_id: String,
}

pub async fn create_calendar_controller(
    req: web::Json<CreateCalendarReq>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let res = create_calendar_usecase(
        req.0,
        CreateCalendarUseCaseCtx {
            calendar_repo: Arc::clone(&ctx.repos.calendar_repo),
        },
    )
    .await;

    match res {
        Ok(json) => HttpResponse::Ok().json(json),
        Err(_) => HttpResponse::UnprocessableEntity().finish(),
    }
}

#[derive(Serialize)]
pub struct CreateCalendarRes {
    pub calendar_id: String,
}

pub struct CreateCalendarUseCaseCtx {
    pub calendar_repo: Arc<dyn ICalendarRepo>,
}

pub async fn create_calendar_usecase(
    req: CreateCalendarReq,
    ctx: CreateCalendarUseCaseCtx,
) -> Result<CreateCalendarRes, anyhow::Error> {
    let calendar = Calendar {
        id: ObjectId::new().to_string(),
        user_id: req.user_id,
    };
    let res = ctx.calendar_repo.insert(&calendar).await;
    match res {
        Ok(_) => Ok(CreateCalendarRes {
            calendar_id: calendar.id.clone(),
        }),
        Err(_) => Err(anyhow::Error::msg("Unable to create calendar")),
    }
}
