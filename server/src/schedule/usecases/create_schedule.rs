use crate::{
    api::{Context, NettuError},
    shared::auth::{protect_account_route, protect_route},
};
use crate::{
    schedule::domain::Schedule,
    shared::usecase::{execute, Usecase},
};
use crate::{schedule::dtos::ScheduleDTO, user::domain::User};
use actix_web::{web, HttpResponse};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AdminControllerPathParams {
    user_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BodyParams {
    timezone: String,
}

pub async fn create_schedule_admin_controller(
    http_req: web::HttpRequest,
    path_params: web::Path<AdminControllerPathParams>,
    body_params: web::Json<BodyParams>,
    ctx: web::Data<Context>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let user_id = User::create_id(&account.id, &path_params.user_id);
    let usecase = CreateScheduleUseCase {
        user_id,
        tzid: body_params.timezone.to_owned(),
    };

    execute(usecase, &ctx)
        .await
        .map(|res| {
            let dto = ScheduleDTO::new(&res.schedule);
            HttpResponse::Created().json(dto)
        })
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
    body_params: web::Json<BodyParams>,
    ctx: web::Data<Context>,
) -> Result<HttpResponse, NettuError> {
    let user = protect_route(&http_req, &ctx).await?;

    let usecase = CreateScheduleUseCase {
        user_id: user.id,
        tzid: body_params.timezone.to_owned(),
    };

    execute(usecase, &ctx)
        .await
        .map(|res| {
            let dto = ScheduleDTO::new(&res.schedule);
            HttpResponse::Created().json(dto)
        })
        .map_err(|e| {
            match e {
                UseCaseErrors::InvalidTimezone(msg) => NettuError::BadClientData(format!(
                    "Invalid timezone: {}. It should be a valid IANA TimeZone.",
                    msg
                )),
                UseCaseErrors::Storage => NettuError::InternalError,
                // This should never happen
                UseCaseErrors::UserNotFound => {
                    NettuError::NotFound("The user was not found.".into())
                }
            }
        })
}

struct CreateScheduleUseCase {
    pub user_id: String,
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
impl Usecase for CreateScheduleUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let tz: Tz = match self.tzid.parse() {
            Ok(tz) => tz,
            Err(_) => return Err(UseCaseErrors::InvalidTimezone(self.tzid.to_string())),
        };

        let user = ctx.repos.user_repo.find(&self.user_id).await;
        if user.is_none() {
            return Err(UseCaseErrors::UserNotFound);
        }

        let schedule = Schedule::new(&self.user_id, &tz);

        let res = ctx.repos.schedule_repo.insert(&schedule).await;
        match res {
            Ok(_) => Ok(UseCaseRes { schedule }),
            Err(_) => Err(UseCaseErrors::Storage),
        }
    }
}
