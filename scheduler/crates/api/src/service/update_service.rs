use crate::shared::usecase::{execute, UseCase};
use crate::{error::NettuError, shared::auth::protect_account_route};
use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_api_structs::update_service::*;
use nettu_scheduler_domain::{Metadata, Service, ID};
use nettu_scheduler_infra::NettuContext;

pub async fn update_service_controller(
    http_req: HttpRequest,
    body: web::Json<RequestBody>,
    path: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let usecase = UpdateServiceUseCase {
        account_id: account.id,
        service_id: path.0.service_id,
        metadata: body.0.metadata,
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| HttpResponse::Ok().json(APIResponse::new(usecase_res.service)))
        .map_err(|e| match e {
            UseCaseErrors::ServiceNotFound(id) => {
                NettuError::NotFound(format!("Service with id: {} was not found.", id))
            }
            UseCaseErrors::StorageError => NettuError::InternalError,
        })
}

#[derive(Debug)]
struct UpdateServiceUseCase {
    account_id: ID,
    service_id: ID,
    metadata: Option<Metadata>,
}
struct UseCaseRes {
    pub service: Service,
}

#[derive(Debug)]
enum UseCaseErrors {
    StorageError,
    ServiceNotFound(ID),
}

#[async_trait::async_trait(?Send)]
impl UseCase for UpdateServiceUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors> {
        let mut service = match ctx.repos.service_repo.find(&self.service_id).await {
            Some(service) if service.account_id == self.account_id => service,
            _ => return Err(UseCaseErrors::ServiceNotFound(self.service_id.clone())),
        };

        if let Some(metadata) = &self.metadata {
            service.metadata = metadata.clone();
        }

        ctx.repos
            .service_repo
            .save(&service)
            .await
            .map(|_| UseCaseRes { service })
            .map_err(|_| UseCaseErrors::StorageError)
    }
}
