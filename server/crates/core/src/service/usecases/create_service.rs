use crate::{account::domain::Account, service::domain::Service};
use crate::{
    context::Context,
    shared::usecase::{execute, UseCase},
};
use serde::Serialize;

struct CreateServiceUseCase {
    account: Account,
}
struct UseCaseRes {
    pub service: Service,
}

#[derive(Debug)]
enum UseCaseErrors {
    StorageError,
}

#[async_trait::async_trait(?Send)]
impl UseCase for CreateServiceUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let service = Service::new(&self.account.id);
        let res = ctx.repos.service_repo.insert(&service).await;
        match res {
            Ok(_) => Ok(UseCaseRes { service }),
            Err(_) => Err(UseCaseErrors::StorageError),
        }
    }
}
