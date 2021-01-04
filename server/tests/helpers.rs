use nettu_scheduler::{Config, Context, Repos, configure_server_app};
use actix_web::{App, dev::ServiceResponse, test};


fn setup_ctx() -> Context {
    let repos = Repos::create_inmemory();

    Context {
        repos,
        config: Config::new(),
    }
}

pub async fn perform(req: test::TestRequest) -> ServiceResponse {
    let ctx = setup_ctx();
    let mut app = test::init_service(
        App::new()
            .data(ctx)
            .configure(|cfg| configure_server_app(cfg)),
    )
    .await;
    test::call_service(&mut app, req.to_request()).await
}