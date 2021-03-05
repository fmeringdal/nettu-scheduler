use crate::shared::{
    auth::{protect_route, Permission},
    usecase::{execute_with_policy, PermissionBoundary, UseCaseErrorContainer},
};
use crate::{error::NettuError, shared::usecase::UseCase};
use actix_web::{web, HttpResponse};
use nettu_scheduler_api_structs::delete_calendar::{APIResponse, PathParams};
use nettu_scheduler_domain::{Calendar, ID};
use nettu_scheduler_infra::NettuContext;

pub async fn delete_calendar_controller(
    http_req: web::HttpRequest,
    req: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let (user, policy) = protect_route(&http_req, &ctx).await?;

    let usecase = DeleteCalendarUseCase {
        user_id: user.id,
        calendar_id: req.calendar_id.clone(),
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|calendar| HttpResponse::Ok().json(APIResponse::new(calendar)))
        .map_err(|e| match e {
            UseCaseErrorContainer::Unauthorized(e) => NettuError::Unauthorized(e),
            UseCaseErrorContainer::UseCase(e) => match e {
                UseCaseErrors::NotFound => NettuError::NotFound(format!(
                    "The calendar with id: {}, was not found.",
                    req.calendar_id
                )),
                UseCaseErrors::UnableToDelete => NettuError::InternalError,
            },
        })
}

#[derive(Debug)]
pub enum UseCaseErrors {
    NotFound,
    UnableToDelete,
}

#[derive(Debug)]
pub struct DeleteCalendarUseCase {
    calendar_id: ID,
    user_id: ID,
}

#[async_trait::async_trait(?Send)]
impl UseCase for DeleteCalendarUseCase {
    type Response = Calendar;

    type Errors = UseCaseErrors;

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors> {
        let calendar = ctx.repos.calendar_repo.find(&self.calendar_id).await;
        match calendar {
            Some(calendar) if calendar.user_id == self.user_id => {
                ctx.repos.calendar_repo.delete(&calendar.id).await;
                let repo_res = ctx.repos.event_repo.delete_by_calendar(&calendar.id).await;
                if repo_res.is_err() {
                    return Err(UseCaseErrors::UnableToDelete);
                }
                let repo_res = ctx
                    .repos
                    .service_repo
                    .remove_calendar_from_services(&calendar.id)
                    .await;
                if repo_res.is_err() {
                    return Err(UseCaseErrors::UnableToDelete);
                }

                Ok(calendar)
            }
            _ => Err(UseCaseErrors::NotFound),
        }
    }
}

impl PermissionBoundary for DeleteCalendarUseCase {
    fn permissions(&self) -> Vec<crate::shared::auth::Permission> {
        vec![Permission::DeleteCalendar]
    }
}
