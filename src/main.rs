extern crate chrono;
extern crate chrono_tz;
extern crate mongodb;
extern crate rrule;

mod api;
mod calendar;
mod event;
mod shared;

use crate::api::Context;
use actix_web::{get, middleware, App, HttpServer};
use env_logger::Env;
use std::sync::Arc;

#[get("/")]
async fn status() -> &'static str {
    println!("Got req");
    "Hello world!\r\n"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let ctx = Arc::new(Context::new().await.unwrap());
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::DefaultHeaders::new().header("X-Version", "0.2"))
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(status)
            .configure(|cfg| event::rest::configure_routes(cfg, Arc::clone(&ctx)))
            .configure(|cfg| calendar::rest::configure_routes(cfg, Arc::clone(&ctx)))
    })
    .bind("0.0.0.0:5000")?
    .workers(4)
    .run()
    .await
}
