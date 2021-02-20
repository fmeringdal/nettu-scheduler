use crate::shared::usecase::{execute, UseCase};
use crate::user::dtos::UserDTO;
use crate::{error::NettuError, shared::auth::protect_account_route};
use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_core::domain::User;
use nettu_scheduler_infra::Context;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BodyParams {
    pub user_id: String,
}

pub async fn create_user_controller(
    http_req: HttpRequest,
    body: web::Json<BodyParams>,
    ctx: web::Data<Context>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let usecase = CreateUserUseCase {
        account_id: account.id.clone(),
        external_user_id: body.user_id.clone(),
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| {
            let res = UserDTO::new(&usecase_res.user);
            HttpResponse::Created().json(res)
        })
        .map_err(|e| match e {
            UseCaseErrors::StorageError => NettuError::InternalError,
            UseCaseErrors::UserAlreadyExists => NettuError::Conflict(
                "A user with that userId already exist. UserIds need to be unique.".into(),
            ),
        })
}

pub struct CreateUserUseCase {
    pub account_id: String,
    pub external_user_id: String,
}
pub struct UseCaseRes {
    pub user: User,
}

#[derive(Debug)]
pub enum UseCaseErrors {
    StorageError,
    UserAlreadyExists,
}

#[async_trait::async_trait(?Send)]
impl UseCase for CreateUserUseCase {
    type Response = UseCaseRes;
    type Errors = UseCaseErrors;
    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let user = User::new(&self.account_id, &self.external_user_id);

        if let Some(_existing_user) = ctx.repos.user_repo.find(&user.id).await {
            return Err(UseCaseErrors::UserAlreadyExists);
        }

        let res = ctx.repos.user_repo.insert(&user).await;
        match res {
            Ok(_) => Ok(UseCaseRes { user }),
            Err(_) => Err(UseCaseErrors::StorageError),
        }
    }
}
