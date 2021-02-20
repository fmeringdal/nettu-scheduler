use std::fmt::Debug;

use super::auth::{Permission, Policy};

#[async_trait::async_trait(?Send)]
pub trait UseCase {
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
    Unauthorized,
    UseCase(T),
}

pub async fn execute_with_policy<U>(
    usecase: U,
    policy: &Policy,
    ctx: &U::Context,
) -> Result<U::Response, UseCaseErrorContainer<U::Errors>>
where
    U: PermissionBoundary,
    U::Errors: Debug,
{
    if !policy.authorize(&usecase.permissions()) {
        return Err(UseCaseErrorContainer::Unauthorized);
    }

    execute(usecase, ctx)
        .await
        .map_err(|e| UseCaseErrorContainer::UseCase(e))
}

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
