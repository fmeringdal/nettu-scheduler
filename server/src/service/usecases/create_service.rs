use crate::{
    account::domain::Account, service::domain::Service, shared::auth::protect_account_route,
};
use crate::{
    api::Context,
    shared::usecase::{perform, Usecase},
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
) -> HttpResponse {
    let account = match protect_account_route(&http_req, &ctx).await {
        Ok(a) => a,
        Err(res) => return res,
    };

    let usecase = CreateServiceUseCase { account };
    let res = perform(usecase, &ctx).await;

    match res {
        Ok(usecase_res) => {
            let res = CreateServiceRes {
                service_id: usecase_res.service.id,
            };
            HttpResponse::Created().json(res)
        }
        Err(e) => match e {
            UseCaseErrors::StorageError => HttpResponse::InternalServerError().finish(),
        },
    }
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
impl Usecase for CreateServiceUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn perform(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let service = Service::new(&self.account.id);
        let res = ctx.repos.service_repo.insert(&service).await;
        match res {
            Ok(_) => Ok(UseCaseRes { service }),
            Err(_) => Err(UseCaseErrors::StorageError),
        }
    }
}
