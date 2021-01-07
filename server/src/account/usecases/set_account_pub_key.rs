use crate::api::Context;
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
) -> HttpResponse {
    let account = match protect_account_route(&http_req, &ctx).await {
        Ok(a) => a,
        Err(res) => return res,
    };

    let usecase = SetAccountPubKeyUseCase {
        account,
        public_key_b64: body.public_key_b64.clone(),
    };

    let res = execute(usecase, &ctx).await;

    match res {
        Ok(()) => HttpResponse::Ok().finish(),
        Err(e) => match e {
            UseCaseErrors::InvalidBase64Key => {
                HttpResponse::UnprocessableEntity().body("Invalid base64 encoding of public key")
            }
            UseCaseErrors::StorageError => HttpResponse::InternalServerError().finish(),
        },
    }
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
