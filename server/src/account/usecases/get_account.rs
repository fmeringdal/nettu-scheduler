use crate::api::{Context, NettuError};
use crate::shared::auth::protect_account_route;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct GetAccountReq {
    pub public_key_b64: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GetAccountRes {
    pub id: String,
    pub public_key_b64: Option<String>,
}

pub async fn get_account_controller(
    http_req: web::HttpRequest,
    ctx: web::Data<Context>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let res = GetAccountRes {
        id: account.id.clone(),
        public_key_b64: account.public_key_b64,
    };

    Ok(HttpResponse::Ok().json(res))
}
