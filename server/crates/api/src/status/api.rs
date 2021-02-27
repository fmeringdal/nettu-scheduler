use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct StatusResponse {
    message: String,
}

async fn status() -> HttpResponse {
    HttpResponse::Ok().json(StatusResponse {
        message: "Yo! We are up!\r\n".into(),
    })
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(status));
}
