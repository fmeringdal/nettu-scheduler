mod add_user_to_service;
mod create_service;
mod delete_service;
mod get_service;
mod get_service_bookingslots;
mod get_services_by_meta;
mod remove_user_from_service;
mod update_service;
mod update_service_user;

use actix_web::web;
use add_user_to_service::add_user_to_service_controller;
use create_service::create_service_controller;
use delete_service::delete_service_controller;
use get_service::get_service_controller;
use get_service_bookingslots::get_service_bookingslots_controller;
use get_services_by_meta::get_services_by_meta_controller;
use remove_user_from_service::remove_user_from_service_controller;
use update_service::update_service_controller;
use update_service_user::update_service_user_controller;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/service", web::post().to(create_service_controller));
    cfg.route(
        "/service/{service_id}",
        web::get().to(get_service_controller),
    );
    cfg.route(
        "/service/{service_id}",
        web::put().to(update_service_controller),
    );
    cfg.route(
        "/service/{service_id}",
        web::delete().to(delete_service_controller),
    );
    cfg.route(
        "/service/meta",
        web::get().to(get_services_by_meta_controller),
    );
    cfg.route(
        "/service/{service_id}/users",
        web::post().to(add_user_to_service_controller),
    );
    cfg.route(
        "/service/{service_id}/users/{user_id}",
        web::delete().to(remove_user_from_service_controller),
    );
    cfg.route(
        "/service/{service_id}/users/{user_id}",
        web::put().to(update_service_user_controller),
    );
    cfg.route(
        "/service/{service_id}/booking",
        web::get().to(get_service_bookingslots_controller),
    );
}
