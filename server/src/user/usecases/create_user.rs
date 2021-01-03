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
#[serde(rename_all = "camelCase")]
pub struct BodyParams {
    pub user_id: String,
}

pub async fn create_user_controller(
    http_req: HttpRequest,
    body: web::Json<BodyParams>,
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

    let res = create_user_usecase(
        UsecaseReq {
            account,
            external_user_id: body.user_id.clone(),
        },
        UsecaseCtx {
            user_repo: Arc::clone(&ctx.repos.user_repo),
        },
    )
    .await;

    match res {
        Ok(usecase_res) => {
            let res = UserDTO::new(&usecase_res.user);
            HttpResponse::Created().json(res)
        }
        Err(e) => match e {
            UsecaseErrors::StorageError => HttpResponse::InternalServerError().finish(),
            UsecaseErrors::UserAlreadyExists => HttpResponse::Conflict()
                .body("A user with that userId already exist. UserIds need to be unique."),
        },
    }
}

struct UsecaseReq {
    account: Account,
    external_user_id: String,
}

struct UsecaseRes {
    pub user: User,
}

enum UsecaseErrors {
    StorageError,
    UserAlreadyExists,
}

struct UsecaseCtx {
    pub user_repo: Arc<dyn IUserRepo>,
}

async fn create_user_usecase(
    req: UsecaseReq,
    ctx: UsecaseCtx,
) -> Result<UsecaseRes, UsecaseErrors> {
    let user = User::new(&req.account.id, &req.external_user_id);

    if let Some(_existing_user) = ctx.user_repo.find(&user.id).await {
        return Err(UsecaseErrors::UserAlreadyExists);
    }

    let res = ctx.user_repo.insert(&user).await;
    match res {
        Ok(_) => Ok(UsecaseRes { user }),
        Err(_) => Err(UsecaseErrors::StorageError),
    }
}
