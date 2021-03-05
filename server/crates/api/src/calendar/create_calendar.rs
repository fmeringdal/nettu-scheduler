use crate::shared::auth::Permission;
use crate::shared::{
    auth::{protect_account_route, protect_route},
    usecase::{execute_with_policy, PermissionBoundary, UseCaseErrorContainer},
};
use crate::{
    error::NettuError,
    shared::usecase::{execute, UseCase},
};
use actix_web::{web, HttpResponse};
use nettu_scheduler_api_structs::create_calendar::{APIResponse, PathParams};
use nettu_scheduler_domain::{Calendar, ID};
use nettu_scheduler_infra::NettuContext;

pub async fn create_calendar_admin_controller(
    http_req: web::HttpRequest,
    path_params: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let usecase = CreateCalendarUseCase {
        user_id: path_params.user_id.clone(),
        account_id: account.id,
    };

    execute(usecase, &ctx)
        .await
        .map(|calendar| HttpResponse::Created().json(APIResponse::new(calendar)))
        .map_err(|e| match e {
            UseCaseErrors::StorageError => NettuError::InternalError,
            UseCaseErrors::UserNotFound => NettuError::NotFound(format!(
                "The user with id: {}, was not found.",
                path_params.user_id
            )),
        })
}

pub async fn create_calendar_controller(
    http_req: web::HttpRequest,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let (user, policy) = protect_route(&http_req, &ctx).await?;

    let usecase = CreateCalendarUseCase {
        user_id: user.id,
        account_id: user.account_id,
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|calendar| HttpResponse::Created().json(APIResponse::new(calendar)))
        .map_err(|e| {
            match e {
                UseCaseErrorContainer::Unauthorized(e) => NettuError::Unauthorized(e),
                UseCaseErrorContainer::UseCase(e) => match e {
                    UseCaseErrors::StorageError => NettuError::InternalError,
                    // This should never happen
                    UseCaseErrors::UserNotFound => {
                        NettuError::NotFound("The user was not found.".into())
                    }
                },
            }
        })
}

#[derive(Debug)]
struct CreateCalendarUseCase {
    pub user_id: ID,
    pub account_id: ID,
}

#[derive(Debug)]
enum UseCaseErrors {
    UserNotFound,
    StorageError,
}

#[async_trait::async_trait(?Send)]
impl UseCase for CreateCalendarUseCase {
    type Response = Calendar;

    type Errors = UseCaseErrors;

    

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors> {
        let _user = match ctx.repos.user_repo.find(&self.user_id).await {
            Some(user) if user.account_id == self.account_id => user,
            _ => return Err(UseCaseErrors::UserNotFound),
        };

        let calendar = Calendar::new(&self.user_id);

        let res = ctx.repos.calendar_repo.insert(&calendar).await;
        match res {
            Ok(_) => Ok(calendar),
            Err(_) => Err(UseCaseErrors::StorageError),
        }
    }
}

impl PermissionBoundary for CreateCalendarUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::CreateCalendar]
    }
}
