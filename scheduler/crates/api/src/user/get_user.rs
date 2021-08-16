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
        .map_err(NettuError::from)
}

#[derive(Debug)]
struct GetUserUseCase {
    account: Account,
    user_id: ID,
}

#[derive(Debug)]
struct UseCaseRes {
    pub user: User,
}

#[derive(Debug)]
enum UseCaseError {
    UserNotFound(ID),
}

impl From<UseCaseError> for NettuError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::UserNotFound(id) => {
                Self::NotFound(format!("A user with id: {}, was not found.", id))
            }
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetUserUseCase {
    type Response = UseCaseRes;

    type Error = UseCaseError;

    const NAME: &'static str = "GetUser";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Error> {
        let user = match ctx.repos.users.find(&self.user_id).await {
            Some(u) if u.account_id == self.account.id => u,
            _ => return Err(UseCaseError::UserNotFound(self.user_id.clone())),
        };

        Ok(UseCaseRes { user })
    }
}
