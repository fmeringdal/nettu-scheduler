use std::fmt::Debug;

#[async_trait::async_trait(?Send)]
pub trait UseCase {
    type Response;
    type Errors;
    type Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors>;
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
