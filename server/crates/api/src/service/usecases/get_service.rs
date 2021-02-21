use crate::{
    error::NettuError,
    service::dtos::ServiceDTO,
    shared::{
        auth::protect_account_route,
        usecase::{execute, UseCase},
    },
};
use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_core::{Account, Service};
use nettu_scheduler_infra::Context;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct PathParams {
    pub service_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetServiceRes {
    pub service_id: String,
}

pub async fn get_service_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    ctx: web::Data<Context>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let usecase = GetServiceUseCase {
        account,
        service_id: path_params.service_id.clone(),
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| {
            let dto = ServiceDTO::new(&usecase_res.service);
            HttpResponse::Ok().json(dto)
        })
        .map_err(|e| match e {
            UseCaseErrors::NotFoundError => NettuError::NotFound(format!(
                "The service with id: {} was not found.",
                path_params.service_id
            )),
        })
}

struct GetServiceUseCase {
    account: Account,
    service_id: String,
}

struct UseCaseRes {
    pub service: Service,
}

#[derive(Debug)]
enum UseCaseErrors {
    NotFoundError,
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetServiceUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let res = ctx.repos.service_repo.find(&self.service_id).await;
        match res {
            Some(service) if service.account_id == self.account.id => Ok(UseCaseRes { service }),
            _ => Err(UseCaseErrors::NotFoundError),
        }
    }
}
