use crate::shared::{
    auth::{account_can_modify_calendar, protect_account_route},
    usecase::{execute, UseCase},
};
use crate::{error::NettuError, shared::auth::protect_route};
use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_api_structs::get_calendar::{APIResponse, PathParams};
use nettu_scheduler_domain::{Calendar, ID};
use nettu_scheduler_infra::NettuContext;

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
        .map_err(NettuError::from)
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
        .map_err(NettuError::from)
}

#[derive(Debug)]
struct GetCalendarUseCase {
    pub user_id: ID,
    pub calendar_id: ID,
}

#[derive(Debug)]
enum UseCaseError {
    NotFound(ID),
}
impl From<UseCaseError> for NettuError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::NotFound(calendar_id) => Self::NotFound(format!(
                "The calendar with id: {}, was not found.",
                calendar_id
            )),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetCalendarUseCase {
    type Response = Calendar;

    type Error = UseCaseError;

    const NAME: &'static str = "GetCalendar";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Error> {
        let cal = ctx.repos.calendars.find(&self.calendar_id).await;
        match cal {
            Some(cal) if cal.user_id == self.user_id => Ok(cal),
            _ => Err(UseCaseError::NotFound(self.calendar_id.clone())),
        }
    }
}
