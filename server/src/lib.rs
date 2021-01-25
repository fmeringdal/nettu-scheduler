mod account;
mod api;
mod calendar;
mod event;
mod clock;
mod schedule;
mod service;
mod shared;
mod user;

use actix_web::web;

async fn status() -> &'static str {
    "Yo! We are up!\r\n"
}

pub fn configure_server_app(cfg: &mut web::ServiceConfig) {
    event::api::configure_routes(cfg);
    calendar::api::configure_routes(cfg);
    account::api::configure_routes(cfg);
    service::api::configure_routes(cfg);
    user::api::configure_routes(cfg);
    schedule::api::configure_routes(cfg);

    cfg.route("/", web::get().to(status));
}

pub use api::{Config, Context, RealSys, Repos};
pub use clock::start_clock;
