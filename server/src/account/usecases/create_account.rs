use crate::api::Context;
use crate::{
    account::domain::Account,
    shared::usecase::{perform, Usecase},
};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct BodyParams {
    code: String
}

pub async fn create_account_controller(ctx: web::Data<Context>, body: web::Json<BodyParams>) -> HttpResponse {
    if body.code != "FW4KbTC2loN1Ckr8KkIcwE3Av" {
        return HttpResponse::Unauthorized().finish()
    }
    
    let usecase = CreateAccountUseCase {};
    let res = perform(usecase, &ctx).await;

    match res {
        Ok(json) => HttpResponse::Created().json(json),
        Err(e) => match e {
            UseCaseErrors::StorageError => HttpResponse::InternalServerError().finish(),
        },
    }
}

struct CreateAccountUseCase {}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UseCaseResponse {
    pub account_id: String,
    pub secret_api_key: String,
}

#[derive(Debug)]
enum UseCaseErrors {
    StorageError,
}

#[async_trait::async_trait(?Send)]
impl Usecase for CreateAccountUseCase {
    type Response = UseCaseResponse;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn perform(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let account = Account::new();
        let res = ctx.repos.account_repo.insert(&account).await;
        match res {
            Ok(_) => Ok(UseCaseResponse {
                account_id: account.id.clone(),
                secret_api_key: account.secret_api_key.clone(),
            }),
            Err(_) => Err(UseCaseErrors::StorageError),
        }
    }
}
