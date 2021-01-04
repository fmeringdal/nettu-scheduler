extern crate nettu_scheduler;

use actix_web::{http, test, App};
use nettu_scheduler::{configure_server_app, Config, Context, Repos};

fn setup_ctx() -> Context {
    let repos = Repos::create_inmemory();

    Context {
        repos,
        config: Config::new(),
    }
}

#[actix_web::main]
#[test]
async fn test_status_ok() {
    let ctx = setup_ctx();
    let mut app = test::init_service(
        App::new()
            .data(ctx)
            .configure(|cfg| configure_server_app(cfg)),
    )
    .await;
    let req = test::TestRequest::with_uri("/").to_request();
    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);
}
