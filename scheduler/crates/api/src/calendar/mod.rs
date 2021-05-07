use actix_web::web;

mod create_calendar;
mod delete_calendar;
mod get_calendar;
mod get_calendar_events;
mod get_calendars_by_meta;
mod get_google_calendars;
mod update_calendar;

use create_calendar::{
    create_calendar_admin_controller, create_calendar_controller, update_synced_calendars,
};
use delete_calendar::{delete_calendar_admin_controller, delete_calendar_controller};
use get_calendar::{get_calendar_admin_controller, get_calendar_controller};
use get_calendar_events::{get_calendar_events_admin_controller, get_calendar_events_controller};
use get_calendars_by_meta::get_calendars_by_meta_controller;
use get_google_calendars::{
    get_google_calendars_admin_controller, get_google_calendars_controller,
};
use update_calendar::{update_calendar_admin_controller, update_calendar_controller};

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/calendar", web::post().to(create_calendar_controller));
    cfg.route(
        "/user/{user_id}/calendar",
        web::post().to(create_calendar_admin_controller),
    );

    cfg.route(
        "/calendar/meta",
        web::get().to(get_calendars_by_meta_controller),
    );

    cfg.route(
        "/calendar/{calendar_id}",
        web::get().to(get_calendar_controller),
    );
    cfg.route(
        "/user/calendar/{calendar_id}",
        web::get().to(get_calendar_admin_controller),
    );

    cfg.route(
        "/calendar/{calendar_id}",
        web::delete().to(delete_calendar_controller),
    );
    cfg.route(
        "/user/calendar/{calendar_id}",
        web::delete().to(delete_calendar_admin_controller),
    );

    cfg.route(
        "/calendar/{calendar_id}",
        web::put().to(update_calendar_controller),
    );
    cfg.route(
        "/user/calendar/{calendar_id}",
        web::put().to(update_calendar_admin_controller),
    );

    cfg.route(
        "/calendar/{calendar_id}/events",
        web::get().to(get_calendar_events_controller),
    );
    cfg.route(
        "/user/calendar/{calendar_id}/events",
        web::get().to(get_calendar_events_admin_controller),
    );

    // Calendar providers
    cfg.route(
        "/calendar/providers/google",
        web::get().to(get_google_calendars_controller),
    );
    cfg.route(
        "/user/{user_id}/calendar/provider/google",
        web::get().to(get_google_calendars_admin_controller),
    );
}
