use super::usecases::create_event::create_event_controller;
use super::usecases::create_event_exception::create_event_exception_controller;
use super::usecases::delete_event::delete_event_controller;
use super::usecases::get_event::get_event_controller;
use super::usecases::get_event_instances::get_event_instances_controller;
use super::usecases::update_event::update_event_controller;
use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    // Hookup Routes to usecases
    cfg.route("/events", web::post().to(create_event_controller));
    cfg.route("/events/{event_id}", web::get().to(get_event_controller));
    cfg.route("/events/{event_id}", web::put().to(update_event_controller));
    cfg.route(
        "/events/{event_id}/exception",
        web::post().to(create_event_exception_controller),
    );
    cfg.route(
        "/events/{event_id}",
        web::delete().to(delete_event_controller),
    );
    cfg.route(
        "/events/{event_id}/instances",
        web::get().to(get_event_instances_controller),
    );
}
