use crate::shared::usecase::{execute, UseCase};
use crate::{error::NettuError, shared::auth::protect_account_route};
use actix_web::{web, HttpResponse};
use nettu_scheduler_api_structs::set_account_pub_key::{APIResponse, RequestBody};
use nettu_scheduler_domain::{Account, PEMKey};
use nettu_scheduler_infra::NettuContext;

pub async fn set_account_pub_key_controller(
    http_req: web::HttpRequest,
    ctx: web::Data<NettuContext>,
    body: web::Json<RequestBody>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let usecase = SetAccountPubKeyUseCase {
        account,
        public_jwt_key: body.public_jwt_key.clone(),
    };

    execute(usecase, &ctx)
        .await
        .map(|account| HttpResponse::Ok().json(APIResponse::new(account)))
        .map_err(NettuError::from)
}

#[derive(Debug)]
struct SetAccountPubKeyUseCase {
    pub account: Account,
    pub public_jwt_key: Option<String>,
}

#[derive(Debug)]
enum UseCaseError {
    InvalidPemKey,
    StorageError,
}

impl From<UseCaseError> for NettuError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::InvalidPemKey => {
                Self::BadClientData("Malformed public pem key provided".into())
            }
            UseCaseError::StorageError => Self::InternalError,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for SetAccountPubKeyUseCase {
    type Response = Account;

    type Error = UseCaseError;

    const NAME: &'static str = "SetAccountPublicKey";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Error> {
        let key = if let Some(key) = &self.public_jwt_key {
            match PEMKey::new(key.clone()) {
                Ok(key) => Some(key),
                Err(_) => return Err(UseCaseError::InvalidPemKey),
            }
        } else {
            None
        };
        self.account.set_public_jwt_key(key);

        match ctx.repos.accounts.save(&self.account).await {
            Ok(_) => Ok(self.account.clone()),
            Err(_) => Err(UseCaseError::StorageError),
        }
    }
}
