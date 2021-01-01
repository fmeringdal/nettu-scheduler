use crate::{account::repos::IAccountRepo, shared::auth::{AccountAuthContext, protect_account_route}};
use crate::{api::Context, shared::auth::protect_route};
use crate::{account::domain::Account, shared::auth::AuthContext};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetAccountPubKeyReq {
    pub public_key_b64: Option<String>,
}

pub async fn set_account_pub_key_controller(
    http_req: web::HttpRequest,
    ctx: web::Data<Context>,
    body: web::Json<SetAccountPubKeyReq>
) -> HttpResponse {
    let account = match protect_account_route(
        &http_req,
        &AccountAuthContext {
            account_repo: Arc::clone(&ctx.repos.account_repo),
        },
    )
    .await
    {
        Ok(a) => a,
        Err(res) => return res,
    };

    let res = set_account_pub_key_usecase(
        SetAccountPubKeyUseCaseReq { account, public_key_b64: body.public_key_b64.clone()  },
        SetAccountPubKeyUseCaseCtx {
            account_repo: Arc::clone(&ctx.repos.account_repo),
        },
    )
    .await;

    match res {
        Ok(()) => HttpResponse::Ok().finish(),
        Err(e) => match e {
            UsecaseErrors::InvalidBase64Key => HttpResponse::UnprocessableEntity().body("Invalid base64 encoding of public key"),
            UsecaseErrors::StorageError => HttpResponse::InternalServerError().finish()
        },
    }
}

struct SetAccountPubKeyUseCaseReq {
    pub account: Account,
    pub public_key_b64: Option<String>,
}

struct SetAccountPubKeyUseCaseCtx {
    pub account_repo: Arc<dyn IAccountRepo>,
}

enum UsecaseErrors {
    InvalidBase64Key,
    StorageError
}

async fn set_account_pub_key_usecase(
    req: SetAccountPubKeyUseCaseReq,
    ctx: SetAccountPubKeyUseCaseCtx,
) -> Result<(), UsecaseErrors> {
    let SetAccountPubKeyUseCaseReq { mut account, public_key_b64 } = req;


    if account.set_public_key_b64(public_key_b64).is_err() {
        return Err(UsecaseErrors::InvalidBase64Key);
    }

    match ctx.account_repo.save(&account).await {
        Ok(_) => Ok(()),
        Err(_) => Err(UsecaseErrors::StorageError),
    }
}
