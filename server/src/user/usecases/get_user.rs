use crate::{account::domain::Account, shared::auth::protect_account_route, user::domain::UserDTO};
use crate::{
    api::Context,
    user::{domain::User, repos::IUserRepo},
};
use actix_web::{web, HttpRequest, HttpResponse};

use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct PathParams {
    pub user_id: String,
}

pub async fn get_user_controller(
    http_req: HttpRequest,
    path_params: web::Json<PathParams>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let account = match protect_account_route(&http_req, &ctx).await {
        Ok(a) => a,
        Err(res) => return res,
    };

    let user_id = User::create_id(&account.id, &path_params.user_id);
    let res = get_user_usecase(
        UsecaseReq { account, user_id },
        UsecaseCtx {
            user_repo: Arc::clone(&ctx.repos.user_repo),
        },
    )
    .await;

    match res {
        Ok(usecase_res) => {
            let dto = UserDTO::new(&usecase_res.user);
            HttpResponse::Ok().json(dto)
        }
        Err(e) => match e {
            UsecaseErrors::UserNotFoundError => {
                HttpResponse::NotFound().body("A user with that id was not found.")
            }
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
    UserNotFoundError,
}

struct UsecaseCtx {
    pub user_repo: Arc<dyn IUserRepo>,
}

async fn get_user_usecase(req: UsecaseReq, ctx: UsecaseCtx) -> Result<UsecaseRes, UsecaseErrors> {
    let user = match ctx.user_repo.find(&req.user_id).await {
        Some(u) if u.account_id == req.account.id => u,
        _ => return Err(UsecaseErrors::UserNotFoundError),
    };

    Ok(UsecaseRes { user })
}
