use std::print;

use crate::{
    api::{Context, NettuError},
    shared::{
        auth::{protect_account_route, protect_route},
        usecase::{execute_with_policy, PermissionBoundary, UseCaseErrorContainer},
    },
};
use crate::{
    calendar::domain::Calendar,
    shared::usecase::{execute, UseCase},
};
use crate::{shared::auth::Permission, user::domain::User};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AdminControllerPathParams {
    user_id: String,
}

pub async fn create_calendar_admin_controller(
    http_req: web::HttpRequest,
    path_params: web::Path<AdminControllerPathParams>,
    ctx: web::Data<Context>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let user_id = User::create_id(&account.id, &path_params.user_id);
    let usecase = CreateCalendarUseCase { user_id };

    execute(usecase, &ctx)
        .await
        .map(|json| HttpResponse::Created().json(json))
        .map_err(|e| match e {
            UseCaseErrors::StorageError => NettuError::InternalError,
            UseCaseErrors::UserNotFoundError => NettuError::NotFound(format!(
                "The user with id: {}, was not found.",
                path_params.user_id
            )),
        })
}

pub async fn create_calendar_controller(
    http_req: web::HttpRequest,
    ctx: web::Data<Context>,
) -> Result<HttpResponse, NettuError> {
    let (user, policy) = protect_route(&http_req, &ctx).await?;
    println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!###########!!!!");
    println!("{:?}", policy);

    let usecase = CreateCalendarUseCase { user_id: user.id };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|usecase_res| HttpResponse::Created().json(usecase_res))
        .map_err(|e| {
            match e {
                UseCaseErrorContainer::Unauthorized => {
                    NettuError::Unauthorized("Client is not permitted to create calendar".into())
                }
                UseCaseErrorContainer::UseCase(e) => match e {
                    UseCaseErrors::StorageError => NettuError::InternalError,
                    // This should never happen
                    UseCaseErrors::UserNotFoundError => {
                        NettuError::NotFound("The user was not found.".into())
                    }
                },
            }
        })
}

struct CreateCalendarUseCase {
    pub user_id: String,
}

#[derive(Debug)]
enum UseCaseErrors {
    UserNotFoundError,
    StorageError,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UseCaseRes {
    pub calendar_id: String,
}

#[async_trait::async_trait(?Send)]
impl UseCase for CreateCalendarUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let user = ctx.repos.user_repo.find(&self.user_id).await;
        if user.is_none() {
            return Err(UseCaseErrors::UserNotFoundError);
        }

        let calendar = Calendar::new(&self.user_id);

        let res = ctx.repos.calendar_repo.insert(&calendar).await;
        match res {
            Ok(_) => Ok(UseCaseRes {
                calendar_id: calendar.id.clone(),
            }),
            Err(_) => Err(UseCaseErrors::StorageError),
        }
    }
}

impl PermissionBoundary for CreateCalendarUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::CreateCalendar]
    }
}
