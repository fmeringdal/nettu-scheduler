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
        .map_err(|e| match e {
            UseCaseErrors::StorageError => NettuError::InternalError,
            UseCaseErrors::IntegrationAlreadyExists => {
                NettuError::Conflict("Account already has an integration for that provider".into())
            }
        })
}

#[derive(Debug)]
pub struct AddAccountIntegrationUseCase {
    pub account: Account,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub provider: IntegrationProvider,
}

#[derive(Debug, PartialEq)]
pub enum UseCaseErrors {
    StorageError,
    IntegrationAlreadyExists,
}

#[async_trait::async_trait(?Send)]
impl UseCase for AddAccountIntegrationUseCase {
    type Response = ();

    type Errors = UseCaseErrors;

    const NAME: &'static str = "AddAccountIntegration";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors> {
        // TODO: check if it is possible to validate client id or client secret

        let acc_integrations = ctx
            .repos
            .account_integrations
            .find(&self.account.id)
            .await
            .map_err(|_| UseCaseErrors::StorageError)?;
        if acc_integrations.iter().any(|i| i.provider == self.provider) {
            return Err(UseCaseErrors::IntegrationAlreadyExists);
        }

        let integration = AccountIntegration {
            account_id: self.account.id.clone(),
            client_id: self.client_id.clone(),
            client_secret: self.client_secret.clone(),
            redirect_uri: self.redirect_uri.clone(),
            provider: self.provider.clone(),
        };

        if ctx
            .repos
            .account_integrations
            .insert(&integration)
            .await
            .is_err()
        {
            return Err(UseCaseErrors::StorageError);
        }

        Ok(())
    }
}
