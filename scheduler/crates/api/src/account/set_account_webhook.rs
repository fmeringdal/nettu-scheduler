use crate::shared::usecase::{execute, UseCase};
use crate::{error::NettuError, shared::auth::protect_account_route};
use actix_web::{web, HttpResponse};
use nettu_scheduler_api_structs::set_account_webhook::{APIResponse, RequestBody};
use nettu_scheduler_domain::Account;
use nettu_scheduler_infra::NettuContext;

pub async fn set_account_webhook_controller(
    http_req: web::HttpRequest,
    ctx: web::Data<NettuContext>,
    body: web::Json<RequestBody>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let usecase = SetAccountWebhookUseCase {
        account,
        webhook_url: Some(body.webhook_url.clone()),
    };

    execute(usecase, &ctx)
        .await
        .map(|account| HttpResponse::Ok().json(APIResponse::new(account)))
        .map_err(NettuError::from)
}

#[derive(Debug)]
pub struct SetAccountWebhookUseCase {
    pub account: Account,
    pub webhook_url: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum UseCaseError {
    InvalidURI(String),
    StorageError,
}

impl From<UseCaseError> for NettuError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::InvalidURI(err) => {
                Self::BadClientData(format!("Invalid URI provided. Error message: {}", err))
            }
            UseCaseError::StorageError => Self::InternalError,
        }
    }
}
#[async_trait::async_trait(?Send)]
impl UseCase for SetAccountWebhookUseCase {
    type Response = Account;

    type Error = UseCaseError;

    const NAME: &'static str = "SetAccountWebhook";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Error> {
        let success = self
            .account
            .settings
            .set_webhook_url(self.webhook_url.clone());

        if !success {
            return Err(UseCaseError::InvalidURI(format!(
                "Malformed url or scheme is not https: {:?}",
                self.webhook_url
            )));
        }

        match ctx.repos.accounts.save(&self.account).await {
            Ok(_) => Ok(self.account.clone()),
            Err(_) => Err(UseCaseError::StorageError),
        }
    }
}

#[cfg(test)]
mod tests {

    use nettu_scheduler_infra::setup_context;

    use super::*;

    #[actix_web::main]
    #[test]
    async fn it_rejects_invalid_webhook_url() {
        let ctx = setup_context().await;
        let bad_uris = vec!["1", "", "test.zzcom", "test.com", "google.com"];
        for bad_uri in bad_uris {
            let mut use_case = SetAccountWebhookUseCase {
                webhook_url: Some(bad_uri.to_string()),
                account: Default::default(),
            };
            let res = use_case.execute(&ctx).await;
            assert!(res.is_err());
            if let Err(err) = res {
                assert_eq!(
                    err,
                    UseCaseError::InvalidURI(format!(
                        "Malformed url or scheme is not https: {:?}",
                        Some(bad_uri)
                    ))
                );
            }
        }
    }

    #[actix_web::main]
    #[test]
    async fn it_accepts_valid_webhook_url() {
        let ctx = setup_context().await;

        let valid_uris = vec!["https://google.com", "https://google.com/v1/webhook"];
        for valid_uri in valid_uris {
            let mut use_case = SetAccountWebhookUseCase {
                webhook_url: Some(valid_uri.to_string()),
                account: Default::default(),
            };
            let res = use_case.execute(&ctx).await;
            assert!(res.is_ok());
        }
    }
}
