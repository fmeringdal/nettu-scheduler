use crate::{
    account::dtos::AccountDTO,
    shared::usecase::{execute, UseCase},
};
use actix_web::{web, HttpResponse};
use nettu_scheduler_core::Account;
use nettu_scheduler_infra::NettuContext;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct BodyParams {
    code: String,
}

pub async fn create_account_controller(
    ctx: web::Data<NettuContext>,
    body: web::Json<BodyParams>,
) -> HttpResponse {
    if body.code != ctx.config.create_account_secret_code {
        return HttpResponse::Unauthorized().finish();
    }

    let usecase = CreateAccountUseCase {};
    let res = execute(usecase, &ctx).await;

    match res {
        Ok(account) => HttpResponse::Created().json(AccountDTO::new(&account)),
        Err(e) => match e {
            UseCaseErrors::StorageError => HttpResponse::InternalServerError().finish(),
        },
    }
}

struct CreateAccountUseCase {}

#[derive(Debug)]
enum UseCaseErrors {
    StorageError,
}

#[async_trait::async_trait(?Send)]
impl UseCase for CreateAccountUseCase {
    type Response = Account;

    type Errors = UseCaseErrors;

    type Context = NettuContext;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let account = Account::new();
        let res = ctx.repos.account_repo.insert(&account).await;

        res.map(|_| account)
            .map_err(|_| UseCaseErrors::StorageError)
    }
}
