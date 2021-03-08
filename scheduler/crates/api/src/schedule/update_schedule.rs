use crate::shared::{
    auth::{account_can_modify_schedule, protect_account_route},
    usecase::{execute, UseCase},
};
use crate::{
    error::NettuError,
    shared::{
        auth::{protect_route, Permission},
        usecase::{execute_with_policy, PermissionBoundary, UseCaseErrorContainer},
    },
};
use actix_web::{web, HttpResponse};
use chrono_tz::Tz;
use nettu_scheduler_api_structs::update_schedule::*;
use nettu_scheduler_domain::{Schedule, ScheduleRule, ID};
use nettu_scheduler_infra::NettuContext;

fn handle_error(e: UseCaseErrors) -> NettuError {
    match e {
        UseCaseErrors::ScheduleNotFound(schedule_id) => NettuError::NotFound(format!(
            "The schedule with id: {}, was not found.",
            schedule_id
        )),
        UseCaseErrors::StorageError => NettuError::InternalError,
        UseCaseErrors::InvalidSettings(err) => NettuError::BadClientData(format!(
            "Bad schedule settings provided. Error message: {}",
            err
        )),
    }
}

pub async fn update_schedule_admin_controller(
    http_req: web::HttpRequest,
    path: web::Path<PathParams>,
    body: web::Json<RequestBody>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;
    let schedule = account_can_modify_schedule(&account, &path.schedule_id, &ctx).await?;

    let body = body.0;
    let usecase = UpdateScheduleUseCase {
        user_id: schedule.user_id,
        schedule_id: schedule.id,
        timezone: body.timezone,
        rules: body.rules,
    };

    execute(usecase, &ctx)
        .await
        .map(|res| HttpResponse::Ok().json(APIResponse::new(res.schedule)))
        .map_err(handle_error)
}

pub async fn update_schedule_controller(
    http_req: web::HttpRequest,
    ctx: web::Data<NettuContext>,
    path: web::Path<PathParams>,
    body: web::Json<RequestBody>,
) -> Result<HttpResponse, NettuError> {
    let (user, policy) = protect_route(&http_req, &ctx).await?;

    let body = body.0;
    let usecase = UpdateScheduleUseCase {
        user_id: user.id,
        schedule_id: path.0.schedule_id,
        timezone: body.timezone,
        rules: body.rules,
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|res| HttpResponse::Ok().json(APIResponse::new(res.schedule)))
        .map_err(|e| match e {
            UseCaseErrorContainer::Unauthorized(e) => NettuError::Unauthorized(e),
            UseCaseErrorContainer::UseCase(e) => handle_error(e),
        })
}

#[derive(Debug)]
struct UpdateScheduleUseCase {
    pub user_id: ID,
    pub schedule_id: ID,
    pub timezone: Option<String>,
    pub rules: Option<Vec<ScheduleRule>>,
}

#[derive(Debug)]
enum UseCaseErrors {
    ScheduleNotFound(ID),
    StorageError,
    InvalidSettings(String),
}
struct UseCaseRes {
    pub schedule: Schedule,
}

#[async_trait::async_trait(?Send)]
impl UseCase for UpdateScheduleUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors> {
        let mut schedule = match ctx.repos.schedule_repo.find(&self.schedule_id).await {
            Some(cal) if cal.user_id == self.user_id => cal,
            _ => return Err(UseCaseErrors::ScheduleNotFound(self.schedule_id.clone())),
        };

        if let Some(tz) = &self.timezone {
            match tz.parse::<Tz>() {
                Ok(tz) => schedule.timezone = tz,
                Err(_) => {
                    return Err(UseCaseErrors::InvalidSettings(format!(
                        "Invalid timezone provided: {}",
                        tz
                    )))
                }
            }
        };
        if let Some(rules) = &self.rules {
            schedule.set_rules(rules);
        }

        let repo_res = ctx.repos.schedule_repo.save(&schedule).await;
        match repo_res {
            Ok(_) => Ok(UseCaseRes { schedule }),
            Err(_) => Err(UseCaseErrors::StorageError),
        }
    }
}

impl PermissionBoundary for UpdateScheduleUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::UpdateSchedule]
    }
}
