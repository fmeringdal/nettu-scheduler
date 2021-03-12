use crate::shared::usecase::{execute, UseCase};
use crate::{error::NettuError, shared::auth::protect_account_route};
use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_api_structs::update_user::*;
use nettu_scheduler_domain::{Metadata, User, ID};
use nettu_scheduler_infra::NettuContext;

pub async fn update_user_controller(
    http_req: HttpRequest,
    body: web::Json<RequestBody>,
    path: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let usecase = UpdateUserUseCase {
        account_id: account.id,
        user_id: path.0.user_id,
        metadata: body.0.metadata,
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| HttpResponse::Ok().json(APIResponse::new(usecase_res.user)))
        .map_err(|e| match e {
            UseCaseErrors::StorageError => NettuError::InternalError,
            UseCaseErrors::UserNotFound(id) => {
                NettuError::Conflict(format!("A user with id {} was not found", id))
            }
        })
}

#[derive(Debug)]
pub struct UpdateUserUseCase {
    pub account_id: ID,
    pub user_id: ID,
    pub metadata: Option<Metadata>,
}

#[derive(Debug)]
pub struct UseCaseRes {
    pub user: User,
}

#[derive(Debug)]
pub enum UseCaseErrors {
    StorageError,
    UserNotFound(ID),
}

#[async_trait::async_trait(?Send)]
impl UseCase for UpdateUserUseCase {
    type Response = UseCaseRes;
    type Errors = UseCaseErrors;

    const NAME: &'static str = "UpdateUser";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors> {
        let mut user = match ctx
            .repos
            .user_repo
            .find_by_account_id(&self.user_id, &self.account_id)
            .await
        {
            Some(user) => user,
            None => return Err(UseCaseErrors::UserNotFound(self.user_id.clone())),
        };

        if let Some(metadata) = &self.metadata {
            user.metadata = metadata.clone();
        }

        ctx.repos
            .user_repo
            .save(&user)
            .await
            .map(|_| UseCaseRes { user })
            .map_err(|_| UseCaseErrors::StorageError)
    }
}
