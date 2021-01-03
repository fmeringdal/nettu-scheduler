use super::get_user_freebusy::{
    get_user_freebusy_usecase, GetUserFreeBusyReq, GetUserFreeBusyUseCaseCtx,
};
use crate::{
    api::Context,
    calendar::domain::date,
    event::domain::booking_slots::{get_booking_slots, BookingSlot, BookingSlotsOptions},
    shared::auth::ensure_nettu_acct_header,
    user::domain::User,
};
use actix_web::{web, HttpRequest, HttpResponse};
use chrono::{prelude::*, Duration};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct UserPathParams {
    external_user_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserBookingQuery {
    iana_tz: Option<String>,
    duration: i64,
    date: String,
    calendar_ids: Option<String>,
}

pub async fn get_user_bookingslots_controller(
    http_req: HttpRequest,
    query_params: web::Query<UserBookingQuery>,
    params: web::Path<UserPathParams>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let account = match ensure_nettu_acct_header(&http_req) {
        Ok(a) => a,
        Err(e) => return e,
    };
    let calendar_ids = match &query_params.calendar_ids {
        Some(calendar_ids) => Some(calendar_ids.split(",").map(|s| String::from(s)).collect()),
        None => None,
    };

    let req = GetUserBookingSlotsReq {
        user_id: User::create_id(&account, &params.external_user_id),
        calendar_ids,
        iana_tz: query_params.iana_tz.clone(),
        date: query_params.date.clone(),
        duration: query_params.duration,
    };
    let ctx = GetUserFreeBusyUseCaseCtx {
        event_repo: ctx.repos.event_repo.clone(),
        calendar_repo: ctx.repos.calendar_repo.clone(),
    };
    let res = get_user_bookingslots_usecase(req, ctx).await;

    match res {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => match e {
            GetUserBookingSlotsErrors::InvalidDateError(msg) => HttpResponse::UnprocessableEntity()
                .body(format!(
                    "Invalid datetime: {}. Should be YYYY-MM-DD, e.g. January 1. 2020 => 2020-1-1",
                    msg
                )),
            GetUserBookingSlotsErrors::InvalidTimezoneError(msg) => {
                HttpResponse::UnprocessableEntity().body(format!(
                    "Invalid timezone: {}. It should be a valid IANA TimeZone.",
                    msg
                ))
            }
            GetUserBookingSlotsErrors::UserFreebusyError => {
                HttpResponse::InternalServerError().finish()
            }
        },
    }
}

#[derive(Serialize, Deserialize)]
pub struct GetUserBookingSlotsReq {
    pub user_id: String,
    pub calendar_ids: Option<Vec<String>>,
    pub date: String,
    pub iana_tz: Option<String>,
    pub duration: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetUserBookingSlotsResponse {
    booking_slots: Vec<BookingSlot>,
}

#[derive(Debug)]
pub enum GetUserBookingSlotsErrors {
    InvalidDateError(String),
    InvalidTimezoneError(String),
    UserFreebusyError,
}

async fn get_user_bookingslots_usecase(
    req: GetUserBookingSlotsReq,
    ctx: GetUserFreeBusyUseCaseCtx,
) -> Result<GetUserBookingSlotsResponse, GetUserBookingSlotsErrors> {
    let tz: Tz = match req.iana_tz.unwrap_or(String::from("UTC")).parse() {
        Ok(tz) => tz,
        Err(_) => return Err(GetUserBookingSlotsErrors::InvalidTimezoneError(req.date)),
    };

    let parsed_date = match date::is_valid_date(&req.date) {
        Ok(val) => val,
        Err(_) => {
            return Err(GetUserBookingSlotsErrors::InvalidDateError(
                req.date.clone(),
            ))
        }
    };
    let date = tz.ymd(parsed_date.0, parsed_date.1, parsed_date.2);

    let start_of_day = date.and_hms(0, 0, 0);
    let end_of_day = (date + Duration::days(1)).and_hms(0, 0, 0);

    let free_events = get_user_freebusy_usecase(
        GetUserFreeBusyReq {
            calendar_ids: req.calendar_ids,
            end_ts: end_of_day.timestamp_millis(),
            start_ts: start_of_day.timestamp_millis(),
            user_id: req.user_id,
        },
        ctx,
    )
    .await;

    match free_events {
        Ok(free_events) => {
            let booking_slots = get_booking_slots(
                &free_events.free,
                &BookingSlotsOptions {
                    interval: 1000 * 60 * 15, // 15 minutes
                    duration: req.duration,
                    end_ts: end_of_day.timestamp_millis(),
                    start_ts: start_of_day.timestamp_millis(),
                },
            );

            Ok(GetUserBookingSlotsResponse { booking_slots })
        }
        Err(_e) => Err(GetUserBookingSlotsErrors::UserFreebusyError),
    }
}
