use crate::shared::{
    auth::{account_can_modify_user, protect_route},
    usecase::{execute, UseCase},
};
use crate::{error::NettuError, shared::auth::protect_account_route};
use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_api_structs::remove_integration::*;
use nettu_scheduler_domain::{IntegrationProvider, User};
use nettu_scheduler_infra::NettuContext;

fn handle_error(e: UseCaseErrors) -> NettuError {
    match e {
        UseCaseErrors::StorageError => NettuError::InternalError,
        UseCaseErrors::IntegrationNotFound => NettuError::NotFound(
            "Did not find an integration between that user and provider".into(),
        ),
    }
}

pub async fn remove_integration_admin_controller(
    http_req: HttpRequest,
    path: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;
    let user = account_can_modify_user(&account, &path.user_id, &ctx).await?;

    let usecase = OAuthIntegrationUseCase {
        user,
        provider: path.0.provider,
    };

    execute(usecase, &ctx)
        .await
        .map(|res| HttpResponse::Ok().json(APIResponse::new(res.user)))
        .map_err(handle_error)
}

pub async fn remove_integration_controller(
    http_req: HttpRequest,
    path: web::Json<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let (user, _) = protect_route(&http_req, &ctx).await?;

    let usecase = OAuthIntegrationUseCase {
        user,
        provider: path.0.provider,
    };

    execute(usecase, &ctx)
        .await
        .map(|res| HttpResponse::Ok().json(APIResponse::new(res.user)))
        .map_err(handle_error)
}

#[derive(Debug)]
pub struct OAuthIntegrationUseCase {
    pub user: User,
    pub provider: IntegrationProvider,
}

#[derive(Debug)]
pub struct UseCaseRes {
    pub user: User,
}

#[derive(Debug)]
pub enum UseCaseErrors {
    StorageError,
    IntegrationNotFound,
}

#[async_trait::async_trait(?Send)]
impl UseCase for OAuthIntegrationUseCase {
    type Response = UseCaseRes;
    type Errors = UseCaseErrors;

    const NAME: &'static str = "RemoveIntegration";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors> {
        let user_integrations = ctx
            .repos
            .user_integrations
            .find(&self.user.id)
            .await
            .map_err(|_| UseCaseErrors::StorageError)?;
        if user_integrations
            .into_iter()
            .find(|i| i.provider == self.provider)
            .is_none()
        {
            return Err(UseCaseErrors::IntegrationNotFound);
        };

        ctx.repos
            .user_integrations
            .delete(&self.user.id, self.provider.clone())
            .await
            .map(|_| UseCaseRes {
                user: self.user.clone(),
            })
            .map_err(|_| UseCaseErrors::StorageError)
    }
}
