use crate::shared::usecase::{execute, UseCase};
use crate::{error::NettuError, shared::auth::protect_account_route};
use actix_web::{web, HttpResponse};
use nettu_scheduler_api_structs::remove_account_integration::{APIResponse, PathParams};
use nettu_scheduler_domain::{Account, IntegrationProvider};
use nettu_scheduler_infra::NettuContext;

pub async fn remove_account_integration_controller(
    http_req: web::HttpRequest,
    path: web::Json<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let usecase = RemoveAccountIntegrationUseCase {
        account,
        provider: path.0.provider,
    };

    execute(usecase, &ctx)
        .await
        .map(|res| {
            HttpResponse::Ok().json(APIResponse::from(
                "Provider integration removed from account",
            ))
        })
        .map_err(|e| match e {
            UseCaseErrors::StorageError => NettuError::InternalError,
            UseCaseErrors::IntegrationNotFound => NettuError::NotFound(
                "Did not find an integration between the given account and provider".into(),
            ),
        })
}

#[derive(Debug)]
pub struct RemoveAccountIntegrationUseCase {
    pub account: Account,
    pub provider: IntegrationProvider,
}

#[derive(Debug, PartialEq)]
pub enum UseCaseErrors {
    StorageError,
    IntegrationNotFound,
}

#[async_trait::async_trait(?Send)]
impl UseCase for RemoveAccountIntegrationUseCase {
    type Response = ();

    type Errors = UseCaseErrors;

    const NAME: &'static str = "RemoveAccountIntegration";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors> {
        let acc_integrations = ctx
            .repos
            .account_integrations
            .find(&self.account.id)
            .await
            .map_err(|_| UseCaseErrors::StorageError)?;
        if acc_integrations
            .iter()
            .find(|i| i.provider == self.provider)
            .is_none()
        {
            return Err(UseCaseErrors::IntegrationNotFound);
        }

        ctx.repos
            .account_integrations
            .delete(&self.account.id, self.provider.clone())
            .await
            .map_err(|_| UseCaseErrors::StorageError)
    }
}
