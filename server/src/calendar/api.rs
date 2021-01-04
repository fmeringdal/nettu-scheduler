use super::usecases::create_calendar::{
    create_calendar_admin_controller, create_calendar_controller,
};
use super::usecases::delete_calendar::delete_calendar_controller;
use super::usecases::get_calendar::get_calendar_controller;
use super::usecases::get_calendar_events::get_calendar_events_controller;
use super::usecases::get_user_bookingslots::get_user_bookingslots_controller;
use super::usecases::get_user_freebusy::get_user_freebusy_controller;
use actix_web::web;

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
        "/calendar/{calendar_id}/events",
        web::get().to(get_calendar_events_controller),
    );
    cfg.route(
        "/user/{external_user_id}/freebusy",
        web::get().to(get_user_freebusy_controller),
    );
    cfg.route(
        "/user/{external_user_id}/booking",
        web::get().to(get_user_bookingslots_controller),
    );
}
