use crate::{
    error::NettuError,
    shared::{
        auth::protect_account_route,
        usecase::{execute, UseCase},
    },
};
use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_api_structs::get_service::*;
use nettu_scheduler_domain::{Account, ServiceWithUsers, ID};
use nettu_scheduler_infra::NettuContext;

pub async fn get_service_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let usecase = GetServiceUseCase {
        account,
        service_id: path_params.service_id.clone(),
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| HttpResponse::Ok().json(APIResponse::new(usecase_res.service)))
        .map_err(NettuError::from)
}

#[derive(Debug)]
struct GetServiceUseCase {
    account: Account,
    service_id: ID,
}

#[derive(Debug)]
struct UseCaseRes {
    pub service: ServiceWithUsers,
}

#[derive(Debug)]
enum UseCaseError {
    NotFound(ID),
}

impl From<UseCaseError> for NettuError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::NotFound(id) => {
                Self::NotFound(format!("The service with id: {} was not found.", id))
            }
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetServiceUseCase {
    type Response = UseCaseRes;

    type Error = UseCaseError;
    const NAME: &'static str = "GetService";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Error> {
        let res = ctx.repos.services.find_with_users(&self.service_id).await;
        match res {
            Some(service) if service.account_id == self.account.id => Ok(UseCaseRes { service }),
            _ => Err(UseCaseError::NotFound(self.service_id.clone())),
        }
    }
}
