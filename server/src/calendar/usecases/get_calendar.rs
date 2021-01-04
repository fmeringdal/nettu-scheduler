use crate::shared::auth::protect_route;
use crate::{
    api::Context,
    calendar::domain::calendar::Calendar,
    shared::usecase::{perform, Usecase},
};
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
) -> HttpResponse {
    let user = match protect_route(&http_req, &ctx).await {
        Ok(u) => u,
        Err(res) => return res,
    };

    let usecase = GetCalendarUseCase {
        user_id: user.id.clone(),
        calendar_id: req.calendar_id.clone(),
    };

    let res = perform(usecase, &ctx).await;
    match res {
        Ok(cal) => HttpResponse::Ok().json(cal),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

struct GetCalendarUseCase {
    pub user_id: String,
    pub calendar_id: String,
}

#[derive(Debug)]
enum UseCaseErrors {
    NotFoundError,
}

#[async_trait::async_trait(?Send)]
impl Usecase for GetCalendarUseCase {
    type Response = Calendar;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn perform(&self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let cal = ctx.repos.calendar_repo.find(&self.calendar_id).await;
        match cal {
            Some(cal) if cal.user_id == self.user_id => Ok(cal),
            _ => Err(UseCaseErrors::NotFoundError),
        }
    }
}
