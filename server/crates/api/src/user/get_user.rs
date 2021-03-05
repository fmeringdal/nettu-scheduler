use crate::{
    error::NettuError,
    shared::{
        auth::protect_account_route,
        usecase::{execute, UseCase},
    },
};
use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_api_structs::get_user::*;
use nettu_scheduler_domain::{Account, User, ID};
use nettu_scheduler_infra::NettuContext;

pub async fn get_user_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let usecase = GetUserUseCase {
        account,
        user_id: path_params.user_id.clone(),
    };
    execute(usecase, &ctx)
        .await
        .map(|usecase_res| HttpResponse::Ok().json(APIResponse::new(usecase_res.user)))
        .map_err(|e| match e {
            UseCaseErrors::UserNotFound => NettuError::NotFound(format!(
                "A user with id: {}, was not found.",
                path_params.user_id
            )),
        })
}

#[derive(Debug)]
struct GetUserUseCase {
    account: Account,
    user_id: ID,
}

struct UseCaseRes {
    pub user: User,
}

#[derive(Debug)]
enum UseCaseErrors {
    UserNotFound,
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetUserUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    type Context = NettuContext;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let user = match ctx.repos.user_repo.find(&self.user_id).await {
            Some(u) if u.account_id == self.account.id => u,
            _ => return Err(UseCaseErrors::UserNotFound),
        };

        Ok(UseCaseRes { user })
    }
}
