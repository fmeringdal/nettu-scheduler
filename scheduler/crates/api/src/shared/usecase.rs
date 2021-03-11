use super::auth::{Permission, Policy};
use futures::future::join_all;
use nettu_scheduler_infra::NettuContext;
use std::fmt::Debug;
use tracing::error;

/// Subscriber is a side effect to a `UseCase`
///
/// It is going to act upon the response of the execution
/// of the `UseCase` if the execution was a success.
#[async_trait::async_trait(?Send)]
pub trait Subscriber<U: UseCase> {
    async fn notify(&self, e: &U::Response, ctx: &NettuContext);
}

#[async_trait::async_trait(?Send)]
pub trait UseCase: Debug {
    type Response;
    type Errors;

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors>;

    fn subscribers() -> Vec<Box<dyn Subscriber<Self>>> {
        Default::default()
    }
}

/// Restrict what `Permission`s are needed for a `User`
/// to be able to execute the `UseCase`
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

    match &res {
        Ok(res) => {
            let subscribers = U::subscribers();
            let mut subscriber_promises = Vec::with_capacity(subscribers.len());
            for subscriber in &subscribers {
                subscriber_promises.push(subscriber.notify(res, ctx));
            }
            join_all(subscriber_promises).await;
        }
        Err(e) => {
            error!("Use case error: {:?}", e);
        }
    }

    res
}
