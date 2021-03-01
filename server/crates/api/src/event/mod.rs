mod create_event;
mod create_event_exception;
mod delete_event;
mod get_event;
mod get_event_instances;
pub mod get_upcoming_reminders;
pub mod sync_event_reminders;
mod update_event;

use actix_web::web;
use create_event::create_event_controller;
use create_event_exception::create_event_exception_controller;
use delete_event::delete_event_controller;
use get_event::get_event_controller;
use get_event_instances::get_event_instances_controller;
use update_event::update_event_controller;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
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
