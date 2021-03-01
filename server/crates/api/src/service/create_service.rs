use crate::shared::usecase::{execute, UseCase};
use crate::{error::NettuError, shared::auth::protect_account_route};
use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_api_structs::create_service::*;
use nettu_scheduler_domain::{Account, Service};
use nettu_scheduler_infra::NettuContext;

pub async fn create_service_controller(
    http_req: HttpRequest,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let usecase = CreateServiceUseCase { account };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| HttpResponse::Created().json(APIResponse::new(usecase_res.service)))
        .map_err(|e| match e {
            UseCaseErrors::StorageError => NettuError::InternalError,
        })
}

#[derive(Debug)]
struct CreateServiceUseCase {
    account: Account,
}
struct UseCaseRes {
    pub service: Service,
}

#[derive(Debug)]
enum UseCaseErrors {
    StorageError,
}

#[async_trait::async_trait(?Send)]
impl UseCase for CreateServiceUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    type Context = NettuContext;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let service = Service::new(&self.account.id);
        let res = ctx.repos.service_repo.insert(&service).await;
        match res {
            Ok(_) => Ok(UseCaseRes { service }),
            Err(_) => Err(UseCaseErrors::StorageError),
        }
    }
}
