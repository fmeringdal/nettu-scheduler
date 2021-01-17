use super::set_account_webhook::{SetAccountWebhookUseCase, UseCaseErrors};
use crate::api::{Context, NettuError};
use crate::shared::auth::protect_account_route;
use crate::shared::usecase::execute;
use actix_web::{web, HttpResponse};

pub async fn delete_account_webhook_controller(
    http_req: web::HttpRequest,
    ctx: web::Data<Context>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let usecase = SetAccountWebhookUseCase {
        account,
        webhook_url: None,
    };

    execute(usecase, &ctx)
        .await
        .map(|_| HttpResponse::Ok().body("Webhook deleted from account".to_string()))
        .map_err(|e| match e {
            UseCaseErrors::InvalidURI => NettuError::BadClientData("Invalid URI provided".into()),
            UseCaseErrors::WebhookUrlTaken => NettuError::BadClientData("URI is already in use by someone else".into()),
            UseCaseErrors::StorageError => NettuError::InternalError,
        })
}
