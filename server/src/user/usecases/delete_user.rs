use crate::{account::{domain::Account, usecases}, shared::auth::protect_account_route};
use crate::{
    api::Context,
    user::{domain::User, repos::IUserRepo},
    shared::usecase::{perform, Usecase}
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
    let res = perform(usecase, &ctx).await;

    match res {
        Ok(usecase_res) => HttpResponse::Ok().body(format!(
            "Used: {} is deleted.",
            usecase_res.user.external_id
        )),
        Err(e) => match e {
            UsecaseErrors::StorageError => HttpResponse::InternalServerError().finish(),
            UsecaseErrors::UserNotFoundError => {
                HttpResponse::NotFound().body("A user with that id was not found.")
            }
        },
    }
}

struct DeleteUserUseCase {
    account: Account,
    user_id: String,
}

struct UsecaseRes {
    pub user: User,
}

#[derive(Debug)]
enum UsecaseErrors {
    StorageError,
    UserNotFoundError,
}

#[async_trait::async_trait]
impl Usecase for DeleteUserUseCase {
    type Response = UsecaseRes;

    type Errors = UsecaseErrors;

    type Context = Context;


    // TODOS:
    // - REMOVE ALL CALENDARS
    // - REMOVE ALL EVENTS
    // - REMOVE FROM ALL SERVICES
    async fn perform(&self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let user = match ctx.repos.user_repo.find(&self.user_id).await {
            Some(u) if u.account_id == self.account.id => {
                match ctx.repos.user_repo.delete(&self.user_id).await {
                    Some(u) => u,
                    None => return Err(UsecaseErrors::StorageError),
                }
            }
            _ => return Err(UsecaseErrors::UserNotFoundError),
        };
    
        Ok(UsecaseRes { user })
    }
}