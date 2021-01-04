use crate::{
    account::domain::Account,
    service::{domain::ServiceResource, repos::IServiceRepo},
    shared::auth::protect_account_route,
    user::domain::User,
};
use crate::{api::Context, user::repos::IUserRepo};
use actix_web::{web, HttpRequest, HttpResponse};

use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct PathParams {
    service_id: String,
    user_id: String,
}

pub async fn remove_user_from_service_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let account = match protect_account_route(&http_req, &ctx).await {
        Ok(a) => a,
        Err(res) => return res,
    };

    let user_id = User::create_id(&account.id, &path_params.user_id);
    let req = UsecaseReq {
        account,
        service_id: path_params.service_id.to_owned(),
        user_id,
    };

    let res = remove_user_from_service_usecase(
        req,
        UsecaseCtx {
            service_repo: Arc::clone(&ctx.repos.service_repo),
            user_repo: Arc::clone(&ctx.repos.user_repo),
        },
    )
    .await;

    match res {
        Ok(_) => HttpResponse::Ok().body("Service successfully updated"),
        Err(e) => match e {
            UsecaseErrors::StorageError => HttpResponse::InternalServerError().finish(),
            UsecaseErrors::ServiceNotFoundError => {
                HttpResponse::NotFound().body("The requested service was not found")
            }
            UsecaseErrors::UserNotFoundError => {
                HttpResponse::NotFound().body("The specified user was not found in the service")
            }
        },
    }
}

struct UsecaseReq {
    pub account: Account,
    pub service_id: String,
    pub user_id: String,
}

struct UsecaseRes {
    pub resource: ServiceResource,
}

enum UsecaseErrors {
    StorageError,
    ServiceNotFoundError,
    UserNotFoundError,
}

struct UsecaseCtx {
    pub service_repo: Arc<dyn IServiceRepo>,
    pub user_repo: Arc<dyn IUserRepo>,
}

async fn remove_user_from_service_usecase(
    req: UsecaseReq,
    ctx: UsecaseCtx,
) -> Result<UsecaseRes, UsecaseErrors> {
    let mut service = match ctx.service_repo.find(&req.service_id).await {
        Some(service) if service.account_id == req.account.id => service,
        _ => return Err(UsecaseErrors::ServiceNotFoundError),
    };

    match service.remove_user(&req.user_id) {
        Some(resource) => match ctx.service_repo.save(&service).await {
            Ok(_) => Ok(UsecaseRes { resource }),
            Err(_) => Err(UsecaseErrors::StorageError),
        },
        None => Err(UsecaseErrors::UserNotFoundError),
    }
}
