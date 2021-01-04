use crate::{
    account::domain::Account,
    service::{domain::Service, repos::IServiceRepo},
    shared::{
        auth::protect_account_route,
        usecase::{perform, Usecase},
    },
};
use crate::{api::Context, service::domain::ServiceDTO};
use actix_web::{web, HttpRequest, HttpResponse};
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
) -> HttpResponse {
    let account = match protect_account_route(&http_req, &ctx).await {
        Ok(a) => a,
        Err(res) => return res,
    };

    let usecase = GetServiceUseCase {
        account,
        service_id: path_params.service_id.clone(),
    };

    let res = perform(usecase, &ctx).await;

    match res {
        Ok(res) => {
            let dto = ServiceDTO::new(&res.service);
            HttpResponse::Ok().json(dto)
        }
        Err(e) => match e {
            UseCaseErrors::NotFoundError => HttpResponse::NotFound().finish(),
        },
    }
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
impl Usecase for GetServiceUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn perform(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let res = ctx.repos.service_repo.find(&self.service_id).await;
        match res {
            Some(service) if service.account_id == self.account.id => Ok(UseCaseRes { service }),
            _ => Err(UseCaseErrors::NotFoundError),
        }
    }
}
