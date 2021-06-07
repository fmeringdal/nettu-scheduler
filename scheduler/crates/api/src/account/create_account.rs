use crate::{
    error::NettuError,
    shared::usecase::{execute, UseCase},
};
use actix_web::{web, HttpResponse};
use nettu_scheduler_api_structs::create_account::{APIResponse, RequestBody};
use nettu_scheduler_domain::Account;
use nettu_scheduler_infra::NettuContext;

pub async fn create_account_controller(
    ctx: web::Data<NettuContext>,
    body: web::Json<RequestBody>,
) -> Result<HttpResponse, NettuError> {
    let usecase = CreateAccountUseCase { code: body.0.code };
    execute(usecase, &ctx)
        .await
        .map(|account| HttpResponse::Created().json(APIResponse::new(account)))
        .map_err(|e| match e {
            UseCaseErrors::InvalidCreateAccountCode => {
                NettuError::Unauthorized("Invalid code provided".into())
            }
            UseCaseErrors::StorageError => NettuError::InternalError,
        })
}

#[derive(Debug)]
struct CreateAccountUseCase {
    code: String,
}

#[derive(Debug)]
enum UseCaseErrors {
    StorageError,
    InvalidCreateAccountCode,
}

#[async_trait::async_trait(?Send)]
impl UseCase for CreateAccountUseCase {
    type Response = Account;

    type Errors = UseCaseErrors;

    const NAME: &'static str = "CreateAccount";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors> {
        if self.code != ctx.config.create_account_secret_code {
            return Err(UseCaseErrors::InvalidCreateAccountCode);
        }
        let account = Account::new();
        let res = ctx.repos.accounts.insert(&account).await;

        res.map(|_| account)
            .map_err(|_| UseCaseErrors::StorageError)
    }
}
