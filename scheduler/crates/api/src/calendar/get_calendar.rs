use crate::shared::{
    auth::{account_can_modify_calendar, protect_account_route},
    usecase::{execute, UseCase},
};
use crate::{error::NettuError, shared::auth::protect_route};
use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_api_structs::get_calendar::{APIResponse, PathParams};
use nettu_scheduler_domain::{Calendar, ID};
use nettu_scheduler_infra::NettuContext;

fn handle_errors(e: UseCaseErrors) -> NettuError {
    match e {
        UseCaseErrors::NotFound(calendar_id) => NettuError::NotFound(format!(
            "The calendar with id: {}, was not found.",
            calendar_id
        )),
    }
}

pub async fn get_calendar_admin_controller(
    http_req: web::HttpRequest,
    path: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;
    let cal = account_can_modify_calendar(&account, &path.calendar_id, &ctx).await?;

    let usecase = GetCalendarUseCase {
        user_id: cal.user_id,
        calendar_id: cal.id,
    };

    execute(usecase, &ctx)
        .await
        .map(|calendar| HttpResponse::Ok().json(APIResponse::new(calendar)))
        .map_err(handle_errors)
}

pub async fn get_calendar_controller(
    http_req: HttpRequest,
    path: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let (user, _policy) = protect_route(&http_req, &ctx).await?;

    let usecase = GetCalendarUseCase {
        user_id: user.id.clone(),
        calendar_id: path.calendar_id.clone(),
    };

    execute(usecase, &ctx)
        .await
        .map(|calendar| HttpResponse::Ok().json(APIResponse::new(calendar)))
        .map_err(handle_errors)
}

#[derive(Debug)]
struct GetCalendarUseCase {
    pub user_id: ID,
    pub calendar_id: ID,
}

#[derive(Debug)]
enum UseCaseErrors {
    NotFound(ID),
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetCalendarUseCase {
    type Response = Calendar;

    type Errors = UseCaseErrors;

    const NAME: &'static str = "GetCalendar";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors> {
        let cal = ctx.repos.calendar_repo.find(&self.calendar_id).await;
        match cal {
            Some(cal) if cal.user_id == self.user_id => Ok(cal),
            _ => Err(UseCaseErrors::NotFound(self.calendar_id.clone())),
        }
    }
}
