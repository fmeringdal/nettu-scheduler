use crate::{
    api::Context,
    calendar::{domain::Calendar, dtos::CalendarDTO},
    shared::usecase::{execute, UseCase},
};
use crate::{api::NettuError, shared::auth::protect_route};
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCalendarReq {
    pub calendar_id: String,
}

pub async fn get_calendar_controller(
    http_req: HttpRequest,
    req: web::Path<GetCalendarReq>,
    ctx: web::Data<Context>,
) -> Result<HttpResponse, NettuError> {
    let (user, _policy) = protect_route(&http_req, &ctx).await?;

    let usecase = GetCalendarUseCase {
        user_id: user.id.clone(),
        calendar_id: req.calendar_id.clone(),
    };

    execute(usecase, &ctx)
        .await
        .map(|calendar| {
            let dto = CalendarDTO::new(&calendar);
            HttpResponse::Ok().json(dto)
        })
        .map_err(|e| match e {
            UseCaseErrors::NotFound => NettuError::NotFound(format!(
                "The calendar with id: {}, was not found.",
                req.calendar_id
            )),
        })
}

struct GetCalendarUseCase {
    pub user_id: String,
    pub calendar_id: String,
}

#[derive(Debug)]
enum UseCaseErrors {
    NotFound,
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetCalendarUseCase {
    type Response = Calendar;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let cal = ctx.repos.calendar_repo.find(&self.calendar_id).await;
        match cal {
            Some(cal) if cal.user_id == self.user_id => Ok(cal),
            _ => Err(UseCaseErrors::NotFound),
        }
    }
}
