use crate::shared::{
    auth::{protect_route, Permission},
    usecase::{execute_with_policy, PermissionBoundary, UseCaseErrorContainer},
};
use crate::{error::NettuError, shared::usecase::UseCase};
use actix_web::{web, HttpResponse};
use nettu_scheduler_api_structs::delete_schedule::*;
use nettu_scheduler_domain::{Schedule, ID};
use nettu_scheduler_infra::NettuContext;

pub async fn delete_schedule_controller(
    http_req: web::HttpRequest,
    req: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let (user, policy) = protect_route(&http_req, &ctx).await?;

    let usecase = DeleteScheduleUseCase {
        user_id: user.id,
        schedule_id: req.schedule_id.clone(),
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|schedule| HttpResponse::Ok().json(APIResponse::new(schedule)))
        .map_err(|e| match e {
            UseCaseErrorContainer::Unauthorized(e) => NettuError::Unauthorized(e),
            UseCaseErrorContainer::UseCase(e) => match e {
                UseCaseErrors::NotFound => NettuError::NotFound(format!(
                    "The schedule with id: {}, was not found.",
                    req.schedule_id
                )),
                UseCaseErrors::StorageError => NettuError::InternalError,
            },
        })
}

#[derive(Debug)]
pub enum UseCaseErrors {
    NotFound,
    StorageError,
}

#[derive(Debug)]
pub struct DeleteScheduleUseCase {
    schedule_id: ID,
    user_id: ID,
}

#[async_trait::async_trait(?Send)]
impl UseCase for DeleteScheduleUseCase {
    type Response = Schedule;

    type Errors = UseCaseErrors;

    type Context = NettuContext;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let schedule = ctx.repos.schedule_repo.find(&self.schedule_id).await;
        match schedule {
            Some(schedule) if schedule.user_id == self.user_id => {
                let res = ctx.repos.schedule_repo.delete(&schedule.id).await;
                if res.is_none() {
                    return Err(UseCaseErrors::StorageError);
                }
                let res = ctx
                    .repos
                    .service_repo
                    .remove_schedule_from_services(&schedule.id)
                    .await;
                if res.is_err() {
                    return Err(UseCaseErrors::StorageError);
                }

                Ok(schedule)
            }
            _ => Err(UseCaseErrors::NotFound),
        }
    }
}

impl PermissionBoundary for DeleteScheduleUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::DeleteSchedule]
    }
}
