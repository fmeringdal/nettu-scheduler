pub mod create_user;
mod delete_user;
mod get_user;
mod get_user_freebusy;

use actix_web::web;
use create_user::create_user_controller;
use delete_user::delete_user_controller;
use get_user::get_user_controller;
use get_user_freebusy::get_freebusy_controller;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/user", web::post().to(create_user_controller));
    cfg.route("/user/{user_id}", web::get().to(get_user_controller));
    cfg.route("/user/{user_id}", web::delete().to(delete_user_controller));
    cfg.route(
        "/user/{external_user_id}/freebusy",
        web::get().to(get_freebusy_controller),
    );
}
