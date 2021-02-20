use crate::{
    account::domain::Account, context::Context, service::domain::Service, shared::usecase::UseCase,
};

struct GetServiceUseCase {
    account: Account,
    service_id: String,
}

struct UseCaseRes {
    pub service: Service,
}

#[derive(Debug)]
enum UseCaseErrors {
    NotFoundError,
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetServiceUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let res = ctx.repos.service_repo.find(&self.service_id).await;
        match res {
            Some(service) if service.account_id == self.account.id => Ok(UseCaseRes { service }),
            _ => Err(UseCaseErrors::NotFoundError),
        }
    }
}
