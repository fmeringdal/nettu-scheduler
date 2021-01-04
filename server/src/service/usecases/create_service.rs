use crate::api::Context;
use crate::{
    account::domain::Account,
    service::{domain::Service, repos::IServiceRepo},
    shared::auth::protect_account_route,
};
use actix_web::{web, HttpRequest, HttpResponse};

use serde::Serialize;
use std::sync::Arc;

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

    let res = create_service_usecase(
        CreateServiceReq { account },
        CreateServiceUseCaseCtx {
            service_repo: Arc::clone(&ctx.repos.service_repo),
        },
    )
    .await;

    match res {
        Ok(usecase_res) => {
            let res = CreateServiceRes {
                service_id: usecase_res.service.id.clone(),
            };
            HttpResponse::Created().json(res)
        }
        Err(e) => match e {
            UsecaseErrors::StorageError => HttpResponse::InternalServerError().finish(),
        },
    }
}

struct CreateServiceReq {
    account: Account,
}

struct CreateServiceUsecaseRes {
    pub service: Service,
}

enum UsecaseErrors {
    StorageError,
}

struct CreateServiceUseCaseCtx {
    pub service_repo: Arc<dyn IServiceRepo>,
}

async fn create_service_usecase(
    req: CreateServiceReq,
    ctx: CreateServiceUseCaseCtx,
) -> Result<CreateServiceUsecaseRes, UsecaseErrors> {
    let service = Service::new(&req.account.id);
    let res = ctx.service_repo.insert(&service).await;
    match res {
        Ok(_) => Ok(CreateServiceUsecaseRes { service }),
        Err(_) => Err(UsecaseErrors::StorageError),
    }
}
