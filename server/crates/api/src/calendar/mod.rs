use actix_web::web;

mod create_calendar;
mod delete_calendar;
mod get_calendar;
mod get_calendar_events;
mod get_freebusy;
mod update_calendar_settings;

use create_calendar::{create_calendar_admin_controller, create_calendar_controller};
use delete_calendar::delete_calendar_controller;
use get_calendar::get_calendar_controller;
use get_calendar_events::get_calendar_events_controller;
use get_freebusy::get_freebusy_controller;
use update_calendar_settings::update_calendar_settings_controller;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    // create calendar jwt token controller
    cfg.route("/calendar", web::post().to(create_calendar_controller));
    // create calendar account admin controller
    cfg.route(
        "/user/{user_id}/calendar",
        web::post().to(create_calendar_admin_controller),
    );

    cfg.route(
        "/calendar/{calendar_id}",
        web::get().to(get_calendar_controller),
    );
    cfg.route(
        "/calendar/{calendar_id}",
        web::delete().to(delete_calendar_controller),
    );

    cfg.route(
        "/calendar/{calendar_id}/settings",
        web::put().to(update_calendar_settings_controller),
    );
    cfg.route(
        "/calendar/{calendar_id}/events",
        web::get().to(get_calendar_events_controller),
    );
    cfg.route(
        "/user/{external_user_id}/freebusy",
        web::get().to(get_freebusy_controller),
    );
}
