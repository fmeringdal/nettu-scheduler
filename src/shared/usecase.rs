use async_trait::async_trait;

#[async_trait(?Send)]
pub trait UseCase<IReq, IRes> {
    async fn execute(&self, req: IReq) -> IRes;
}