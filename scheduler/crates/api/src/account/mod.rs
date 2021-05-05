mod create_account;
mod delete_account_webhook;
mod get_account;
mod set_account_google_integration;
mod set_account_pub_key;
mod set_account_webhook;

use actix_web::web;
use create_account::create_account_controller;
use delete_account_webhook::delete_account_webhook_controller;
use get_account::get_account_controller;
use set_account_google_integration::set_account_google_integration_controller;
use set_account_pub_key::set_account_pub_key_controller;
use set_account_webhook::set_account_webhook_controller;

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
    cfg.route(
        "/account/webhook",
        web::delete().to(delete_account_webhook_controller),
    );
    cfg.route(
        "/account/integration/google",
        web::put().to(set_account_google_integration_controller),
    );
}
