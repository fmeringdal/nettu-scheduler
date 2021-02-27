use actix_web::web;

async fn status() -> &'static str {
    "Yo! We are up!\r\n"
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(status));
}
