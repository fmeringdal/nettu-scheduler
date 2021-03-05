use crate::shared::usecase::{execute, UseCase, UseCaseErrorContainer};
use crate::{
    error::NettuError,
    shared::{
        auth::{protect_account_route, protect_route, Permission},
        usecase::{execute_with_policy, PermissionBoundary},
    },
};
use actix_web::{web, HttpResponse};
use chrono_tz::Tz;
use nettu_scheduler_api_structs::create_schedule::*;
use nettu_scheduler_domain::{Schedule, ID};
use nettu_scheduler_infra::NettuContext;

pub async fn create_schedule_admin_controller(
    http_req: web::HttpRequest,
    path_params: web::Path<AdminPathParams>,
    body_params: web::Json<RequstBody>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let usecase = CreateScheduleUseCase {
        user_id: path_params.user_id.clone(),
        account_id: account.id,
        tzid: body_params.timezone.to_owned(),
    };

    execute(usecase, &ctx)
        .await
        .map(|res| HttpResponse::Created().json(APIResponse::new(res.schedule)))
        .map_err(|e| match e {
            UseCaseErrors::InvalidTimezone(msg) => NettuError::BadClientData(format!(
                "Invalid timezone: {}. It should be a valid IANA TimeZone.",
                msg
            )),
            UseCaseErrors::Storage => NettuError::InternalError,
            UseCaseErrors::UserNotFound => NettuError::NotFound(format!(
                "The user with id: {}, was not found.",
                path_params.user_id
            )),
        })
}

pub async fn create_schedule_controller(
    http_req: web::HttpRequest,
    body_params: web::Json<RequstBody>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let (user, policy) = protect_route(&http_req, &ctx).await?;

    let usecase = CreateScheduleUseCase {
        user_id: user.id,
        account_id: user.account_id,
        tzid: body_params.timezone.to_owned(),
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|res| HttpResponse::Created().json(APIResponse::new(res.schedule)))
        .map_err(|e| {
            match e {
                UseCaseErrorContainer::Unauthorized(e) => NettuError::Unauthorized(e),
                UseCaseErrorContainer::UseCase(e) => match e {
                    UseCaseErrors::InvalidTimezone(msg) => NettuError::BadClientData(format!(
                        "Invalid timezone: {}. It should be a valid IANA TimeZone.",
                        msg
                    )),
                    UseCaseErrors::Storage => NettuError::InternalError,
                    // This should never happen
                    UseCaseErrors::UserNotFound => {
                        NettuError::NotFound("The user was not found.".into())
                    }
                },
            }
        })
}

#[derive(Debug)]
struct CreateScheduleUseCase {
    pub user_id: ID,
    pub account_id: ID,
    pub tzid: String,
}

#[derive(Debug)]
enum UseCaseErrors {
    InvalidTimezone(String),
    UserNotFound,
    Storage,
}

struct UseCaseRes {
    pub schedule: Schedule,
}

#[async_trait::async_trait(?Send)]
impl UseCase for CreateScheduleUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    type Context = NettuContext;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let tz: Tz = match self.tzid.parse() {
            Ok(tz) => tz,
            Err(_) => return Err(UseCaseErrors::InvalidTimezone(self.tzid.to_string())),
        };

        let user = ctx
            .repos
            .user_repo
            .find_by_account_id(&self.user_id, &self.account_id)
            .await;
        if user.is_none() {
            return Err(UseCaseErrors::UserNotFound);
        }

        let schedule = Schedule::new(self.user_id.clone(), &tz);

        let res = ctx.repos.schedule_repo.insert(&schedule).await;
        match res {
            Ok(_) => Ok(UseCaseRes { schedule }),
            Err(_) => Err(UseCaseErrors::Storage),
        }
    }
}

impl PermissionBoundary for CreateScheduleUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::CreateSchedule]
    }
}
