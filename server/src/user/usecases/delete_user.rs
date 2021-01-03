use crate::{
    account::domain::Account,
    service::{domain::Service, repos::IServiceRepo},
    shared::auth::{protect_account_route, AccountAuthContext},
};
use crate::{
    api::Context,
    user::{
        domain::{User, UserDTO},
        repos::IUserRepo,
    },
};
use actix_web::{web, HttpRequest, HttpResponse};

use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize)]
pub struct PathParams {
    pub user_id: String,
}

pub async fn delete_user_controller(
    http_req: HttpRequest,
    path_params: web::Json<PathParams>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let account = match protect_account_route(
        &http_req,
        &AccountAuthContext {
            account_repo: Arc::clone(&ctx.repos.account_repo),
        },
    )
    .await
    {
        Ok(a) => a,
        Err(res) => return res,
    };

    let user_id = User::create_id(&account.id, &path_params.user_id);
    let res = delete_user_usecase(
        UsecaseReq { account, user_id },
        UsecaseCtx {
            user_repo: Arc::clone(&ctx.repos.user_repo),
        },
    )
    .await;

    match res {
        Ok(usecase_res) => HttpResponse::Ok().body(format!(
            "Used: {} is deleted.",
            usecase_res.user.external_id
        )),
        Err(e) => match e {
            UsecaseErrors::StorageError => HttpResponse::InternalServerError().finish(),
            UsecaseErrors::UserNotFoundError => HttpResponse::NotFound()
                .body("A user with that userId already exist. UserIds need to be unique."),
        },
    }
}

struct UsecaseReq {
    account: Account,
    user_id: String,
}

struct UsecaseRes {
    pub user: User,
}

enum UsecaseErrors {
    StorageError,
    UserNotFoundError,
}

struct UsecaseCtx {
    pub user_repo: Arc<dyn IUserRepo>,
}

// TODOS:
// - REMOVE ALL CALENDARS
// - REMOVE ALL EVENTS
// - REMOVE FROM ALL SERVICES
async fn delete_user_usecase(
    req: UsecaseReq,
    ctx: UsecaseCtx,
) -> Result<UsecaseRes, UsecaseErrors> {
    let user = match ctx.user_repo.find(&req.user_id).await {
        Some(u) if u.account_id == req.account.id => {
            match ctx.user_repo.delete(&req.user_id).await {
                Some(u) => u,
                None => return Err(UsecaseErrors::StorageError)
            }
        },
        _ => return Err(UsecaseErrors::UserNotFoundError),
    };

    Ok(UsecaseRes { user })
}
