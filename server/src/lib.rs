pub mod account;
mod api;
pub mod calendar;
pub mod event;
pub mod service;
mod shared;
pub mod user;
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

    cfg.route("/", web::get().to(status));
}

pub use api::{Config, Context, Repos};
