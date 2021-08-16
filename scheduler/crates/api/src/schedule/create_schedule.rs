use crate::shared::{
    auth::account_can_modify_user,
    usecase::{execute, UseCase},
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
use nettu_scheduler_domain::{Metadata, Schedule, ScheduleRule, ID};
use nettu_scheduler_infra::NettuContext;

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
        metadata: body_params.0.metadata,
    };

    execute(usecase, &ctx)
        .await
        .map(|res| HttpResponse::Created().json(APIResponse::new(res.schedule)))
        .map_err(NettuError::from)
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
        metadata: body_params.0.metadata,
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|res| HttpResponse::Created().json(APIResponse::new(res.schedule)))
        .map_err(NettuError::from)
}

#[derive(Debug)]
struct CreateScheduleUseCase {
    pub user_id: ID,
    pub account_id: ID,
    pub tzid: String,
    pub rules: Option<Vec<ScheduleRule>>,
    pub metadata: Option<Metadata>,
}

#[derive(Debug)]
enum UseCaseError {
    InvalidTimezone(String),
    UserNotFound(ID),
    Storage,
}

impl From<UseCaseError> for NettuError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::InvalidTimezone(msg) => Self::BadClientData(format!(
                "Invalid timezone: {}. It should be a valid IANA TimeZone.",
                msg
            )),
            UseCaseError::Storage => Self::InternalError,
            UseCaseError::UserNotFound(user_id) => {
                Self::NotFound(format!("The user with id: {}, was not found.", user_id))
            }
        }
    }
}

#[derive(Debug)]
struct UseCaseRes {
    pub schedule: Schedule,
}

#[async_trait::async_trait(?Send)]
impl UseCase for CreateScheduleUseCase {
    type Response = UseCaseRes;

    type Error = UseCaseError;

    const NAME: &'static str = "CreateSchedule";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Error> {
        let tz: Tz = match self.tzid.parse() {
            Ok(tz) => tz,
            Err(_) => return Err(UseCaseError::InvalidTimezone(self.tzid.to_string())),
        };

        let user = ctx
            .repos
            .users
            .find_by_account_id(&self.user_id, &self.account_id)
            .await;
        if user.is_none() {
            return Err(UseCaseError::UserNotFound(self.user_id.clone()));
        }
        let user = user.unwrap();

        let mut schedule = Schedule::new(user.id, user.account_id, &tz);
        if let Some(rules) = &self.rules {
            schedule.rules = rules.clone();
        }
        if let Some(metadata) = &self.metadata {
            schedule.metadata = metadata.clone();
        }

        let res = ctx.repos.schedules.insert(&schedule).await;
        match res {
            Ok(_) => Ok(UseCaseRes { schedule }),
            Err(_) => Err(UseCaseError::Storage),
        }
    }
}

impl PermissionBoundary for CreateScheduleUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::CreateSchedule]
    }
}
