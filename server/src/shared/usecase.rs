use std::fmt::Debug;

// how to get context ??
#[async_trait::async_trait(?Send)]
pub trait Usecase {
    type Response;
    type Errors;
    type Context;

    async fn perform(&self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors>;
}

pub async fn perform<U>(usecase: U, ctx: &U::Context) -> Result<U::Response, U::Errors>
where
    U: Usecase,
    U::Errors: Debug,
{
    let res = usecase.perform(ctx).await;

    if let Err(e) = &res {
        println!("Usecase error: {:?}", e);
    }

    res
}
