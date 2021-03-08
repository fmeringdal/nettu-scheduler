use crate::shared::{
    auth::account_can_modify_user,
    usecase::{execute, UseCase, UseCaseErrorContainer},
};
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
use nettu_scheduler_domain::{Schedule, ScheduleRule, ID};
use nettu_scheduler_infra::NettuContext;

fn handle_error(e: UseCaseErrors) -> NettuError {
    match e {
        UseCaseErrors::InvalidTimezone(msg) => NettuError::BadClientData(format!(
            "Invalid timezone: {}. It should be a valid IANA TimeZone.",
            msg
        )),
        UseCaseErrors::Storage => NettuError::InternalError,
        UseCaseErrors::UserNotFound(user_id) => {
            NettuError::NotFound(format!("The user with id: {}, was not found.", user_id))
        }
    }
}

pub async fn create_schedule_admin_controller(
    http_req: web::HttpRequest,
    path_params: web::Path<PathParams>,
    body_params: web::Json<RequestBody>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;
    let user = account_can_modify_user(&account, &path_params.user_id, &ctx).await?;

    let usecase = CreateScheduleUseCase {
        user_id: user.id,
        account_id: account.id,
        tzid: body_params.0.timezone,
        rules: body_params.0.rules,
    };

    execute(usecase, &ctx)
        .await
        .map(|res| HttpResponse::Created().json(APIResponse::new(res.schedule)))
        .map_err(handle_error)
}

pub async fn create_schedule_controller(
    http_req: web::HttpRequest,
    body_params: web::Json<RequestBody>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let (user, policy) = protect_route(&http_req, &ctx).await?;

    let usecase = CreateScheduleUseCase {
        user_id: user.id,
        account_id: user.account_id,
        tzid: body_params.0.timezone,
        rules: body_params.0.rules,
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|res| HttpResponse::Created().json(APIResponse::new(res.schedule)))
        .map_err(|e| match e {
            UseCaseErrorContainer::Unauthorized(e) => NettuError::Unauthorized(e),
            UseCaseErrorContainer::UseCase(e) => handle_error(e),
        })
}

#[derive(Debug)]
struct CreateScheduleUseCase {
    pub user_id: ID,
    pub account_id: ID,
    pub tzid: String,
    pub rules: Option<Vec<ScheduleRule>>,
}

#[derive(Debug)]
enum UseCaseErrors {
    InvalidTimezone(String),
    UserNotFound(ID),
    Storage,
}

struct UseCaseRes {
    pub schedule: Schedule,
}

#[async_trait::async_trait(?Send)]
impl UseCase for CreateScheduleUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors> {
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
            return Err(UseCaseErrors::UserNotFound(self.user_id.clone()));
        }
        let user = user.unwrap();

        let mut schedule = Schedule::new(user.id, user.account_id, &tz);
        if let Some(rules) = &self.rules {
            schedule.rules = rules.clone();
        }

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
