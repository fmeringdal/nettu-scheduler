use crate::shared::usecase::{execute, UseCase};
use crate::{error::NettuError, shared::auth::protect_account_route};
use actix_web::{web, HttpResponse};
use nettu_scheduler_api_structs::add_account_integration::{APIResponse, RequestBody};
use nettu_scheduler_domain::{Account, AccountIntegration, IntegrationProvider};
use nettu_scheduler_infra::NettuContext;

pub async fn add_account_integration_controller(
    http_req: web::HttpRequest,
    body: web::Json<RequestBody>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let body = body.0;
    let usecase = AddAccountIntegrationUseCase {
        account,
        client_id: body.client_id,
        client_secret: body.client_secret,
        redirect_uri: body.redirect_uri,
        provider: body.provider,
    };

    execute(usecase, &ctx)
        .await
        .map(|_| HttpResponse::Ok().json(APIResponse::from("Integration enabled for account")))
        .map_err(NettuError::from)
}

#[derive(Debug, Clone)]
pub struct AddAccountIntegrationUseCase {
    pub account: Account,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub provider: IntegrationProvider,
}

impl From<AddAccountIntegrationUseCase> for AccountIntegration {
    fn from(e: AddAccountIntegrationUseCase) -> Self {
        Self {
            account_id: e.account.id,
            client_id: e.client_id,
            client_secret: e.client_secret,
            redirect_uri: e.redirect_uri,
            provider: e.provider,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum UseCaseError {
    StorageError,
    IntegrationAlreadyExists,
}

impl From<UseCaseError> for NettuError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::StorageError => Self::InternalError,
            UseCaseError::IntegrationAlreadyExists => {
                Self::Conflict("Account already has an integration for that provider".into())
            }
        }
    }
}

impl From<anyhow::Error> for UseCaseError {
    fn from(_: anyhow::Error) -> Self {
        UseCaseError::StorageError
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for AddAccountIntegrationUseCase {
    type Response = ();

    type Error = UseCaseError;

    const NAME: &'static str = "AddAccountIntegration";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Error> {
        // TODO: check if it is possible to validate client id or client secret

        let acc_integrations = ctx
            .repos
            .account_integrations
            .find(&self.account.id)
            .await?;
        if acc_integrations.iter().any(|i| i.provider == self.provider) {
            return Err(UseCaseError::IntegrationAlreadyExists);
        }

        ctx.repos
            .account_integrations
            .insert(&self.clone().into())
            .await?;

        Ok(())
    }
}
