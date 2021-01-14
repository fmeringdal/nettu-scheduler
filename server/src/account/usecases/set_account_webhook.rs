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
    pub webhook_url: Option<String>,
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
        webhook_url: body.webhook_url.clone(),
    };

    execute(usecase, &ctx)
        .await
        .map(|res| match res.webhook_key {
            Some(webhook_key) => HttpResponse::Ok().json(CreatedAccountWebhookRes { webhook_key }),
            None => HttpResponse::Ok().finish(),
        })
        .map_err(|e| match e {
            UseCaseErrors::InvalidURI => NettuError::BadClientData("Invalid URI provided".into()),
            UseCaseErrors::StorageError => NettuError::InternalError,
        })
}

struct SetAccountWebhookUseCase {
    pub account: Account,
    pub webhook_url: Option<String>,
}

struct SetAccountWebhookUseCaseResponse {
    pub webhook_key: Option<String>,
}

#[derive(Debug)]
enum UseCaseErrors {
    InvalidURI,
    StorageError,
}

#[async_trait::async_trait(?Send)]
impl Usecase for SetAccountWebhookUseCase {
    type Response = SetAccountWebhookUseCaseResponse;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let webhook_key = self
            .account
            .settings
            .set_webhook_url(self.webhook_url.clone());

        match ctx.repos.account_repo.save(&self.account).await {
            Ok(_) => Ok(SetAccountWebhookUseCaseResponse { webhook_key }),
            Err(_) => Err(UseCaseErrors::StorageError),
        }
    }
}
