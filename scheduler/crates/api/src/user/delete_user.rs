use crate::shared::usecase::{execute, UseCase};
use crate::{error::NettuError, shared::auth::protect_account_route};
use actix_web::{web, HttpRequest, HttpResponse};
use futures::future::join_all;
use nettu_scheduler_api_structs::delete_user::*;
use nettu_scheduler_domain::{Account, User, ID};
use nettu_scheduler_infra::NettuContext;

pub async fn delete_user_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let usecase = DeleteUserUseCase {
        account,
        user_id: path_params.user_id.clone(),
    };
    execute(usecase, &ctx)
        .await
        .map(|usecase_res| HttpResponse::Ok().json(APIResponse::new(usecase_res.user)))
        .map_err(|e| match e {
            UseCaseErrors::StorageError => NettuError::InternalError,
            UseCaseErrors::UserNotFound => NettuError::NotFound(format!(
                "A user with id: {}, was not found.",
                path_params.user_id
            )),
        })
}

#[derive(Debug)]
struct DeleteUserUseCase {
    account: Account,
    user_id: ID,
}

#[derive(Debug)]
struct UseCaseRes {
    pub user: User,
}

#[derive(Debug)]
enum UseCaseErrors {
    StorageError,
    UserNotFound,
}

#[async_trait::async_trait(?Send)]
impl UseCase for DeleteUserUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    const NAME: &'static str = "DeleteUser";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors> {
        let user = match ctx.repos.users.find(&self.user_id).await {
            Some(u) if u.account_id == self.account.id => {
                match ctx.repos.users.delete(&self.user_id).await {
                    Some(u) => u,
                    None => return Err(UseCaseErrors::StorageError),
                }
            }
            _ => return Err(UseCaseErrors::UserNotFound),
        };

        let _ = join_all(vec![
            ctx.repos.calendars.delete_by_user(&user.id),
            ctx.repos.events.delete_by_user(&user.id),
            ctx.repos.schedules.delete_by_user(&user.id),
        ]);
        let _ = ctx.repos.services.remove_user_from_services(&user.id).await;

        Ok(UseCaseRes { user })
    }
}
