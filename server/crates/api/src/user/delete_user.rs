use crate::shared::usecase::{execute, UseCase};
use crate::{error::NettuError, shared::auth::protect_account_route};
use actix_web::{web, HttpRequest, HttpResponse};
use futures::future::join_all;
use nettu_scheduler_api_structs::delete_user::*;
use nettu_scheduler_domain::{Account, User};
use nettu_scheduler_infra::NettuContext;

pub async fn delete_user_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let user_id = User::create_id(&account.id, &path_params.user_id);
    let usecase = DeleteUserUseCase { account, user_id };
    execute(usecase, &ctx)
        .await
        .map(|usecase_res| HttpResponse::Created().json(APIResponse::new(usecase_res.user)))
        .map_err(|e| match e {
            UseCaseErrors::StorageError => NettuError::InternalError,
            UseCaseErrors::UserNotFoundError => NettuError::NotFound(format!(
                "A user with id: {}, was not found.",
                path_params.user_id
            )),
        })
}

#[derive(Debug)]
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
impl UseCase for DeleteUserUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    type Context = NettuContext;

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

        let _ = join_all(vec![
            ctx.repos.calendar_repo.delete_by_user(&user.id),
            ctx.repos.event_repo.delete_by_user(&user.id),
            ctx.repos.schedule_repo.delete_by_user(&user.id),
        ]);
        let _ = ctx
            .repos
            .service_repo
            .remove_user_from_services(&user.id)
            .await;

        Ok(UseCaseRes { user })
    }
}
