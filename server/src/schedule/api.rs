use super::usecases::delete_schedule::delete_schedule_controller;
use super::usecases::get_schedule::get_schedule_controller;
use super::usecases::{
    create_schedule::{create_schedule_admin_controller, create_schedule_controller},
    update_schedule::update_schedule_controller,
};
use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    // create schedule jwt token controller
    cfg.route("/schedule", web::post().to(create_schedule_controller));
    // create schedule account admin controller
    cfg.route(
        "/user/{user_id}/schedule",
        web::post().to(create_schedule_admin_controller),
    );
    cfg.route(
        "/schedule/{schedule_id}",
        web::get().to(get_schedule_controller),
    );
    cfg.route(
        "/schedule/{schedule_id}",
        web::delete().to(delete_schedule_controller),
    );
    cfg.route(
        "/schedule/{schedule_id}",
        web::put().to(update_schedule_controller),
    );
}
