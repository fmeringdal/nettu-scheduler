use crate::{
    error::NettuError,
    shared::{
        auth::protect_account_route,
        usecase::{execute, UseCase},
    },
};
use actix_web::{web, HttpRequest, HttpResponse};

use nettu_scheduler_api_structs::remove_user_from_service::*;
use nettu_scheduler_core::{Account, Service, User};
use nettu_scheduler_infra::NettuContext;

pub async fn remove_user_from_service_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let user_id = User::create_id(&account.id, &path_params.user_id);
    let usecase = RemoveUserFromServiceUseCase {
        account,
        service_id: path_params.service_id.to_owned(),
        user_id,
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| HttpResponse::Ok().json(APIResponse::new(usecase_res.service)))
        .map_err(|e| match e {
            UseCaseErrors::StorageError => NettuError::InternalError,
            UseCaseErrors::ServiceNotFoundError => {
                NettuError::NotFound("The requested service was not found".to_string())
            }
            UseCaseErrors::UserNotFoundError => {
                NettuError::NotFound("The specified user was not found in the service".to_string())
            }
        })
}

#[derive(Debug)]
struct RemoveUserFromServiceUseCase {
    pub account: Account,
    pub service_id: String,
    pub user_id: String,
}

struct UseCaseRes {
    pub service: Service,
}

#[derive(Debug)]
enum UseCaseErrors {
    StorageError,
    ServiceNotFoundError,
    UserNotFoundError,
}

#[async_trait::async_trait(?Send)]
impl UseCase for RemoveUserFromServiceUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    type Context = NettuContext;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let mut service = match ctx.repos.service_repo.find(&self.service_id).await {
            Some(service) if service.account_id == self.account.id => service,
            _ => return Err(UseCaseErrors::ServiceNotFoundError),
        };

        match service.remove_user(&self.user_id) {
            Some(_) => match ctx.repos.service_repo.save(&service).await {
                Ok(_) => Ok(UseCaseRes { service }),
                Err(_) => Err(UseCaseErrors::StorageError),
            },
            None => Err(UseCaseErrors::UserNotFoundError),
        }
    }
}
