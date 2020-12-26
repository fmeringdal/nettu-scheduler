use actix_web::{web, Responder};
use crate::shared::usecase::UseCase;
use super::usecases::create_event::CreateEventUseCase;
use std::sync::Arc;
use super::usecases::create_event::CreateEventReq;
use super::repo::IEventRepo;

pub fn configure_routes(cfg: &mut web::ServiceConfig, event_repo: Arc<dyn IEventRepo>) {
    let create_event_usecase = CreateEventUseCase {
        event_repo: Arc::clone(&event_repo),
    };
    cfg.app_data(web::Data::new(create_event_usecase));
    cfg.route("/events", web::post().to(create_event_controller));
}

async fn create_event_controller(
    create_event_usecase: web::Data<CreateEventUseCase>,
    req: web::Json<CreateEventReq>,
) -> impl Responder {
    let res = create_event_usecase.execute(req.0).await;
    "Hello, from create event we are up and running!\r\n"
}
