mod create_event;
mod delete_event;
mod get_event;
mod get_event_instances;
mod get_events_by_meta;
pub mod get_upcoming_reminders;
pub mod sync_event_reminders;
mod update_event;

use actix_web::web;
use create_event::{create_event_admin_controller, create_event_controller};
use delete_event::{delete_event_admin_controller, delete_event_controller};
use get_event::{get_event_admin_controller, get_event_controller};
use get_event_instances::{get_event_instances_admin_controller, get_event_instances_controller};
use get_events_by_meta::get_events_by_meta_controller;
use update_event::{update_event_admin_controller, update_event_controller};

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/events", web::post().to(create_event_controller));
    cfg.route(
        "/user/{user_id}/events",
        web::post().to(create_event_admin_controller),
    );

    cfg.route("/events/{event_id}", web::get().to(get_event_controller));
    cfg.route(
        "/user/events/{event_id}",
        web::get().to(get_event_admin_controller),
    );
    cfg.route("/events/meta", web::get().to(get_events_by_meta_controller));

    cfg.route(
        "/events/{event_id}",
        web::delete().to(delete_event_controller),
    );
    cfg.route(
        "/user/events/{event_id}",
        web::delete().to(delete_event_admin_controller),
    );

    cfg.route("/events/{event_id}", web::put().to(update_event_controller));
    cfg.route(
        "/user/events/{event_id}",
        web::put().to(update_event_admin_controller),
    );

    cfg.route(
        "/events/{event_id}/instances",
        web::get().to(get_event_instances_controller),
    );
    cfg.route(
        "/user/events/{event_id}/instances",
        web::get().to(get_event_instances_admin_controller),
    );
}
