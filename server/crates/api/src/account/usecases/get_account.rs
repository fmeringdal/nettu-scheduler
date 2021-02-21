use crate::shared::auth::protect_account_route;
use crate::{account::dtos::AccountDTO, error::NettuError};
use actix_web::{web, HttpResponse};
use nettu_scheduler_infra::NettuContext;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GetAccountReq {
    pub public_key_b64: Option<String>,
}

pub async fn get_account_controller(
    http_req: web::HttpRequest,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    Ok(HttpResponse::Ok().json(AccountDTO::new(&account)))
}
