use nettu_scheduler_api::configure_server_app;
use nettu_scheduler_infra::setup_context;

use actix_web::{middleware, App, HttpServer};
use env_logger::Env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let context = setup_context().await;

    HttpServer::new(move || {
        let ctx = context.clone();

        App::new()
            .wrap(middleware::DefaultHeaders::new().header("X-Version", "0.2"))
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .data(ctx)
            .configure(|cfg| configure_server_app(cfg))
    })
    .bind("0.0.0.0:5000")?
    .workers(4)
    .run()
    .await
}
