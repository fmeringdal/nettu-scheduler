mod api;
mod calendar;
mod event;
mod shared;

use crate::api::Context;
use actix_web::{get, middleware, App, HttpServer};
use api::Repos;
use env_logger::Env;

#[get("/")]
async fn status() -> &'static str {
    println!("Got req");
    "Hello world!\r\n"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let repos = Repos::create().await.unwrap();

    HttpServer::new(move || {
        let ctx = Context {
            repos: repos.clone(),
        };

        App::new()
            .wrap(middleware::DefaultHeaders::new().header("X-Version", "0.2"))
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(status)
            .data(ctx)
            .configure(|cfg| event::rest::configure_routes(cfg))
            .configure(|cfg| calendar::rest::configure_routes(cfg))
    })
    .bind("0.0.0.0:5000")?
    .workers(4)
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http, test, App};

    #[actix_web::main]
    #[test]
    async fn test_status_ok() {
        let mut app = test::init_service(App::new().service(status)).await;
        let req = test::TestRequest::with_uri("/").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
    }
}
