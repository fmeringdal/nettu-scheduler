use crate::context::Context;
use crate::{account::domain::Account, shared::usecase::UseCase};
use serde::Serialize;

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
impl UseCase for CreateAccountUseCase {
    type Response = UseCaseResponse;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
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
