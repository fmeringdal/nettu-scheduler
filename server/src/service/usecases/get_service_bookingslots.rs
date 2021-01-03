use crate::{
    api::Context,
    calendar::domain::date,
    event::domain::booking_slots::{
        get_service_bookingslots, BookingSlotsOptions, ServiceBookingSlot, ServiceBookingSlotDTO,
    },
    shared::auth::ensure_nettu_acct_header,
};
use crate::{
    calendar::{
        repos::ICalendarRepo,
        usecases::get_user_freebusy::{
            get_user_freebusy_usecase, GetUserFreeBusyReq, GetUserFreeBusyUseCaseCtx,
        },
    },
    event::{domain::booking_slots::UserFreeEvents, repos::IEventRepo},
    service::repos::IServiceRepo,
};
use actix_web::{web, HttpRequest, HttpResponse};
use chrono::{prelude::*, Duration};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct PathParams {
    service_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryParams {
    iana_tz: Option<String>,
    duration: i64,
    interval: i64,
    date: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct APIRes {
    booking_slots: Vec<ServiceBookingSlotDTO>,
}

pub async fn get_service_bookingslots_controller(
    http_req: HttpRequest,
    query_params: web::Query<QueryParams>,
    path_params: web::Path<PathParams>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let _account = match ensure_nettu_acct_header(&http_req) {
        Ok(a) => a,
        Err(e) => return e,
    };

    let req = UsecaseReq {
        service_id: path_params.service_id.clone(),
        iana_tz: query_params.iana_tz.clone(),
        date: query_params.date.clone(),
        duration: query_params.duration,
    };
    let ctx = UsecaseCtx {
        event_repo: ctx.repos.event_repo.clone(),
        calendar_repo: ctx.repos.calendar_repo.clone(),
        service_repo: ctx.repos.service_repo.clone(),
    };
    let res = get_service_bookingslots_usecase(req, ctx).await;

    match res {
        Ok(r) => {
            let res = APIRes {
                booking_slots: r
                    .booking_slots
                    .iter()
                    .map(|slot| ServiceBookingSlotDTO::new(slot))
                    .collect(),
            };
            HttpResponse::Ok().json(res)
        }
        Err(e) => match e {
            UsecaseErrors::InvalidDateError(msg) => {
                HttpResponse::UnprocessableEntity().body(format!(
                    "Invalid datetime: {}. Should be YYYY-MM-DD, e.g. January 1. 2020 => 2020-1-1",
                    msg
                ))
            }
            UsecaseErrors::InvalidTimezoneError(msg) => {
                HttpResponse::UnprocessableEntity().body(format!(
                    "Invalid timezone: {}. It should be a valid IANA TimeZone.",
                    msg
                ))
            }
            UsecaseErrors::ServiceNotFoundError => HttpResponse::NotFound().finish(),
        },
    }
}

struct UsecaseReq {
    pub service_id: String,
    pub date: String,
    pub iana_tz: Option<String>,
    pub duration: i64,
}

struct UsecaseRes {
    booking_slots: Vec<ServiceBookingSlot>,
}

struct UsecaseCtx {
    pub service_repo: Arc<dyn IServiceRepo>,
    pub calendar_repo: Arc<dyn ICalendarRepo>,
    pub event_repo: Arc<dyn IEventRepo>,
}

#[derive(Debug)]
enum UsecaseErrors {
    ServiceNotFoundError,
    InvalidDateError(String),
    InvalidTimezoneError(String),
}

async fn get_service_bookingslots_usecase(
    req: UsecaseReq,
    ctx: UsecaseCtx,
) -> Result<UsecaseRes, UsecaseErrors> {
    let tz: Tz = match req.iana_tz.unwrap_or(String::from("UTC")).parse() {
        Ok(tz) => tz,
        Err(_) => return Err(UsecaseErrors::InvalidTimezoneError(req.date)),
    };

    let parsed_date = match date::is_valid_date(&req.date) {
        Ok(val) => val,
        Err(_) => return Err(UsecaseErrors::InvalidDateError(req.date.clone())),
    };

    let date = tz.ymd(parsed_date.0, parsed_date.1, parsed_date.2);

    let start_of_day = date.and_hms(0, 0, 0);
    let end_of_day = (date + Duration::days(1)).and_hms(0, 0, 0);

    let service = match ctx.service_repo.find(&req.service_id).await {
        Some(s) => s,
        None => return Err(UsecaseErrors::ServiceNotFoundError),
    };

    let mut users_freebusy: Vec<UserFreeEvents> = Vec::with_capacity(service.users.len());

    for user in &service.users {
        let free_events = get_user_freebusy_usecase(
            GetUserFreeBusyReq {
                calendar_ids: Some(user.calendar_ids.clone()),
                end_ts: end_of_day.timestamp_millis(),
                start_ts: start_of_day.timestamp_millis(),
                user_id: user.user_id.clone(),
            },
            GetUserFreeBusyUseCaseCtx {
                calendar_repo: Arc::clone(&ctx.calendar_repo),
                event_repo: Arc::clone(&ctx.event_repo),
            },
        )
        .await;

        match free_events {
            Ok(free_events) => {
                users_freebusy.push(UserFreeEvents {
                    free_events: free_events.free,
                    user_id: user.user_id.clone(),
                });
            }
            Err(e) => {
                println!("Error getting user freebusy: {:?}", e);
            }
        }
    }

    let booking_slots = get_service_bookingslots(
        users_freebusy,
        &BookingSlotsOptions {
            interval: 1000 * 60 * 15, // 15 minutes
            duration: req.duration,
            end_ts: end_of_day.timestamp_millis(),
            start_ts: start_of_day.timestamp_millis(),
        },
    );

    Ok(UsecaseRes { booking_slots })
}
