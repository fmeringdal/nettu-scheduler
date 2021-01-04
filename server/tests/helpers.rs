use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse},
    test, App, Error,
};
use nettu_scheduler::{configure_server_app, Config, Context, Repos};

fn setup_ctx() -> Context {
    let repos = Repos::create_inmemory();

    Context {
        repos,
        config: Config::new(),
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
