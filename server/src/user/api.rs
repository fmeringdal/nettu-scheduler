use super::usecases::create_user::create_user_controller;
use super::usecases::delete_user::delete_user_controller;
use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/user", web::post().to(create_user_controller));
    cfg.route("/user/{user_id}", web::delete().to(delete_user_controller));
}
