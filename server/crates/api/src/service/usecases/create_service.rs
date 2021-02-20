use crate::{
    account::domain::Account, api::NettuError, service::domain::Service,
    shared::auth::protect_account_route,
};
use crate::{
    api::Context,
    shared::usecase::{execute, UseCase},
};
use actix_web::{web, HttpRequest, HttpResponse};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateServiceRes {
    pub service_id: String,
}

pub async fn create_service_controller(
    http_req: HttpRequest,
    ctx: web::Data<Context>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let usecase = CreateServiceUseCase { account };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| {
            let res = CreateServiceRes {
                service_id: usecase_res.service.id,
            };
            HttpResponse::Created().json(res)
        })
        .map_err(|e| match e {
            UseCaseErrors::StorageError => NettuError::InternalError,
        })
}

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

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let service = Service::new(&self.account.id);
        let res = ctx.repos.service_repo.insert(&service).await;
        match res {
            Ok(_) => Ok(UseCaseRes { service }),
            Err(_) => Err(UseCaseErrors::StorageError),
        }
    }
}
