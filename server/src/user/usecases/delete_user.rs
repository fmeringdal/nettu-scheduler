use crate::{account::domain::Account, shared::auth::protect_account_route};
use crate::{
    api::Context,
    shared::usecase::{execute, Usecase},
    user::domain::User,
};
use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PathParams {
    pub user_id: String,
}

pub async fn delete_user_controller(
    http_req: HttpRequest,
    path_params: web::Json<PathParams>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let account = match protect_account_route(&http_req, &ctx).await {
        Ok(a) => a,
        Err(res) => return res,
    };

    let user_id = User::create_id(&account.id, &path_params.user_id);
    let usecase = DeleteUserUseCase { account, user_id };
    let res = execute(usecase, &ctx).await;

    match res {
        Ok(usecase_res) => HttpResponse::Ok().body(format!(
            "Used: {} is deleted.",
            usecase_res.user.external_id
        )),
        Err(e) => match e {
            UseCaseErrors::StorageError => HttpResponse::InternalServerError().finish(),
            UseCaseErrors::UserNotFoundError => {
                HttpResponse::NotFound().body("A user with that id was not found.")
            }
        },
    }
}

struct DeleteUserUseCase {
    account: Account,
    user_id: String,
}

struct UseCaseRes {
    pub user: User,
}

#[derive(Debug)]
enum UseCaseErrors {
    StorageError,
    UserNotFoundError,
}

#[async_trait::async_trait(?Send)]
impl Usecase for DeleteUserUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    type Context = Context;

    // TODOS:
    // - REMOVE ALL CALENDARS
    // - REMOVE ALL EVENTS
    // - REMOVE FROM ALL SERVICES
    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let user = match ctx.repos.user_repo.find(&self.user_id).await {
            Some(u) if u.account_id == self.account.id => {
                match ctx.repos.user_repo.delete(&self.user_id).await {
                    Some(u) => u,
                    None => return Err(UseCaseErrors::StorageError),
                }
            }
            _ => return Err(UseCaseErrors::UserNotFoundError),
        };

        Ok(UseCaseRes { user })
    }
}
