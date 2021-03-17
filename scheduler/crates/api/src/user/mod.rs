pub mod create_user;
mod delete_user;
mod get_me;
mod get_user;
mod get_user_freebusy;
mod get_users_by_meta;
mod update_user;

use actix_web::web;
use create_user::create_user_controller;
use delete_user::delete_user_controller;
use get_me::get_me_controller;
use get_user::get_user_controller;
use get_user_freebusy::get_freebusy_controller;
use get_users_by_meta::get_users_by_meta_controller;
use update_user::update_user_controller;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/user", web::post().to(create_user_controller));
    cfg.route("/me", web::get().to(get_me_controller));
    cfg.route("/user/meta", web::get().to(get_users_by_meta_controller));
    cfg.route("/user/{user_id}", web::get().to(get_user_controller));
    cfg.route("/user/{user_id}", web::put().to(update_user_controller));
    cfg.route("/user/{user_id}", web::delete().to(delete_user_controller));
    cfg.route(
        "/user/{user_id}/freebusy",
        web::get().to(get_freebusy_controller),
    );
}
