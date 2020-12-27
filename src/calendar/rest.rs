use super::repo::ICalendarRepo;
use super::usecases::create_calendar::{CreateCalendarReq, CreateCalendarUseCase};
use super::usecases::delete_calendar::{DeleteCalendarReq, DeleteCalendarUseCase};
use super::usecases::get_calendar::{GetCalendarReq, GetCalendarUseCase};
use super::usecases::get_calendar_events::{
    GetCalendarEventsErrors, GetCalendarEventsReq, GetCalendarEventsUseCase,
};
use super::usecases::get_user_bookingslots::{
    GetUserBookingSlotsErrors, GetUserBookingSlotsReq, GetUserBookingSlotsUseCase,
};
use super::usecases::get_user_freebusy::{
    GetUserFreeBusyErrors, GetUserFreeBusyReq, GetUserFreeBusyUseCase,
};
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
    let get_user_freebusy_usecase = GetUserFreeBusyUseCase {
        event_repo: Arc::clone(&ctx.repos.event_repo),
        calendar_repo: Arc::clone(&ctx.repos.calendar_repo),
    };
    let get_user_freebusy_usecase_arc = Arc::new(get_user_freebusy_usecase);
    let get_user_bookingslots_usecase = GetUserBookingSlotsUseCase {
        get_user_freebusy_usecase: Arc::clone(&get_user_freebusy_usecase_arc),
    };

    cfg.app_data(web::Data::new(create_calendar_usecase));
    cfg.app_data(web::Data::new(delete_calendar_usecase));
    cfg.app_data(web::Data::new(get_calendar_usecase));
    cfg.app_data(web::Data::new(get_calendar_events_usecase));
    cfg.app_data(web::Data::new(get_user_freebusy_usecase_arc));
    cfg.app_data(web::Data::new(get_user_bookingslots_usecase));

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
    cfg.route(
        "/user/{user_id}/freebusy",
        web::get().to(get_user_freebusy_controller),
    );
    cfg.route(
        "/user/{user_id}/booking",
        web::get().to(get_user_bookingslots_controller),
    );
}

async fn create_calendar_controller(
    create_calendar_usecase: web::Data<CreateCalendarUseCase>,
    req: web::Json<CreateCalendarReq>,
) -> impl Responder {
    println!("got here?");
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
        Err(_) => HttpResponse::NotFound().finish(),
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
        Err(_) => HttpResponse::NotFound().finish(),
    };
}

#[derive(Debug, Deserialize)]
struct TimespanBodyReq {
    pub start_ts: i64,
    pub end_ts: i64,
}

async fn get_calendar_events_controller(
    get_calendar_events_usecase: web::Data<GetCalendarEventsUseCase>,
    body: web::Json<TimespanBodyReq>,
    params: web::Path<CalendarPathParams>,
) -> impl Responder {
    let req = GetCalendarEventsReq {
        calendar_id: params.calendar_id.clone(),
        start_ts: body.start_ts,
        end_ts: body.end_ts,
    };
    let res = get_calendar_events_usecase.execute(req).await;

    match res {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => match e {
            GetCalendarEventsErrors::InvalidTimespanError => {
                HttpResponse::UnprocessableEntity().finish()
            }
            GetCalendarEventsErrors::NotFoundError => HttpResponse::NotFound().finish(),
        },
    }
}

#[derive(Debug, Deserialize)]
struct UserPathParams {
    user_id: String,
}

#[derive(Debug, Deserialize)]
struct UserFreebusyBodyReq {
    start_ts: i64,
    end_ts: i64,
    calendar_ids: Option<Vec<String>>,
}

async fn get_user_freebusy_controller(
    get_user_freebusy_usecase: web::Data<Arc<GetUserFreeBusyUseCase>>,
    body: web::Query<UserFreebusyBodyReq>,
    params: web::Path<UserPathParams>,
) -> impl Responder {
    let req = GetUserFreeBusyReq {
        user_id: params.user_id.clone(),
        calendar_ids: body.calendar_ids.clone(),
        start_ts: body.start_ts,
        end_ts: body.end_ts,
    };
    let res = get_user_freebusy_usecase.execute(req).await;

    match res {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => match e {
            GetUserFreeBusyErrors::InvalidTimespanError => {
                HttpResponse::UnprocessableEntity().finish()
            }
        },
    }
}

#[derive(Debug, Deserialize)]
struct UserBookingBodyReq {
    iana_tz: Option<String>,
    duration: i64,
    date: String,
    calendar_ids: Option<Vec<String>>,
}

async fn get_user_bookingslots_controller(
    get_user_bookingslots_usecase: web::Data<GetUserBookingSlotsUseCase>,
    query_params: web::Query<UserBookingBodyReq>,
    params: web::Path<UserPathParams>,
) -> impl Responder {
    let req = GetUserBookingSlotsReq {
        user_id: params.user_id.clone(),
        calendar_ids: query_params.calendar_ids.clone(),
        iana_tz: query_params.iana_tz.clone(),
        date: query_params.date.clone(),
        duration: query_params.duration,
    };
    let res = get_user_bookingslots_usecase.execute(req).await;

    match res {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => match e {
            GetUserBookingSlotsErrors::InvalidTimespanError => {
                HttpResponse::UnprocessableEntity().finish()
            }
        },
    }
}
