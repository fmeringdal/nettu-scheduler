use crate::shared::usecase::{execute, UseCase};
use crate::{error::NettuError, shared::auth::protect_account_route};
use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_api_structs::update_service::*;
use nettu_scheduler_domain::{Metadata, Service, ServiceMultiPersonOptions, ID};
use nettu_scheduler_infra::NettuContext;

pub async fn update_service_controller(
    http_req: HttpRequest,
    body: web::Json<RequestBody>,
    mut path: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let body = body.0;
    let usecase = UpdateServiceUseCase {
        account_id: account.id,
        service_id: std::mem::take(&mut path.service_id),
        metadata: body.metadata,
        multi_person: body.multi_person,
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| HttpResponse::Ok().json(APIResponse::new(usecase_res.service)))
        .map_err(NettuError::from)
}

#[derive(Debug)]
struct UpdateServiceUseCase {
    account_id: ID,
    service_id: ID,
    metadata: Option<Metadata>,
    multi_person: Option<ServiceMultiPersonOptions>,
}
#[derive(Debug)]
struct UseCaseRes {
    pub service: Service,
}

#[derive(Debug)]
enum UseCaseError {
    StorageError,
    ServiceNotFound(ID),
}

impl From<UseCaseError> for NettuError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::ServiceNotFound(id) => {
                Self::NotFound(format!("Service with id: {} was not found.", id))
            }
            UseCaseError::StorageError => Self::InternalError,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for UpdateServiceUseCase {
    type Response = UseCaseRes;

    type Error = UseCaseError;

    const NAME: &'static str = "UpdateService";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Error> {
        let mut service = match ctx.repos.services.find(&self.service_id).await {
            Some(service) if service.account_id == self.account_id => service,
            _ => return Err(UseCaseError::ServiceNotFound(self.service_id.clone())),
        };

        if let Some(metadata) = &self.metadata {
            service.metadata = metadata.clone();
        }
        if let Some(opts) = &self.multi_person {
            if let ServiceMultiPersonOptions::Group(new_count) = opts {
                if let ServiceMultiPersonOptions::Group(old_count) = &service.multi_person {
                    if new_count > old_count {
                        // Delete all calendar events for this service, because
                        // then it should be possible for more people to book
                        ctx.repos
                            .events
                            .delete_by_service(&service.id)
                            .await
                            .map_err(|_| UseCaseError::StorageError)?;
                    }
                }
            }
            service.multi_person = opts.clone();
        }

        ctx.repos
            .services
            .save(&service)
            .await
            .map(|_| UseCaseRes { service })
            .map_err(|_| UseCaseError::StorageError)
    }
}
