use crate::{
    account::domain::Account,
    api::NettuError,
    shared::{
        auth::protect_account_route,
        usecase::{execute, Usecase},
    },
    user::domain::UserDTO,
};
use crate::{api::Context, user::domain::User};
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
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let user_id = User::create_id(&account.id, &path_params.user_id);
    let usecase = GetUserUseCase { account, user_id };
    execute(usecase, &ctx)
        .await
        .map(|usecase_res| {
            let dto = UserDTO::new(&usecase_res.user);
            HttpResponse::Ok().json(dto)
        })
        .map_err(|e| match e {
            UseCaseErrors::UserNotFoundError => {
                NettuError::NotFound(format!("A user with id: {}, was not found.", path_params.user_id))
            }
        })
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

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let user = match ctx.repos.user_repo.find(&self.user_id).await {
            Some(u) if u.account_id == self.account.id => u,
            _ => return Err(UseCaseErrors::UserNotFoundError),
        };

        Ok(UseCaseRes { user })
    }
}
