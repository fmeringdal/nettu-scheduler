use crate::{
    account::domain::Account,
    context::Context,
    service::domain::ServiceResource,
    shared::usecase::{execute, UseCase},
};

struct RemoveUserFromServiceUseCase {
    pub account: Account,
    pub service_id: String,
    pub user_id: String,
}

struct UseCaseRes {
    pub resource: ServiceResource,
}

#[derive(Debug)]
enum UseCaseErrors {
    StorageError,
    ServiceNotFoundError,
    UserNotFoundError,
}

#[async_trait::async_trait(?Send)]
impl UseCase for RemoveUserFromServiceUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let mut service = match ctx.repos.service_repo.find(&self.service_id).await {
            Some(service) if service.account_id == self.account.id => service,
            _ => return Err(UseCaseErrors::ServiceNotFoundError),
        };

        match service.remove_user(&self.user_id) {
            Some(resource) => match ctx.repos.service_repo.save(&service).await {
                Ok(_) => Ok(UseCaseRes { resource }),
                Err(_) => Err(UseCaseErrors::StorageError),
            },
            None => Err(UseCaseErrors::UserNotFoundError),
        }
    }
}
