use super::usecases::create_service::create_service_controller;
use super::usecases::get_service::get_service_controller;
use super::usecases::remove_user_from_service::remove_user_from_service_controller;
use super::usecases::update_service_user::update_service_user_controller;
use super::usecases::{
    add_user_to_service::add_user_to_service_controller,
    get_service_bookingslots::get_service_bookingslots_controller,
};
use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/service", web::post().to(create_service_controller));
    cfg.route(
        "/service/{service_id}",
        web::get().to(get_service_controller),
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
