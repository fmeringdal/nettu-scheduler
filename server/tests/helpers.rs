use actix_web::{dev::ServiceResponse, test, App};
use nettu_scheduler_api::configure_server_app;
use nettu_scheduler_infra::{Config, Context, RealSys, Repos};
use std::sync::Arc;

fn setup_ctx() -> Context {
    let repos = Repos::create_inmemory();

    Context {
        repos,
        config: Config::new(),
        sys: Arc::new(RealSys {}),
    }
}

pub async fn perform(req: test::TestRequest) -> ServiceResponse {
    let ctx = setup_ctx();
    let mut serivce = test::init_service(
        App::new()
            .data(ctx)
            .configure(|cfg| configure_server_app(cfg)),
    )
    .await;
    test::call_service(&mut serivce, req.to_request()).await
}
