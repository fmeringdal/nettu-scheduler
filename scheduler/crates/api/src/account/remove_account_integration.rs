use crate::shared::usecase::{execute, UseCase};
use crate::{error::NettuError, shared::auth::protect_account_route};
use actix_web::{web, HttpResponse};
use nettu_scheduler_api_structs::remove_account_integration::{APIResponse, PathParams};
use nettu_scheduler_domain::{Account, IntegrationProvider};
use nettu_scheduler_infra::NettuContext;

pub async fn remove_account_integration_controller(
    http_req: web::HttpRequest,
    mut path: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let usecase = RemoveAccountIntegrationUseCase {
        account,
        provider: std::mem::take(&mut path.provider),
    };

    execute(usecase, &ctx)
        .await
        .map(|_| {
            HttpResponse::Ok().json(APIResponse::from(
                "Provider integration removed from account",
            ))
        })
        .map_err(NettuError::from)
}

#[derive(Debug)]
pub struct RemoveAccountIntegrationUseCase {
    pub account: Account,
    pub provider: IntegrationProvider,
}

#[derive(Debug, PartialEq)]
pub enum UseCaseError {
    StorageError,
    IntegrationNotFound,
}

impl From<UseCaseError> for NettuError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::StorageError => Self::InternalError,
            UseCaseError::IntegrationNotFound => Self::NotFound(
                "Did not find an integration between the given account and provider".into(),
            ),
        }
    }
}
#[async_trait::async_trait(?Send)]
impl UseCase for RemoveAccountIntegrationUseCase {
    type Response = ();

    type Error = UseCaseError;

    const NAME: &'static str = "RemoveAccountIntegration";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Error> {
        let acc_integrations = ctx
            .repos
            .account_integrations
            .find(&self.account.id)
            .await
            .map_err(|_| UseCaseError::StorageError)?;
        if !acc_integrations.iter().any(|i| i.provider == self.provider) {
            return Err(UseCaseError::IntegrationNotFound);
        }

        ctx.repos
            .account_integrations
            .delete(&self.account.id, self.provider.clone())
            .await
            .map_err(|_| UseCaseError::StorageError)
    }
}
