use super::usecases::create_event::{CreateEventReq, CreateEventUseCase};
use super::usecases::delete_event::{DeleteEventReq, DeleteEventUseCase};
use super::usecases::get_event::{GetEventReq, GetEventUseCase};
use super::usecases::get_event_instances::{GetEventInstancesReq, GetEventInstancesUseCase};
use super::usecases::update_event::{UpdateEventReq, UpdateEventUseCase};
use super::{domain::event::RRuleOptions, repo::IEventRepo};
use crate::api::Context;
use crate::shared::usecase::UseCase;
use actix_web::{web, Responder, HttpResponse};
use serde::Deserialize;
use std::sync::Arc;

pub fn configure_routes(cfg: &mut web::ServiceConfig, ctx: Arc<Context>) {
    // Add usecases to actix data
    let create_event_usecase = CreateEventUseCase {
        event_repo: Arc::clone(&ctx.repos.event_repo),
    };
    let update_event_usecase = UpdateEventUseCase {
        event_repo: Arc::clone(&ctx.repos.event_repo),
    };
    let delete_event_usecase = DeleteEventUseCase {
        event_repo: Arc::clone(&ctx.repos.event_repo),
    };
    let get_event_usecase = GetEventUseCase {
        event_repo: Arc::clone(&ctx.repos.event_repo),
    };
    let get_event_instances_usecase = GetEventInstancesUseCase {
        event_repo: Arc::clone(&ctx.repos.event_repo),
    };

    cfg.app_data(web::Data::new(create_event_usecase));
    cfg.app_data(web::Data::new(update_event_usecase));
    cfg.app_data(web::Data::new(delete_event_usecase));
    cfg.app_data(web::Data::new(get_event_usecase));
    cfg.app_data(web::Data::new(get_event_instances_usecase));

    // Hookup Routes to usecases
    cfg.route("/events", web::post().to(create_event_controller));
    cfg.route("/events/{event_id}", web::get().to(get_event_controller));
    cfg.route("/events/{event_id}", web::put().to(update_event_controller));
    cfg.route(
        "/events/{event_id}",
        web::delete().to(delete_event_controller),
    );
    cfg.route(
        "/events/{event_id}/instances",
        web::get().to(get_event_instances_controller),
    );
}

async fn create_event_controller(
    create_event_usecase: web::Data<CreateEventUseCase>,
    req: web::Json<CreateEventReq>,
) -> impl Responder {
    let res = create_event_usecase.execute(req.0).await;
    "Hello, from create event we are up and running!\r\n"
}

#[derive(Deserialize)]
struct UpdateEventBody {
    start_ts: Option<i64>,
    duration: Option<i64>,
    rrule_options: Option<RRuleOptions>,
}

#[derive(Deserialize)]
struct EventPathParams {
    event_id: String,
}

async fn update_event_controller(
    update_event_usecase: web::Data<UpdateEventUseCase>,
    body: web::Json<UpdateEventBody>,
    params: web::Path<EventPathParams>,
) -> impl Responder {
    let req = UpdateEventReq {
        duration: body.duration,
        start_ts: body.start_ts,
        rrule_options: body.rrule_options.clone(),
        event_id: params.event_id.clone(),
    };
    let res = update_event_usecase.execute(req).await;
    "Hello, from create event we are up and running!\r\n"
}

async fn delete_event_controller(
    delete_event_usecase: web::Data<DeleteEventUseCase>,
    params: web::Path<EventPathParams>,
) -> impl Responder {
    let req = DeleteEventReq {
        event_id: params.event_id.clone(),
    };
    let res = delete_event_usecase.execute(req).await;
    return match res {
        Ok(_) => HttpResponse::Ok().body("Event deleted"),
        Err(_) => HttpResponse::NoContent().finish()
    }
}

async fn get_event_controller(
    get_event_usecase: web::Data<GetEventUseCase>,
    params: web::Path<EventPathParams>,
) -> impl Responder {
    let req = GetEventReq {
        event_id: params.event_id.clone(),
    };
    let res = get_event_usecase.execute(req).await;
    return match res {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(_) => HttpResponse::NoContent().finish()
    }
}

async fn get_event_instances_controller(
    get_event_instances_usecase: web::Data<GetEventInstancesUseCase>,
    params: web::Path<GetEventInstancesReq>,
) -> impl Responder {
    let req = GetEventInstancesReq {
        event_id: params.event_id.clone(),
    };
    let res = get_event_instances_usecase.execute(req).await;
    

    return match res {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(_) => HttpResponse::NoContent().finish()
    }
}
