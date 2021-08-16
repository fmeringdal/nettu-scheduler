use crate::shared::{
    auth::{account_can_modify_calendar, protect_account_route, protect_route, Permission},
    usecase::{execute, execute_with_policy, PermissionBoundary},
};
use crate::{error::NettuError, shared::usecase::UseCase};
use actix_web::{web, HttpResponse};
use nettu_scheduler_api_structs::delete_calendar::{APIResponse, PathParams};
use nettu_scheduler_domain::{Calendar, ID};
use nettu_scheduler_infra::NettuContext;

pub async fn delete_calendar_admin_controller(
    http_req: web::HttpRequest,
    path: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;
    let cal = account_can_modify_calendar(&account, &path.calendar_id, &ctx).await?;

    let usecase = DeleteCalendarUseCase {
        user_id: cal.user_id,
        calendar_id: cal.id,
    };

    execute(usecase, &ctx)
        .await
        .map(|calendar| HttpResponse::Ok().json(APIResponse::new(calendar)))
        .map_err(NettuError::from)
}

pub async fn delete_calendar_controller(
    http_req: web::HttpRequest,
    path: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let (user, policy) = protect_route(&http_req, &ctx).await?;

    let usecase = DeleteCalendarUseCase {
        user_id: user.id,
        calendar_id: path.calendar_id.clone(),
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|calendar| HttpResponse::Ok().json(APIResponse::new(calendar)))
        .map_err(NettuError::from)
}

#[derive(Debug)]
pub enum UseCaseError {
    NotFound(ID),
    UnableToDelete,
}

impl From<UseCaseError> for NettuError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::NotFound(calendar_id) => Self::NotFound(format!(
                "The calendar with id: {}, was not found.",
                calendar_id
            )),
            UseCaseError::UnableToDelete => Self::InternalError,
        }
    }
}

#[derive(Debug)]
pub struct DeleteCalendarUseCase {
    calendar_id: ID,
    user_id: ID,
}

#[async_trait::async_trait(?Send)]
impl UseCase for DeleteCalendarUseCase {
    type Response = Calendar;

    type Error = UseCaseError;

    const NAME: &'static str = "DeleteCalendar";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Error> {
        let calendar = ctx.repos.calendars.find(&self.calendar_id).await;
        match calendar {
            Some(calendar) if calendar.user_id == self.user_id => ctx
                .repos
                .calendars
                .delete(&calendar.id)
                .await
                .map(|_| calendar)
                .map_err(|_| UseCaseError::UnableToDelete),
            _ => Err(UseCaseError::NotFound(self.calendar_id.clone())),
        }
    }
}

impl PermissionBoundary for DeleteCalendarUseCase {
    fn permissions(&self) -> Vec<crate::shared::auth::Permission> {
        vec![Permission::DeleteCalendar]
    }
}
