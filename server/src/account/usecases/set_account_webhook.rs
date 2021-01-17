use crate::api::{Context, NettuError};
use crate::shared::auth::protect_account_route;
use crate::{
    account::domain::Account,
    shared::usecase::{execute, Usecase},
};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetAccountWebhookReq {
    pub webhook_url: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatedAccountWebhookRes {
    pub webhook_key: String,
}

pub async fn set_account_webhook_controller(
    http_req: web::HttpRequest,
    ctx: web::Data<Context>,
    body: web::Json<SetAccountWebhookReq>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let usecase = SetAccountWebhookUseCase {
        account,
        webhook_url: Some(body.webhook_url.clone()),
    };

    execute(usecase, &ctx)
        .await
        .map(|res| match res.webhook_key {
            Some(webhook_key) => HttpResponse::Ok().json(CreatedAccountWebhookRes { webhook_key }),
            None => HttpResponse::Ok().finish(),
        })
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

pub struct SetAccountWebhookUseCase {
    pub account: Account,
    pub webhook_url: Option<String>,
}

pub struct SetAccountWebhookUseCaseResponse {
    pub webhook_key: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum UseCaseErrors {
    InvalidURI(String),
    WebhookUrlTaken,
    StorageError,
}

#[async_trait::async_trait(?Send)]
impl Usecase for SetAccountWebhookUseCase {
    type Response = SetAccountWebhookUseCaseResponse;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
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

        let webhook_key = if let Some(settings) = &self.account.settings.webhook {
            Some(settings.key.clone())
        } else {
            None
        };

        match ctx.repos.account_repo.save(&self.account).await {
            Ok(_) => Ok(SetAccountWebhookUseCaseResponse { webhook_key }),
            Err(_) => Err(UseCaseErrors::StorageError),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::account::domain::{AccountSettings, AccountWebhookSettings};

    use super::*;

    #[actix_web::main]
    #[test]
    async fn it_rejects_invalid_webhook_url() {
        let ctx = Context::create_inmemory();
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
        let ctx = Context::create_inmemory();
        let bad_uris = vec!["https://google.com", "https://google.com/v1/webhook"];
        for bad_uri in bad_uris {
            let mut use_case = SetAccountWebhookUseCase {
                webhook_url: Some(bad_uri.to_string()),
                account: Default::default(),
            };
            let res = use_case.execute(&ctx).await;
            assert!(res.is_ok());
            assert!(res.unwrap().webhook_key.is_some());
        }
    }
}
