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
        .map_err(NettuError::from)
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
pub enum UseCaseError {
    StorageError,
    UserNotFound(ID),
}

impl From<UseCaseError> for NettuError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::StorageError => Self::InternalError,
            UseCaseError::UserNotFound(id) => {
                Self::Conflict(format!("A user with id {} was not found", id))
            }
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for UpdateUserUseCase {
    type Response = UseCaseRes;
    type Error = UseCaseError;

    const NAME: &'static str = "UpdateUser";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Error> {
        let mut user = match ctx
            .repos
            .users
            .find_by_account_id(&self.user_id, &self.account_id)
            .await
        {
            Some(user) => user,
            None => return Err(UseCaseError::UserNotFound(self.user_id.clone())),
        };

        if let Some(metadata) = &self.metadata {
            user.metadata = metadata.clone();
        }

        ctx.repos
            .users
            .save(&user)
            .await
            .map(|_| UseCaseRes { user })
            .map_err(|_| UseCaseError::StorageError)
    }
}
