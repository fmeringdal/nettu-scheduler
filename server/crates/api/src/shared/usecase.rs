use super::auth::{Permission, Policy};
use std::fmt::Debug;

#[async_trait::async_trait(?Send)]
pub trait UseCase: Debug {
    type Response;
    type Errors;
    type Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors>;
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
    ctx: &U::Context,
) -> Result<U::Response, UseCaseErrorContainer<U::Errors>>
where
    U: PermissionBoundary,
    U::Errors: Debug,
{
    let permissions_boundary = usecase.permissions();
    if !policy.authorize(&permissions_boundary) {
        return Err(UseCaseErrorContainer::Unauthorized(format!(
            "Client is not permitted to perform some or all of these actions: {:?}",
            permissions_boundary
        )));
    }

    execute(usecase, ctx)
        .await
        .map_err(UseCaseErrorContainer::UseCase)
}

// TODO: Better †racing context
#[tracing::instrument(name = "Executing usecase", skip(usecase, ctx))]
pub async fn execute<U>(mut usecase: U, ctx: &U::Context) -> Result<U::Response, U::Errors>
where
    U: UseCase,
    U::Errors: Debug,
{
    let res = usecase.execute(ctx).await;

    if let Err(e) = &res {
        println!("Use case error: {:?}", e);
    }

    res
}
