use crate::api::Context;
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
) -> HttpResponse {
    let account = match protect_account_route(&http_req, &ctx).await {
        Ok(a) => a,
        Err(res) => return res,
    };

    let res = GetAccountRes {
        id: account.id.clone(),
        public_key_b64: account.public_key_b64.clone(),
    };

    HttpResponse::Ok().json(res)
}
