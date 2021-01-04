use crate::{
    account::domain::Account,
    service::{domain::Service, repos::IServiceRepo},
    shared::auth::protect_account_route,
};
use crate::{api::Context, service::domain::ServiceDTO};
use actix_web::{web, HttpRequest, HttpResponse};

use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

    let res = get_service_usecase(
        UsecaseReq {
            account,
            service_id: path_params.service_id.clone(),
        },
        UsecaseCtx {
            service_repo: Arc::clone(&ctx.repos.service_repo),
        },
    )
    .await;

    match res {
        Ok(res) => {
            let dto = ServiceDTO::new(&res.service);
            HttpResponse::Ok().json(dto)
        }
        Err(e) => match e {
            UsecaseErrors::NotFoundError => HttpResponse::NotFound().finish(),
        },
    }
}

struct UsecaseReq {
    account: Account,
    service_id: String,
}

struct UsecaseRes {
    pub service: Service,
}

enum UsecaseErrors {
    NotFoundError,
}

struct UsecaseCtx {
    pub service_repo: Arc<dyn IServiceRepo>,
}

async fn get_service_usecase(
    req: UsecaseReq,
    ctx: UsecaseCtx,
) -> Result<UsecaseRes, UsecaseErrors> {
    let res = ctx.service_repo.find(&req.service_id).await;
    match res {
        Some(service) if service.account_id == req.account.id => Ok(UsecaseRes { service }),
        _ => Err(UsecaseErrors::NotFoundError),
    }
}
