extern crate chrono;
extern crate chrono_tz;
extern crate mongodb;
extern crate rrule;

mod api;
mod calendar;
mod event;
mod shared;

use crate::api::Context;
use crate::event::rest::configure_routes;
use actix_web::{get, middleware, web, App, HttpServer};
use env_logger::Env;
use std::sync::Arc;

#[get("/")]
async fn status() -> &'static str {
    println!("Got req");
    "Hello world!\r\n"
}

#[get("/events/{eventId}")]
async fn get_events(data: web::Data<Arc<Context>>, event_id: web::Path<String>) -> &'static str {
    let res = data.repos.event_repo.find(&event_id).await;
    if let Some(e) = res {
        println!("All of that: {:?}", e.expand());
    }
    "Hello, yes we are up and running!\r\n"
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
            .service(get_events)
            .configure(|cfg| configure_routes(cfg, Arc::clone(&ctx)))
    })
    .bind("0.0.0.0:5000")?
    .workers(4)
    .run()
    .await
}
