use crate::{
    account::domain::Account,
    shared::{
        auth::protect_account_route,
        usecase::{perform, Usecase},
    },
    user::domain::UserDTO,
};
use crate::{
    api::Context,
    user::{domain::User, repos::IUserRepo},
};
use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;

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
    let usecase = GetUserUseCase { account, user_id };
    let res = perform(usecase, &ctx).await;

    match res {
        Ok(usecase_res) => {
            let dto = UserDTO::new(&usecase_res.user);
            HttpResponse::Ok().json(dto)
        }
        Err(e) => match e {
            UseCaseErrors::UserNotFoundError => {
                HttpResponse::NotFound().body("A user with that id was not found.")
            }
        },
    }
}

struct GetUserUseCase {
    account: Account,
    user_id: String,
}

struct UseCaseRes {
    pub user: User,
}

#[derive(Debug)]
enum UseCaseErrors {
    UserNotFoundError,
}

#[async_trait::async_trait(?Send)]
impl Usecase for GetUserUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn perform(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let user = match ctx.repos.user_repo.find(&self.user_id).await {
            Some(u) if u.account_id == self.account.id => u,
            _ => return Err(UseCaseErrors::UserNotFoundError),
        };

        Ok(UseCaseRes { user })
    }
}
