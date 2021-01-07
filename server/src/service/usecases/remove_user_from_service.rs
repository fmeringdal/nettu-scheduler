use crate::api::Context;
use crate::{
    account::domain::Account,
    api::NettuError,
    service::domain::ServiceResource,
    shared::{
        auth::protect_account_route,
        usecase::{execute, Usecase},
    },
    user::domain::User,
};
use actix_web::{web, HttpRequest, HttpResponse};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct PathParams {
    service_id: String,
    user_id: String,
}

pub async fn remove_user_from_service_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    ctx: web::Data<Context>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let user_id = User::create_id(&account.id, &path_params.user_id);
    let usecase = RemoveUserFromServiceUsecase {
        account,
        service_id: path_params.service_id.to_owned(),
        user_id,
    };

    execute(usecase, &ctx)
        .await
        .map(|_| HttpResponse::Ok().body("Service successfully updated"))
        .map_err(|e| match e {
            UseCaseErrors::StorageError => NettuError::InternalError,
            UseCaseErrors::ServiceNotFoundError => {
                NettuError::NotFound(format!("The requested service was not found"))
            }
            UseCaseErrors::UserNotFoundError => {
                NettuError::NotFound(format!("The specified user was not found in the service"))
            }
        })
}

struct RemoveUserFromServiceUsecase {
    pub account: Account,
    pub service_id: String,
    pub user_id: String,
}

struct UseCaseRes {
    pub resource: ServiceResource,
}

#[derive(Debug)]
enum UseCaseErrors {
    StorageError,
    ServiceNotFoundError,
    UserNotFoundError,
}

#[async_trait::async_trait(?Send)]
impl Usecase for RemoveUserFromServiceUsecase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let mut service = match ctx.repos.service_repo.find(&self.service_id).await {
            Some(service) if service.account_id == self.account.id => service,
            _ => return Err(UseCaseErrors::ServiceNotFoundError),
        };

        match service.remove_user(&self.user_id) {
            Some(resource) => match ctx.repos.service_repo.save(&service).await {
                Ok(_) => Ok(UseCaseRes { resource }),
                Err(_) => Err(UseCaseErrors::StorageError),
            },
            None => Err(UseCaseErrors::UserNotFoundError),
        }
    }
}
