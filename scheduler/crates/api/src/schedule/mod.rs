mod create_schedule;
mod delete_schedule;
mod get_schedule;
mod get_schedules_by_meta;
mod update_schedule;

use actix_web::web;
use create_schedule::{create_schedule_admin_controller, create_schedule_controller};
use delete_schedule::{delete_schedule_admin_controller, delete_schedule_controller};
use get_schedule::{get_schedule_admin_controller, get_schedule_controller};
use get_schedules_by_meta::get_schedules_by_meta_controller;
use update_schedule::{update_schedule_admin_controller, update_schedule_controller};

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/schedule", web::post().to(create_schedule_controller));
    cfg.route(
        "/user/{user_id}/schedule",
        web::post().to(create_schedule_admin_controller),
    );

    cfg.route(
        "/schedule/meta",
        web::get().to(get_schedules_by_meta_controller),
    );

    cfg.route(
        "/schedule/{schedule_id}",
        web::get().to(get_schedule_controller),
    );
    cfg.route(
        "/user/schedule/{schedule_id}",
        web::get().to(get_schedule_admin_controller),
    );

    cfg.route(
        "/schedule/{schedule_id}",
        web::delete().to(delete_schedule_controller),
    );
    cfg.route(
        "/user/schedule/{schedule_id}",
        web::delete().to(delete_schedule_admin_controller),
    );

    cfg.route(
        "/schedule/{schedule_id}",
        web::put().to(update_schedule_controller),
    );
    cfg.route(
        "/user/schedule/{schedule_id}",
        web::put().to(update_schedule_admin_controller),
    );
}
