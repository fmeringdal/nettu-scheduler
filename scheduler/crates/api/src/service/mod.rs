mod add_busy_calendar;
mod add_user_to_service;
mod create_service;
mod create_service_event_intend;
mod delete_service;
mod get_service;
mod get_service_bookingslots;
mod get_services_by_meta;
mod remove_busy_calendar;
mod remove_service_event_intend;
mod remove_user_from_service;
mod update_service;
mod update_service_user;

use actix_web::web;
use add_busy_calendar::add_busy_calendar_controller;
use add_user_to_service::add_user_to_service_controller;
use create_service::create_service_controller;
use create_service_event_intend::create_service_event_intend_controller;
use delete_service::delete_service_controller;
use get_service::get_service_controller;
use get_service_bookingslots::get_service_bookingslots_controller;
use get_services_by_meta::get_services_by_meta_controller;
use remove_busy_calendar::remove_busy_calendar_controller;
use remove_service_event_intend::remove_service_event_intend_controller;
use remove_user_from_service::remove_user_from_service_controller;
use update_service::update_service_controller;
use update_service_user::update_service_user_controller;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/service", web::post().to(create_service_controller));
    cfg.route(
        "/service/meta",
        web::get().to(get_services_by_meta_controller),
    );
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
        "/service/{service_id}/users/{user_id}/busy",
        web::put().to(add_busy_calendar_controller),
    );
    cfg.route(
        "/service/{service_id}/users/{user_id}/busy",
        web::delete().to(remove_busy_calendar_controller),
    );
    cfg.route(
        "/service/{service_id}/booking",
        web::get().to(get_service_bookingslots_controller),
    );
    cfg.route(
        "/service/{service_id}/booking-intend",
        web::post().to(create_service_event_intend_controller),
    );
    cfg.route(
        "/service/{service_id}/booking-intend",
        web::delete().to(remove_service_event_intend_controller),
    );
}
