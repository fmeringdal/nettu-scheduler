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
        .map_err(|e| match e {
            UseCaseErrors::InvalidURI(err) => {
                NettuError::BadClientData(format!("Invalid URI provided. Error message: {}", err))
            }
            UseCaseErrors::WebhookUrlTaken => {
                NettuError::BadClientData("URI is already in use by someone else".into())
            }
            UseCaseErrors::StorageError => NettuError::InternalError,
        })
}

#[derive(Debug)]
pub struct SetAccountWebhookUseCase {
    pub account: Account,
    pub webhook_url: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum UseCaseErrors {
    InvalidURI(String),
    WebhookUrlTaken,
    StorageError,
}

#[async_trait::async_trait(?Send)]
impl UseCase for SetAccountWebhookUseCase {
    type Response = Account;

    type Errors = UseCaseErrors;

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors> {
        let success = self
            .account
            .settings
            .set_webhook_url(self.webhook_url.clone());

        if !success {
            return Err(UseCaseErrors::InvalidURI(format!(
                "Malformed url or scheme is not https: {:?}",
                self.webhook_url
            )));
        }

        if let Some(url) = &self.webhook_url {
            if let Some(acc) = ctx.repos.account_repo.find_by_webhook_url(url).await {
                if acc.id != self.account.id {
                    return Err(UseCaseErrors::WebhookUrlTaken);
                }
            }
        }

        match ctx.repos.account_repo.save(&self.account).await {
            Ok(_) => Ok(self.account.clone()),
            Err(_) => Err(UseCaseErrors::StorageError),
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
        let bad_uris = vec![
            "1",
            "",
            "test.zzcom",
            "http://google.com",
            "test.com",
            "google.com",
        ];
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
                    UseCaseErrors::InvalidURI(format!(
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
