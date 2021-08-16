use actix_web::{web, HttpResponse};
use nettu_scheduler_api_structs::get_service_health::*;

async fn status() -> HttpResponse {
    HttpResponse::Ok().json(APIResponse {
        message: "Yo! We are up!\r\n".into(),
    })
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(status));
}
