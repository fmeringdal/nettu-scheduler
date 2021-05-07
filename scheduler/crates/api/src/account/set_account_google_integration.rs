use crate::shared::usecase::{execute, UseCase};
use crate::{error::NettuError, shared::auth::protect_account_route};
use actix_web::{web, HttpResponse};
use nettu_scheduler_api_structs::set_account_google_integration::{APIResponse, RequestBody};
use nettu_scheduler_domain::{Account, AccountGoogleIntegration};
use nettu_scheduler_infra::NettuContext;

pub async fn set_account_google_integration_controller(
    http_req: web::HttpRequest,
    body: web::Json<RequestBody>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let body = body.0;
    let usecase = SetAccountGoogleIntegrationUseCase {
        account,
        client_id: body.client_id,
        client_secret: body.client_secret,
        redirect_uri: body.redirect_uri,
    };

    execute(usecase, &ctx)
        .await
        .map(|account| HttpResponse::Ok().json(APIResponse::new(account)))
        .map_err(|e| match e {
            UseCaseErrors::StorageError => NettuError::InternalError,
        })
}

#[derive(Debug)]
pub struct SetAccountGoogleIntegrationUseCase {
    pub account: Account,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Debug, PartialEq)]
pub enum UseCaseErrors {
    StorageError,
}

#[async_trait::async_trait(?Send)]
impl UseCase for SetAccountGoogleIntegrationUseCase {
    type Response = Account;

    type Errors = UseCaseErrors;

    const NAME: &'static str = "SetAccountGoogleIntegration";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors> {
        // TODO: check if it is possible to validate client id or client secret

        // If client id or client secret changes, then all user google integrations
        // with this account becomes invalid and needs to be revoked
        let need_to_revoke_user_integrations = match &self.account.settings.google {
            Some(g_settings) => {
                g_settings.client_id != self.client_id
                    || g_settings.client_secret != self.client_secret
            }
            _ => false,
        };
        if need_to_revoke_user_integrations {
            if ctx
                .repos
                .user_repo
                .revoke_google_integration(&self.account.id)
                .await
                .is_err()
            {
                return Err(UseCaseErrors::StorageError);
            }
        }

        self.account.settings.google = Some(AccountGoogleIntegration {
            client_id: self.client_id.clone(),
            client_secret: self.client_secret.clone(),
            redirect_uri: self.redirect_uri.clone(),
        });

        ctx.repos
            .account_repo
            .save(&self.account)
            .await
            .map(|_| self.account.clone())
            .map_err(|_| UseCaseErrors::StorageError)
    }
}
