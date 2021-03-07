use nettu_scheduler_infra::NettuContext;

use super::auth::{Permission, Policy};
use std::fmt::Debug;
use tracing::error;

#[async_trait::async_trait(?Send)]
pub trait UseCase: Debug {
    type Response;
    type Errors;

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors>;
}

pub trait PermissionBoundary: UseCase {
    fn permissions(&self) -> Vec<Permission>;
}

#[derive(Debug)]
pub enum UseCaseErrorContainer<T: Debug> {
    Unauthorized(String),
    UseCase(T),
}

// TODO: How to able better tracing context ? Usecase: Debug
#[tracing::instrument(name = "Executing usecase with policy", skip(usecase, ctx))]
pub async fn execute_with_policy<U>(
    usecase: U,
    policy: &Policy,
    ctx: &NettuContext,
) -> Result<U::Response, UseCaseErrorContainer<U::Errors>>
where
    U: PermissionBoundary,
    U::Errors: Debug,
{
    let required_permissions = usecase.permissions();
    if !policy.authorize(&required_permissions) {
        return Err(UseCaseErrorContainer::Unauthorized(format!(
            "Client is not permitted to perform some or all of these actions: {:?}",
            required_permissions
        )));
    }

    execute(usecase, ctx)
        .await
        .map_err(UseCaseErrorContainer::UseCase)
}

// TODO: Better †racing context
#[tracing::instrument(name = "Executing usecase", skip(usecase, ctx))]
pub async fn execute<U>(mut usecase: U, ctx: &NettuContext) -> Result<U::Response, U::Errors>
where
    U: UseCase,
    U::Errors: Debug,
{
    let res = usecase.execute(ctx).await;

    if let Err(e) = &res {
        error!("Use case error: {:?}", e);
    }

    res
}
