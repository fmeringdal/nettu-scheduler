use crate::account::repos::IAccountRepo;
use crate::{account::domain::Account};
use crate::{api::Context};
use actix_web::{web, HttpResponse};

use serde::{Serialize};
use std::sync::Arc;

pub async fn create_account_controller(ctx: web::Data<Context>) -> HttpResponse {
    let res = create_account_usecase(
        CreateAccountReq {},
        CreateAccountUseCaseCtx {
            account_repo: Arc::clone(&ctx.repos.account_repo),
        },
    )
    .await;

    match res {
        Ok(json) => HttpResponse::Created().json(json),
        Err(e) => match e {
            UsecaseErrors::StorageError => HttpResponse::InternalServerError().finish(),
        },
    }
}

struct CreateAccountReq {}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateAccountRes {
    pub account_id: String,
    pub secret_api_key: String,
}

enum UsecaseErrors {
    StorageError,
}

struct CreateAccountUseCaseCtx {
    pub account_repo: Arc<dyn IAccountRepo>,
}

async fn create_account_usecase(
    _req: CreateAccountReq,
    ctx: CreateAccountUseCaseCtx,
) -> Result<CreateAccountRes, UsecaseErrors> {
    let account = Account::new();
    let res = ctx.account_repo.insert(&account).await;
    match res {
        Ok(_) => Ok(CreateAccountRes {
            account_id: account.id.clone(),
            secret_api_key: account.secret_api_key.clone(),
        }),
        Err(_) => Err(UsecaseErrors::StorageError),
    }
}
