use crate::context::Context;
use crate::{account::domain::Account, shared::usecase::UseCase};

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
impl UseCase for SetAccountPubKeyUseCase {
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
