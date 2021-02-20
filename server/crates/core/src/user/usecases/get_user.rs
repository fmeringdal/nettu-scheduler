use crate::{
    account::domain::Account,
    shared::usecase::{execute, UseCase},
};
use crate::{context::Context, user::domain::User};

struct GetUserUseCase {
    account: Account,
    user_id: String,
}

struct UseCaseRes {
    pub user: User,
}

#[derive(Debug)]
enum UseCaseErrors {
    UserNotFoundError,
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetUserUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let user = match ctx.repos.user_repo.find(&self.user_id).await {
            Some(u) if u.account_id == self.account.id => u,
            _ => return Err(UseCaseErrors::UserNotFoundError),
        };

        Ok(UseCaseRes { user })
    }
}
