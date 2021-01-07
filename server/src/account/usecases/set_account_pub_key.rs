use crate::api::{Context, NettuError};
use crate::shared::auth::protect_account_route;
use crate::{
    account::domain::Account,
    shared::usecase::{execute, Usecase},
};
use actix_web::{web, HttpResponse};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetAccountPubKeyReq {
    pub public_key_b64: Option<String>,
}

pub async fn set_account_pub_key_controller(
    http_req: web::HttpRequest,
    ctx: web::Data<Context>,
    body: web::Json<SetAccountPubKeyReq>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let usecase = SetAccountPubKeyUseCase {
        account,
        public_key_b64: body.public_key_b64.clone(),
    };

    execute(usecase, &ctx)
        .await
        .map(|_| HttpResponse::Ok().finish())
        .map_err(|e| match e {
            UseCaseErrors::InvalidBase64Key => {
                NettuError::BadClientData(format!("Invalid base64 encoding of public key"))
            }
            UseCaseErrors::StorageError => NettuError::InternalError,
        })
}

struct SetAccountPubKeyUseCase {
    pub account: Account,
    pub public_key_b64: Option<String>,
}

#[derive(Debug)]
enum UseCaseErrors {
    InvalidBase64Key,
    StorageError,
}

#[async_trait::async_trait(?Send)]
impl Usecase for SetAccountPubKeyUseCase {
    type Response = ();

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        if self
            .account
            .set_public_key_b64(self.public_key_b64.clone())
            .is_err()
        {
            return Err(UseCaseErrors::InvalidBase64Key);
        }

        match ctx.repos.account_repo.save(&self.account).await {
            Ok(_) => Ok(()),
            Err(_) => Err(UseCaseErrors::StorageError),
        }
    }
}
