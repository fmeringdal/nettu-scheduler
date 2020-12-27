use super::repo::ICalendarRepo;
use super::usecases::create_calendar::{CreateCalendarReq, CreateCalendarUseCase};
use super::usecases::delete_calendar::{DeleteCalendarReq, DeleteCalendarUseCase};
use super::usecases::get_calendar::{GetCalendarReq, GetCalendarUseCase};
use super::usecases::get_calendar_events::{GetCalendarEventsReq, GetCalendarEventsUseCase};
use crate::api::Context;
use crate::event::repo::IEventRepo;
use crate::shared::usecase::UseCase;
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use std::sync::Arc;

pub fn configure_routes(cfg: &mut web::ServiceConfig, ctx: Arc<Context>) {
    // Add usecases to actix data
    let create_calendar_usecase = CreateCalendarUseCase {
        calendar_repo: Arc::clone(&ctx.repos.calendar_repo),
    };

    let delete_calendar_usecase = DeleteCalendarUseCase {
        event_repo: Arc::clone(&ctx.repos.event_repo),
        calendar_repo: Arc::clone(&ctx.repos.calendar_repo),
    };
    let get_calendar_usecase = GetCalendarUseCase {
        calendar_repo: Arc::clone(&ctx.repos.calendar_repo),
    };
    let get_calendar_events_usecase = GetCalendarEventsUseCase {
        event_repo: Arc::clone(&ctx.repos.event_repo),
        calendar_repo: Arc::clone(&ctx.repos.calendar_repo),
    };

    cfg.app_data(web::Data::new(create_calendar_usecase));
    cfg.app_data(web::Data::new(delete_calendar_usecase));
    cfg.app_data(web::Data::new(get_calendar_usecase));
    cfg.app_data(web::Data::new(get_calendar_events_usecase));

    // Hookup Routes to usecases
    cfg.route("/calendar", web::post().to(create_calendar_controller));
    cfg.route(
        "/calendar/{calendar_id}",
        web::get().to(get_event_controller),
    );
    cfg.route(
        "/calendar/{calendar_id}",
        web::delete().to(delete_calendar_controller),
    );
    cfg.route(
        "/calendar/{calendar_id}/events",
        web::get().to(get_calendar_events_controller),
    );
}

async fn create_calendar_controller(
    create_calendar_usecase: web::Data<CreateCalendarUseCase>,
    req: web::Json<CreateCalendarReq>,
) -> impl Responder {
    let res = create_calendar_usecase.execute(req.0).await;
    "Hello, from create event we are up and running!\r\n"
}

#[derive(Debug, Deserialize)]
struct CalendarPathParams {
    calendar_id: String,
}

async fn delete_calendar_controller(
    delete_calendar_usecase: web::Data<DeleteCalendarUseCase>,
    params: web::Path<CalendarPathParams>,
) -> impl Responder {
    let req = DeleteCalendarReq {
        calendar_id: params.calendar_id.clone(),
    };
    let res = delete_calendar_usecase.execute(req).await;
    return match res {
        Ok(_) => HttpResponse::Ok().body("Calendar deleted"),
        Err(_) => HttpResponse::NoContent().finish(),
    };
}

async fn get_event_controller(
    get_calendar_usecase: web::Data<GetCalendarUseCase>,
    params: web::Path<CalendarPathParams>,
) -> impl Responder {
    let req = GetCalendarReq {
        calendar_id: params.calendar_id.clone(),
    };
    let res = get_calendar_usecase.execute(req).await;
    return match res {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(_) => HttpResponse::NoContent().finish(),
    };
}

async fn get_calendar_events_controller(
    get_calendar_events_usecase: web::Data<GetCalendarEventsUseCase>,
    params: web::Path<GetCalendarEventsReq>,
) -> impl Responder {
    let req = GetCalendarEventsReq {
        calendar_id: params.calendar_id.clone(),
        start_ts: 0,
        end_ts: 0,
    };
    let res = get_calendar_events_usecase.execute(req).await;

    return match res {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(_) => HttpResponse::NoContent().finish(),
    };
}
