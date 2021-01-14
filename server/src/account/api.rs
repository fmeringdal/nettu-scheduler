use super::usecases::set_account_pub_key::set_account_pub_key_controller;
use super::usecases::set_account_webhook::set_account_webhook_controller;
use super::usecases::{
    create_account::create_account_controller, get_account::get_account_controller,
};
use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/account", web::post().to(create_account_controller));
    cfg.route("/account", web::get().to(get_account_controller));
    cfg.route(
        "/account/pubkey",
        web::put().to(set_account_pub_key_controller),
    );
    cfg.route(
        "/account/webhook",
        web::put().to(set_account_webhook_controller),
    );
}
