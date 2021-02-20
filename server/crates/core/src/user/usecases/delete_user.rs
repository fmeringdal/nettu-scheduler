use crate::account::domain::Account;
use crate::{
    context::Context,
    shared::usecase::{execute, UseCase},
    user::domain::User,
};

struct DeleteUserUseCase {
    account: Account,
    user_id: String,
}

struct UseCaseRes {
    pub user: User,
}

#[derive(Debug)]
enum UseCaseErrors {
    StorageError,
    UserNotFoundError,
}

#[async_trait::async_trait(?Send)]
impl UseCase for DeleteUserUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    type Context = Context;

    // TODOS:
    // - REMOVE ALL CALENDARS
    // - REMOVE ALL EVENTS
    // - REMOVE FROM ALL SERVICES
    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let user = match ctx.repos.user_repo.find(&self.user_id).await {
            Some(u) if u.account_id == self.account.id => {
                match ctx.repos.user_repo.delete(&self.user_id).await {
                    Some(u) => u,
                    None => return Err(UseCaseErrors::StorageError),
                }
            }
            _ => return Err(UseCaseErrors::UserNotFoundError),
        };

        Ok(UseCaseRes { user })
    }
}
